[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Directory)
$ErrorActionPreference = 'Stop'
$expectedAgent = [IO.Path]::GetFullPath((Join-Path $Directory 'swan-agent.exe'))
$rollbackPath = Join-Path $Directory 'configuration-task-rollback.json'
if (-not (Test-Path -LiteralPath $rollbackPath)) { return }
$previous = Get-Content -LiteralPath $rollbackPath -Raw | ConvertFrom-Json
if ($previous.enabled -isnot [bool] -or $previous.running -isnot [bool]) { throw 'Invalid configuration task rollback state.' }
$tasks = @(Get-ScheduledTask | Where-Object { $_.TaskName -eq 'Swan Company Configuration' })
if ($tasks.Count -gt 1) { throw 'Ambiguous company configuration task.' }
if ($tasks.Count -eq 1) {
    $task = $tasks[0]
    if ($task.Actions.Count -ne 1 -or [IO.Path]::GetFullPath($task.Actions[0].Execute) -ine $expectedAgent -or $task.Actions[0].Arguments -cne 'watch' -or $task.Principal.UserId -notin @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM')) { throw 'Configuration task does not belong to this installation.' }
} else {
if ((Get-AuthenticodeSignature -LiteralPath $expectedAgent).Status -ne 'Valid') { throw 'Cannot restore the configuration task with an untrusted agent.' }
$action = New-ScheduledTaskAction -Execute $expectedAgent -Argument 'watch'
$trigger = New-ScheduledTaskTrigger -AtStartup
$settings = New-ScheduledTaskSettingsSet -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero)
Register-ScheduledTask -TaskName 'Swan Company Configuration' -Action $action -Trigger $trigger -Settings $settings -User 'SYSTEM' -RunLevel Highest | Out-Null
}
if ($previous.enabled) { Enable-ScheduledTask -TaskName 'Swan Company Configuration' | Out-Null } else { Disable-ScheduledTask -TaskName 'Swan Company Configuration' | Out-Null }
if ($previous.running) { Start-ScheduledTask -TaskName 'Swan Company Configuration' }
Remove-Item -LiteralPath $rollbackPath
