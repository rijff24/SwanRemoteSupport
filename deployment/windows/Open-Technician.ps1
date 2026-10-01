[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$env:SWAN_STATE_DIR = Join-Path $env:LOCALAPPDATA 'SwanRemoteSupport-Technician'
$agent = Join-Path $PSScriptRoot 'swan-agent.exe'
if ((Get-AuthenticodeSignature -LiteralPath $agent).Status -ne 'Valid') { throw 'Technician agent requires a valid signature.' }
& $agent sync
if ($LASTEXITCODE -ne 0) { Write-Warning 'Company server unavailable. Cached branding can be displayed; authorization still requires valid company policy.' }
$app = Join-Path $PSScriptRoot 'SwanRemoteSupport-install.exe'
$portable = Join-Path $env:SWAN_STATE_DIR 'SwanRemoteSupport-Technician.exe'
if (-not (Test-Path -LiteralPath $portable)) {
    if (-not (Test-Path -LiteralPath $app)) { throw 'Portable technician EXE is required.' }
    & $agent verify-package (Join-Path $PSScriptRoot 'release.json') $app
    if ($LASTEXITCODE -ne 0) { throw 'Technician package verification failed.' }
    # The fixed filename avoids the portable packer opening installation mode.
    Copy-Item -LiteralPath $app -Destination $portable
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'release.json') -Destination (Join-Path $env:SWAN_STATE_DIR 'installed-release.json')
}
& $agent verify-installed
if ($LASTEXITCODE -ne 0) { throw 'Technician installed identity or pinned publisher verification failed.' }
# Login, inventory and ticket creation take place in the graphical application.
# No password, MFA code, token or grant is passed in process arguments/environment.
& $portable
if ($LASTEXITCODE -ne 0) { throw 'Technician application could not start.' }
