$ErrorActionPreference = 'Stop'
# Load only the pure guard function. Do not execute the recovery script, query
# services, open MSI databases, or invoke installation/removal on this host.
$tokens=$null; $errors=$null
foreach ($source in @('Restore-ReleaseMsi.ps1','Restore-CustomerExe.ps1')) {
$ast=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot $source),[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Recovery script failed parsing.' }
$function=$ast.Find({param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Assert-CustomerServiceRecord'},$true)
if ($null -eq $function) { throw 'Service identity guard missing.' }
. ([ScriptBlock]::Create($function.Extent.Text))
$expected='"C:\IsolatedFixture\Swan Remote Support.exe" --service'
$valid=@{PathName=$expected;StartName='LocalSystem';State='Running';StartMode='Auto'}
Assert-CustomerServiceRecord $valid $expected $true
foreach ($change in @(@{PathName='"C:\OtherInstallation\Swan Remote Support.exe" --service'},@{StartName='OtherAccount'},@{State='Stopped'},@{StartMode='Manual'},@{PathName=($expected+' --other')})) {
    $record=$valid.Clone()
    foreach ($key in $change.Keys) { $record[$key]=$change[$key] }
    $rejected=$false
    try { Assert-CustomerServiceRecord $record $expected $true } catch { $rejected=$true }
    if (-not $rejected) { throw 'Unsafe service identity or startup state accepted.' }
}
$stopped=$valid.Clone();$stopped.State='Stopped'
Assert-CustomerServiceRecord $stopped $expected $false
}
Write-Output 'Service guard component checks passed; no native service or installer operations executed.'
