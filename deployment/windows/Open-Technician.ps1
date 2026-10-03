[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$env:SWAN_STATE_DIR = Join-Path $env:LOCALAPPDATA 'SwanRemoteSupport-Technician'
$agent = Join-Path $env:SWAN_STATE_DIR 'swan-agent.exe'
if ((Get-AuthenticodeSignature -LiteralPath $agent).Status -ne 'Valid') { throw 'Technician agent requires a valid signature.' }
# Windows PowerShell turns redirected native stderr into terminating errors under
# Stop. Observe the actual process exit so an offline sync can use cached branding.
$syncStart = [Diagnostics.ProcessStartInfo]::new()
$syncStart.FileName = $agent
$syncStart.Arguments = 'sync'
$syncStart.UseShellExecute = $false
$syncStart.CreateNoWindow = $true
$syncProcess = [Diagnostics.Process]::Start($syncStart)
try {
    $syncProcess.WaitForExit()
    $syncExit = $syncProcess.ExitCode
} finally { $syncProcess.Dispose() }
if ($syncExit -ne 0) { Write-Warning 'Company server unavailable. Cached branding can be displayed; authorization still requires valid company policy.' }
$portable = Join-Path $env:SWAN_STATE_DIR 'SwanRemoteSupport-Technician.exe'
if (-not (Test-Path -LiteralPath $portable)) {
    throw 'Technician application is not installed. Run the company installation bundle.'
}
& $agent verify-installed
if ($LASTEXITCODE -ne 0) { throw 'Technician installed identity or pinned publisher verification failed.' }
# Login, inventory and ticket creation take place in the graphical application.
# No password, MFA code, token or grant is passed in process arguments/environment.
& $portable
if ($LASTEXITCODE -ne 0) { throw 'Technician application could not start.' }
