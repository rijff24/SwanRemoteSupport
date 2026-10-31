[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Package)
$ErrorActionPreference = 'Stop'
# Database mode 0 is read-only. No installation or custom action is invoked.
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.OpenDatabase((Resolve-Path -LiteralPath $Package).Path, 0)
function Read-Property([string]$Name) {
    if ($Name -notin @('ProductCode','UpgradeCode','ProductVersion')) { throw 'Unsupported MSI property.' }
    $view = $database.OpenView('SELECT `Value` FROM `Property` WHERE `Property` = ''' + $Name + '''')
    try {
        $null = $view.Execute()
        $row = $view.Fetch()
        if ($null -eq $row) { throw "MSI lacks $Name." }
        return $row.StringData(1)
    } finally { $null = $view.Close() }
}
$summary = $database.SummaryInformation(0)
@{
    product_code = Read-Property 'ProductCode'
    upgrade_code = Read-Property 'UpgradeCode'
    product_version = Read-Property 'ProductVersion'
    template = $summary.Property(7)
} | ConvertTo-Json -Compress
