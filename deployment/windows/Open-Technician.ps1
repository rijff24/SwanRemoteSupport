[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$env:SWAN_STATE_DIR = Join-Path $env:LOCALAPPDATA 'SwanRemoteSupport-Technician'
$agent = Join-Path $PSScriptRoot 'swan-agent.exe'
if ((Get-AuthenticodeSignature -LiteralPath $agent).Status -ne 'Valid') { throw 'Technician agent requires a valid signature.' }
& $agent sync
if ($LASTEXITCODE -ne 0) { throw 'Company synchronization failed.' }
$user = Read-Host 'Technician username'
$secure = Read-Host 'Password' -AsSecureString
$passwordPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
try {
    $env:SWAN_LOGIN_PASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($passwordPointer)
    $env:SWAN_LOGIN_TOTP = Read-Host 'Authenticator code'
    $loginJson = & $agent login $user
    if ($LASTEXITCODE -ne 0) { throw 'Technician authentication failed.' }
    $login = $loginJson | ConvertFrom-Json
    $env:SWAN_TECHNICIAN_TOKEN = $login.token
} finally {
    [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($passwordPointer)
    Remove-Item Env:SWAN_LOGIN_PASSWORD,Env:SWAN_LOGIN_TOTP -ErrorAction SilentlyContinue
}
try {
    & $agent update
    if ($LASTEXITCODE -ne 0) { throw 'Update validation failed or recovery is required.' }
    & $agent devices
    if ($LASTEXITCODE -ne 0) { throw 'Device inventory unavailable.' }
    $device = Read-Host 'Approved device identifier'
    $app = Join-Path $PSScriptRoot 'SwanRemoteSupport-install.exe'
    if (-not (Test-Path -LiteralPath $app)) { throw 'Portable technician EXE is required.' }
    # Avoid the portable packer opening its installer when the filename ends in install.exe.
    $portable = Join-Path $env:SWAN_STATE_DIR 'SwanRemoteSupport-Technician.exe'
    if (-not (Test-Path -LiteralPath $portable)) {
        & $agent verify-package (Join-Path $PSScriptRoot 'release.json') $app
        if ($LASTEXITCODE -ne 0) { throw 'Technician package verification failed.' }
        Copy-Item -LiteralPath $app -Destination $portable
    }
    & $agent connect $device $portable
    if ($LASTEXITCODE -ne 0) { throw 'Support session could not start.' }
} finally { Remove-Item Env:SWAN_TECHNICIAN_TOKEN -ErrorAction SilentlyContinue }
