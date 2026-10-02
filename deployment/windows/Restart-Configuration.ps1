[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Directory)
$ErrorActionPreference = 'Stop'
$expectedAgent = [IO.Path]::GetFullPath((Join-Path $Directory 'swan-agent.exe'))
$deadline = [DateTime]::UtcNow.AddSeconds(30)
do {
    $tasks = @(Get-ScheduledTask | Where-Object { $_.TaskName -eq 'Swan Company Configuration' })
    if ($tasks.Count -ne 1) { throw 'Expected one company configuration task.' }
    $task = $tasks[0]
    if ($task.Actions.Count -ne 1 -or [IO.Path]::GetFullPath($task.Actions[0].Execute) -ine $expectedAgent -or $task.Actions[0].Arguments -cne 'watch' -or $task.Principal.UserId -notin @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM')) { throw 'Configuration task identity does not match this installation.' }
    if ($task.State -ne 'Running') { break }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Previous configuration task has not exited.' }
    Start-Sleep -Milliseconds 100
} while ($true)
$task | Enable-ScheduledTask | Out-Null
$task | Start-ScheduledTask
