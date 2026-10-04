$ErrorActionPreference='Stop'
# Execute only the pure ownership guard, never task registration or removal.
$tokens=$null; $errors=$null
$ast=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'Update-RecoveryTask.ps1'),[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Recovery task script failed parsing.' }
$guard=$ast.Find({param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Assert-RecoveryTask'},$true)
if ($null -eq $guard) { throw 'Recovery task ownership guard missing.' }
. ([ScriptBlock]::Create($guard.Extent.Text))
$helper='C:\IsolatedFixture\updates\2\swan-agent.exe'
$action=@{Execute=$helper;Arguments='recover-update-task'}
$valid=@{TaskPath='\';Actions=@($action);Principal=@{UserId='S-1-5-18'}}
Assert-RecoveryTask $valid $helper @('S-1-5-18')
foreach ($change in @(
    @{TaskPath='\OtherProduct\'},
    @{Actions=@(@{Execute='C:\OtherInstallation\swan-agent.exe';Arguments='recover-update-task'})},
    @{Actions=@(@{Execute=$helper;Arguments='watch'})},
    @{Actions=@($action,$action)},
    @{Principal=@{UserId='OtherAccount'}}
)) {
    $record=$valid.Clone()
    foreach ($key in $change.Keys) { $record[$key]=$change[$key] }
    $rejected=$false
    try { Assert-RecoveryTask $record $helper @('S-1-5-18') } catch { $rejected=$true }
    if (-not $rejected) { throw 'Recovery task ownership mismatch was accepted.' }
}
Write-Output 'Recovery task guard checks passed; no native task operations executed.'
$fixture=Join-Path ([IO.Path]::GetTempPath()) ('swan-uninstall-cancellation-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $fixture
$marker=Join-Path $fixture 'pending-uninstall'
$rollback=Join-Path $fixture 'uninstall-cancellation-rollback.json'
try {
    $script=Join-Path $PSScriptRoot 'Update-UninstallCancellation.ps1'
    & $script -Directory $fixture -Operation cancel
    if (-not (Test-Path -LiteralPath $marker)) { throw 'Uninstall did not cancel recovery.' }
    & $script -Directory $fixture -Operation restore
    if (Test-Path -LiteralPath $marker) { throw 'Uninstall rollback did not restore recovery eligibility.' }
    [IO.File]::WriteAllText($marker,'earlier explicit cancellation')
    & $script -Directory $fixture -Operation cancel
    & $script -Directory $fixture -Operation restore
    if ([IO.File]::ReadAllText($marker) -cne 'earlier explicit cancellation') { throw 'Uninstall rollback lost earlier cancellation.' }
    [IO.File]::WriteAllText($rollback,'{"marker_existed":"invalid"}')
    $rejected=$false
    try { & $script -Directory $fixture -Operation restore } catch { $rejected=$true }
    if (-not $rejected -or -not (Test-Path -LiteralPath $marker)) { throw 'Invalid rollback state cleared cancellation.' }
} finally {
    foreach ($file in @($marker,$rollback)) { if (Test-Path -LiteralPath $file) { Remove-Item -LiteralPath $file } }
    Remove-Item -LiteralPath $fixture
}
Write-Output 'Uninstall cancellation file checks passed in an isolated temporary directory.'
