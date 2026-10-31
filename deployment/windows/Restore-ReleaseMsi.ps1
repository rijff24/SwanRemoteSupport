[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$PreviousPackage,
    [Parameter(Mandatory=$true)][string]$NextProductCode,
    [Parameter(Mandatory=$true)][string]$NextVersion,
    [Parameter(Mandatory=$true)][string]$PreviousProductCode,
    [Parameter(Mandatory=$true)][string]$PreviousVersion,
    [Parameter(Mandatory=$true)][string]$Directory,
    [Parameter(Mandatory=$true)][ValidateSet('customer','technician')][string]$Edition,
    [switch]$InspectOnly
)
$ErrorActionPreference = 'Stop'
$upgrade = if ($Edition -eq 'customer') { '{32A585D7-9A78-4AD2-AF72-D0266EFC709D}' } else { '{A4374699-436F-4917-9A4E-223F62A9E634}' }
$installer = New-Object -ComObject WindowsInstaller.Installer
function Read-Identity([string]$Path) {
    $database = $installer.OpenDatabase((Resolve-Path -LiteralPath $Path).Path, 0)
    $result = @{}
    foreach ($name in @('ProductCode','UpgradeCode','ProductVersion')) {
        $view = $database.OpenView('SELECT `Value` FROM `Property` WHERE `Property` = ''' + $name + '''')
        try {
            $null = $view.Execute()
            $row = $view.Fetch()
            if ($null -eq $row) { throw 'Missing MSI identity.' }
            $result[$name] = $row.StringData(1)
        } finally { $null = $view.Close() }
    }
    if ($database.SummaryInformation(0).Property(7).Split(';')[0] -cne 'x64') { throw 'Unexpected MSI platform.' }
    return $result
}
function Assert-Identity($Identity, [string]$Code, [string]$Version) {
    if ($Identity.ProductCode -ine $Code -or $Identity.UpgradeCode -ine $upgrade -or $Identity.ProductVersion -cne $Version) { throw 'MSI registration differs from the signed recovery identity.' }
}
foreach ($code in @($PreviousProductCode,$NextProductCode)) {
    if ($code -notmatch '^\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\}$') { throw 'Invalid recovery product code.' }
}
Assert-Identity (Read-Identity $PreviousPackage) $PreviousProductCode $PreviousVersion
# Enumerate only this edition's upgrade family in the caller's installer context.
# Never remove an unexpected product or another user's installation.
foreach ($code in @($installer.RelatedProducts($upgrade))) {
    if ($code -ine $PreviousProductCode -and $code -ine $NextProductCode) { throw 'Unexpected installed release; explicit recovery required.' }
    $version = if ($code -ieq $NextProductCode) { $NextVersion } else { $PreviousVersion }
    Assert-Identity (Read-Identity ($installer.ProductInfo($code,'LocalPackage'))) $code $version
}
$msiexec = Join-Path ([Environment]::GetFolderPath('System')) 'msiexec.exe'
if ($PreviousProductCode -ieq $NextProductCode -and $PreviousVersion -cne $NextVersion) { throw 'Different versions must use different product codes.' }
$nextState = $installer.ProductState($NextProductCode)
if ($nextState -notin @(-1,1,5)) { throw 'Ambiguous next product registration.' }
$previousState = $installer.ProductState($PreviousProductCode)
if ($previousState -notin @(-1,1,5)) { throw 'Ambiguous previous product registration.' }
if ($InspectOnly) {
    @{ previous_product_code=$PreviousProductCode; next_product_code=$NextProductCode; next_state=$nextState; would_remove_next=($NextProductCode -ine $PreviousProductCode -and $nextState -eq 5); installer_executed=$false } | ConvertTo-Json -Compress
    return
}
function Assert-CustomerServiceRecord($Service, [string]$Expected, [bool]$Required) {
    if ($Service.PathName -ine $Expected -or $Service.StartName -ne 'LocalSystem') { throw 'Customer service belongs to another installation.' }
    if ($Required -and ($Service.State -ne 'Running' -or $Service.StartMode -ne 'Auto')) { throw 'Restored customer service is not running automatically.' }
}
function Assert-CustomerService([bool]$Required) {
    $services = @(Get-CimInstance -ClassName Win32_Service -Filter "Name='Swan Remote Support'")
    if ($services.Count -eq 0 -and -not $Required) { return }
    if ($services.Count -ne 1) { throw 'Expected exactly one customer service.' }
    $expected = '"' + [IO.Path]::GetFullPath((Join-Path $Directory 'Swan Remote Support.exe')) + '" --service'
    Assert-CustomerServiceRecord $services[0] $expected $Required
}
if ($Edition -eq 'customer') { Assert-CustomerService $false }
if ($NextProductCode -ine $PreviousProductCode -and $installer.ProductState($NextProductCode) -eq 5) {
    # Recheck the exact cached database immediately before narrowly removing it.
    Assert-Identity (Read-Identity ($installer.ProductInfo($NextProductCode,'LocalPackage'))) $NextProductCode $NextVersion
    # Suppress user-uninstall cancellation only for this signed recovery path.
    & $msiexec /x $NextProductCode /qn /norestart SWAN_RECOVERY=1
    if ($LASTEXITCODE -ne 0) { throw "Removal requires recovery or restart (exit $LASTEXITCODE)." }
}
$state = $installer.ProductState($PreviousProductCode)
$mode = if ($state -eq 5) { '/fvamus' } elseif ($state -in @(-1,1)) { '/i' } else { throw 'Ambiguous previous product registration.' }
# Quote paths as installer arguments; profile data never supplies commands.
& $msiexec $mode ('"' + (Resolve-Path -LiteralPath $PreviousPackage).Path + '"') /qn /norestart ('INSTALLFOLDER="' + [IO.Path]::GetFullPath($Directory) + '"')
if ($LASTEXITCODE -ne 0) { throw "Restoration requires recovery or restart (exit $LASTEXITCODE)." }
Assert-Identity (Read-Identity ($installer.ProductInfo($PreviousProductCode,'LocalPackage'))) $PreviousProductCode $PreviousVersion
if ($installer.ProductState($PreviousProductCode) -ne 5) { throw 'Previous product registration was not restored.' }
if ($Edition -eq 'customer') { Assert-CustomerService $true }
