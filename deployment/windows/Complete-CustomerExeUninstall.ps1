$ErrorActionPreference='Stop'
$env:PSModulePath=Join-Path ([Environment]::GetFolderPath('System')) 'WindowsPowerShell/v1.0/Modules'
function Remove-CustomerExeAgent {
    param([Parameter(Mandatory=$true)][string]$Directory)
    $folder=Get-Item -LiteralPath $Directory
    if (-not $folder.PSIsContainer -or ($folder.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe customer state directory.' }
    $marker=Get-Item -LiteralPath (Join-Path $Directory 'pending-uninstall')
    if ($marker.PSIsContainer -or ($marker.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Customer uninstall cancellation is missing or unsafe.' }
    $agent=Join-Path $Directory 'swan-agent.exe'
    if (-not (Test-Path -LiteralPath $agent)) { return }
    $file=Get-Item -LiteralPath $agent
    if ($file.PSIsContainer -or ($file.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe installed configuration agent.' }
    Remove-Item -LiteralPath $agent
}
$directory=Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanRemoteSupport'
if (-not (Test-Path -LiteralPath (Join-Path $directory 'managed-state.json'))) { return }
if (Test-Path -LiteralPath (Join-Path ([Environment]::GetFolderPath('ProgramFiles')) 'Swan Remote Support/Swan Remote Support.exe')) { throw 'Customer application removal is incomplete.' }
if (Get-CimInstance Win32_Service -Filter "Name='Swan Remote Support'") { throw 'Customer service removal is incomplete.' }
if (Get-ScheduledTask 'Swan Company Configuration' -ErrorAction SilentlyContinue) { throw 'Customer configuration task removal is incomplete.' }
$expected=[IO.Path]::GetFullPath((Join-Path $directory 'swan-agent.exe'))
if (Get-CimInstance Win32_Process -Filter "Name='swan-agent.exe'" | Where-Object { $_.ExecutablePath -and [IO.Path]::GetFullPath($_.ExecutablePath) -ieq $expected }) { throw 'Installed configuration agent is still running.' }
Remove-CustomerExeAgent -Directory $directory
