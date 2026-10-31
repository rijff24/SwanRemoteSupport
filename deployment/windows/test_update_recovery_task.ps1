$ErrorActionPreference='Stop'
# Validate ownership first, then exercise one isolated real task lifecycle.
$tokens=$null; $errors=$null
$ast=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'Update-RecoveryTask.ps1'),[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Recovery task script failed parsing.' }
$guard=$ast.Find({param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Assert-RecoveryTask'},$true)
if ($null -eq $guard) { throw 'Recovery task ownership guard missing.' }
. ([ScriptBlock]::Create($guard.Extent.Text))
$helper='C:\IsolatedFixture\updates\2\swan-agent.exe'
$action=@{Execute=$helper;Arguments='recover-update-task'}
$valid=@{TaskPath='\';Actions=@($action);Principal=@{UserId='S-1-5-18'}}
Assert-RecoveryTask $valid $helper 'S-1-5-18'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
foreach ($account in @($identity.User.Value,$identity.Name,($identity.Name -split '\\')[-1])) {
    $ownTask=$valid.Clone();$ownTask.Principal=@{UserId=$account}
    Assert-RecoveryTask $ownTask $helper $identity.User.Value
}
foreach ($account in @('SYSTEM','NT AUTHORITY\SYSTEM')) {
    $systemTask=$valid.Clone();$systemTask.Principal=@{UserId=$account}
    Assert-RecoveryTask $systemTask $helper 'S-1-5-18'
}
foreach ($change in @(
    @{TaskPath='\OtherProduct\'},
    @{Actions=@(@{Execute='C:\OtherInstallation\swan-agent.exe';Arguments='recover-update-task'})},
    @{Actions=@(@{Execute=$helper;Arguments='watch'})},
    @{Actions=@($action,$action)},
    @{Principal=@{UserId='OtherAccount'}},
    @{Principal=@{UserId='S-1-5-19'}},
    @{Principal=@{UserId=$identity.Name}}
)) {
    $record=$valid.Clone()
    foreach ($key in $change.Keys) { $record[$key]=$change[$key] }
    $rejected=$false
    try { Assert-RecoveryTask $record $helper 'S-1-5-18' } catch { $rejected=$true }
    if (-not $rejected) { throw 'Recovery task ownership mismatch was accepted.' }
}
Write-Output 'Recovery task account aliases resolve to the exact SID; foreign identities and actions rejected.'
$taskFixture=Join-Path ([IO.Path]::GetTempPath()) ('swan-recovery-principal-'+[Guid]::NewGuid().ToString('N'))
$updates=Join-Path $taskFixture 'updates';$staged=Join-Path $updates '42';$stagedHelper=Join-Path $staged 'swan-agent.exe'
$taskName=$null
try {
    $null=New-Item -ItemType Directory -Path $staged
    # A signed OS binary is only a registration fixture; never start this task.
    Copy-Item -LiteralPath (Join-Path $env:SystemRoot 'System32\whoami.exe') -Destination $stagedHelper
    $script=Join-Path $PSScriptRoot 'Update-RecoveryTask.ps1'
    $inspection=& $script -Operation register -Directory $taskFixture -Edition technician -Sequence 42 -InspectOnly | ConvertFrom-Json
    $taskName=$inspection.task_name
    & $script -Operation register -Directory $taskFixture -Edition technician -Sequence 42
    $registered=Get-ScheduledTask -TaskName $taskName
    Assert-RecoveryTask $registered $stagedHelper $identity.User.Value
    $inspection=& $script -Operation remove -Directory $taskFixture -Edition technician -Sequence 42 -InspectOnly | ConvertFrom-Json
    if ($inspection.task_mutated -or -not (Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue)) { throw 'Read-only inspection changed the task.' }
    & $script -Operation remove -Directory $taskFixture -Edition technician -Sequence 42
    if (Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue) { throw 'Owned task was not removed after principal normalization.' }
} finally {
    if ($taskName) {
        $remaining=Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
        if ($remaining) { Assert-RecoveryTask $remaining $stagedHelper $identity.User.Value; $remaining | Unregister-ScheduledTask -Confirm:$false }
    }
    if (Test-Path -LiteralPath $stagedHelper) { Remove-Item -LiteralPath $stagedHelper }
    foreach ($directory in @($staged,$updates,$taskFixture)) { if (Test-Path -LiteralPath $directory) { Remove-Item -LiteralPath $directory } }
}
Write-Output 'Real technician recovery task registration, read-only inspection and removal passed; helper never executed.'
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
