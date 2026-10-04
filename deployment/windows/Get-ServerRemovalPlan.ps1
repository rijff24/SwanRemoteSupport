[CmdletBinding()]
param([Parameter(Mandatory)][string]$InstallDirectory,[Parameter(Mandatory)][string]$DataDirectory,
      [Parameter(Mandatory)][string]$ServiceBinary)
$ErrorActionPreference = 'Stop'
$install = [IO.Path]::GetFullPath($InstallDirectory)
$data = [IO.Path]::GetFullPath($DataDirectory)
if ([IO.Path]::GetFileName($install) -ne 'Swan Company Server' -or [IO.Path]::GetFileName($data) -ne 'SwanCompanyServer') { throw 'Unexpected company installation directory.' }
$expected = '"'+(Join-Path $install 'swan-management.exe')+'" --service'
if (-not [string]::Equals($ServiceBinary.Trim(),$expected,[StringComparison]::OrdinalIgnoreCase)) { throw 'Company service executable does not match this installation.' }
foreach ($directory in @($install,$data,(Join-Path $install 'components'))) {
    if (Test-Path -LiteralPath $directory) {
        if ((Get-Item -LiteralPath $directory -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Company installation directory is a reparse point.' }
    }
}
$receiptPath = Join-Path $data 'installation.json'
if ((Get-Item -LiteralPath $receiptPath -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Company receipt is a reparse point.' }
$receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
if ($receipt.schema -ne 1 -or $receipt.service_name -ne 'SwanCompanyServer' -or $receipt.full_stack -isnot [bool] -or
    -not [string]::Equals($receipt.install_directory,$install,[StringComparison]::OrdinalIgnoreCase) -or
    -not [string]::Equals($receipt.data_directory,$data,[StringComparison]::OrdinalIgnoreCase)) { throw 'Company installation receipt does not match.' }
$rules = @($receipt.firewall_rules)
$expectedRules = @()
if ($receipt.full_stack) { $expectedRules = @('SwanCompanyServer-TCP','SwanCompanyServer-UDP') }
if ($rules.Count -ne $expectedRules.Count -or @(Compare-Object $rules $expectedRules).Count) { throw 'Unexpected company firewall ownership.' }
$files = @(Join-Path $install 'swan-management.exe')
if ($receipt.full_stack) {
    foreach ($name in @('hbbs.exe','hbbr.exe','caddy.exe','Caddy-LICENSE.txt','RustDesk-LICENSE.txt','THIRD-PARTY.txt','server-components.json')) { $files += Join-Path (Join-Path $install 'components') $name }
}
foreach ($file in $files) {
    if (Test-Path -LiteralPath $file) {
        $item = Get-Item -LiteralPath $file -Force
        if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unexpected company installation file type.' }
    }
}
[pscustomobject]@{Files=$files; FirewallRules=$rules; InstallDirectory=$install; DataDirectory=$data; FullStack=$receipt.full_stack}
