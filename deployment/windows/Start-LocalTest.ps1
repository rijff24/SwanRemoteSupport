param(
    [string]$EnvironmentFile = (Join-Path $PSScriptRoot 'local-test.env'),
    [string]$ServerExecutable = (Join-Path $PSScriptRoot '../../product/target/debug/swan-management.exe')
)
$ErrorActionPreference = 'Stop'
if (!(Test-Path -LiteralPath $EnvironmentFile)) { throw 'Create local-test.env from local-test.env.example first.' }
# Only named server settings are accepted; the file is data, never PowerShell code.
$allowed = @('SWAN_LISTEN', 'SWAN_DATA_DIR', 'SWAN_RELEASE_PUBLIC_KEY', 'SWAN_TEST_TLS_PORT', 'SWAN_TEST_TLS_PFX', 'SWAN_TEST_TLS_PASSWORD', 'SWAN_TEST_FAULTS', 'SWAN_TEST_CA_FILE')
foreach ($line in Get-Content -LiteralPath $EnvironmentFile) {
    if (!$line.Trim() -or $line.TrimStart().StartsWith('#')) { continue }
    if ($line -notmatch '^([A-Z_]+)=(.*)$' -or $Matches[1] -notin $allowed) { throw 'Invalid local environment setting.' }
    [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
}
if ($env:SWAN_LISTEN -notmatch '^127\.0\.0\.1:\d+$') { throw 'Local tests must bind to loopback.' }
if (!$env:SWAN_DATA_DIR) { throw 'Set a separate local test data directory.' }
$testDataPath = [IO.Path]::GetFullPath($env:SWAN_DATA_DIR)
New-Item -ItemType Directory -Path $testDataPath -Force | Out-Null
$testUserSid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
& icacls.exe $testDataPath /inheritance:r /grant:r ('*' + $testUserSid + ':(OI)(CI)F') '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Failed to protect local test data.' }
$env:SWAN_DATA_DIR = $testDataPath
if (!(Test-Path -LiteralPath $ServerExecutable)) { throw 'Build the product workspace before starting local tests.' }
& $ServerExecutable
if ($LASTEXITCODE -ne 0) { throw "Management server exited with code $LASTEXITCODE" }
