[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Directory)
$ErrorActionPreference = 'Stop'
$expectedAgent = [IO.Path]::GetFullPath((Join-Path $Directory 'swan-agent.exe'))
$rollbackPath = Join-Path $Directory 'configuration-task-rollback.json'
if (Test-Path -LiteralPath $rollbackPath) { Remove-Item -LiteralPath $rollbackPath }
$tasks = @(Get-ScheduledTask | Where-Object { $_.TaskName -eq 'Swan Company Configuration' })
if ($tasks.Count -eq 0) { return }
if ($tasks.Count -ne 1) { throw 'Ambiguous company configuration task.' }
$task = $tasks[0]
if ($task.Actions.Count -ne 1 -or [IO.Path]::GetFullPath($task.Actions[0].Execute) -ine $expectedAgent -or $task.Actions[0].Arguments -cne 'watch' -or $task.Principal.UserId -notin @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM')) {
    throw 'Configuration task does not belong to this installation.'
}
@{enabled=($task.State -ne 'Disabled');running=($task.State -eq 'Running')} | ConvertTo-Json | Set-Content -LiteralPath $rollbackPath -Encoding UTF8
$task | Disable-ScheduledTask | Out-Null
$task | Stop-ScheduledTask
$task | Unregister-ScheduledTask -Confirm:$false
# Preserve company identity and consent data for explicit repair/reinstallation.
# MSI removes its owned agent binary after this action exits.
