$ErrorActionPreference='Stop'
$env:PSModulePath=Join-Path ([Environment]::GetFolderPath('System')) 'WindowsPowerShell\v1.0\Modules'
$directory=Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanRemoteSupport'
if (-not (Test-Path -LiteralPath (Join-Path $directory 'managed-state.json'))) { return }
$marker=Join-Path $directory 'pending-uninstall'
if (-not (Test-Path -LiteralPath $marker)) {
    $stream=[IO.File]::Open($marker,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
    try { $stream.Flush($true) } finally { $stream.Dispose() }
}
$expected=[IO.Path]::GetFullPath((Join-Path $directory 'swan-agent.exe'))
$tasks=@(Get-ScheduledTask | Where-Object { $_.TaskName -eq 'Swan Company Configuration' })
if ($tasks.Count -gt 1) { throw 'Ambiguous company configuration task.' }
if ($tasks.Count -eq 1) {
    $task=$tasks[0]
    if ($task.Actions.Count -ne 1 -or [IO.Path]::GetFullPath($task.Actions[0].Execute) -ine $expected -or $task.Actions[0].Arguments -cne 'watch' -or $task.Principal.UserId -notin @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM')) { throw 'Configuration task belongs to another installation.' }
    $task | Disable-ScheduledTask | Out-Null
    $task | Stop-ScheduledTask
    $task | Unregister-ScheduledTask -Confirm:$false
}
