[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Package)
$ErrorActionPreference = 'Stop'
# Read only: no installer actions or custom actions are executed.
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.OpenDatabase((Resolve-Path -LiteralPath $Package).Path, 0)
function Read-Property([string]$Name,[switch]$Optional) {
    $view = $database.OpenView("SELECT ``Value`` FROM ``Property`` WHERE ``Property`` = '$Name'")
    try {
        $null = $view.Execute()
        $row = $view.Fetch()
        if ($null -eq $row) {
            if ($Optional) { return '' }
            throw "MSI property is missing: $Name."
        }
        return $row.StringData(1)
    } finally { $null = $view.Close() }
}
function ConvertTo-MsiVersion([string]$Value) {
    # Windows Installer ignores a legacy package's fourth version field.
    if ($Value -notmatch '^(\d+)\.(\d+)\.(\d+)(?:\.\d+)?$') { throw 'MSI version cannot be compared safely.' }
    $parts = @([int]$Matches[1],[int]$Matches[2],[int]$Matches[3])
    if ($parts[0] -gt 255 -or $parts[1] -gt 255 -or $parts[2] -gt 65535) { throw 'MSI version exceeds supported fields.' }
    return [version]::new($parts[0],$parts[1],$parts[2])
}
$code = Read-Property 'ProductCode'
if ($code -notmatch '^\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\}$') { throw 'Invalid MSI product identity.' }
$upgrade = Read-Property 'UpgradeCode'
if ($upgrade -notmatch '^\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\}$') { throw 'Invalid MSI upgrade identity.' }
$version = ConvertTo-MsiVersion (Read-Property 'ProductVersion')
foreach ($related in $installer.RelatedProducts($upgrade)) {
    $registered = ConvertTo-MsiVersion ($installer.ProductInfo($related,'VersionString'))
    if ($registered -gt $version -or ($registered -eq $version -and $related -ine $code)) { throw "This package cannot replace registered product version $registered. Use a newer signed release; setup was not started." }
}
$state = $installer.ProductState($code)
if ($state -eq 5) { Write-Output '/fvamus' }
elseif ($state -in @(-1, 1)) { Write-Output '/i' }
elseif ($state -eq 2 -and [string]::IsNullOrEmpty((Read-Property 'ALLUSERS' -Optional))) {
    # INSTALLSTATE_ABSENT means another user has this product. An ordinary
    # per-user MSI can install independently for the current Windows account.
    Write-Output '/i'
}
else { throw "MSI product registration cannot be safely recovered (state $state)." }
