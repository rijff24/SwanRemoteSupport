[CmdletBinding(SupportsShouldProcess=$true)]
param()
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Server removal requires administrator rights.' }
$install = Join-Path $env:ProgramFiles 'Swan Company Server'
$data = Join-Path $env:ProgramData 'SwanCompanyServer'
$service = Get-CimInstance Win32_Service -Filter "Name='SwanCompanyServer'"
if (-not $service) { throw 'Company service is not installed. Retained files/data require the recovery procedure.' }
$plan = & (Join-Path $PSScriptRoot 'Get-ServerRemovalPlan.ps1') -InstallDirectory $install -DataDirectory $data -ServiceBinary $service.PathName
# Check rule ownership before stopping anything. Never remove unrelated rules.
foreach ($name in $plan.FirewallRules) {
    $rule = Get-NetFirewallRule -Name $name -ErrorAction SilentlyContinue
    if ($rule -and $rule.Group -ne 'SwanCompanyServer') { throw 'Company firewall rule ownership changed.' }
}
if ($PSCmdlet.ShouldProcess('SwanCompanyServer','Remove company service, owned firewall rules and packaged files; retain company data')) {
    Stop-Service -Name SwanCompanyServer
    (Get-Service -Name SwanCompanyServer).WaitForStatus([ServiceProcess.ServiceControllerStatus]::Stopped,[TimeSpan]::FromSeconds(30))
    & sc.exe delete SwanCompanyServer | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Company service removal failed.' }
    foreach ($name in $plan.FirewallRules) {
        if (Get-NetFirewallRule -Name $name -ErrorAction SilentlyContinue) { Remove-NetFirewallRule -Name $name }
    }
    foreach ($file in $plan.Files) { if (Test-Path -LiteralPath $file) { Remove-Item -LiteralPath $file } }
    # Nonrecursive removal leaves any unexpected files untouched.
    foreach ($directory in @((Join-Path $install 'components'),$install)) {
        if ((Test-Path -LiteralPath $directory) -and -not @(Get-ChildItem -LiteralPath $directory -Force).Count) { Remove-Item -LiteralPath $directory }
    }
    Write-Host "Company service removed. Company identity, database, transport keys, TLS storage and logs remain in $data."
    Write-Host 'Preserve an encrypted backup before reinstalling or restoring. Other RustDesk and Tailscale services were not changed.'
}
