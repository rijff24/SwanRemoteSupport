[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$WixExe,
    [Parameter(Mandatory=$true)][string]$TechnicianExe,
    [Parameter(Mandatory=$true)][string]$AgentExe,
    [Parameter(Mandatory=$true)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [Parameter(Mandatory=$true)][string]$Output,
    [switch]$UnsignedTest
)
$ErrorActionPreference = 'Stop'
$WixExe = (Resolve-Path -LiteralPath $WixExe).Path
$TechnicianExe = (Resolve-Path -LiteralPath $TechnicianExe).Path
$AgentExe = (Resolve-Path -LiteralPath $AgentExe).Path
$Output = [IO.Path]::GetFullPath($Output)
if ($Output -notmatch '-unsigned\.msi$') { throw 'Build output must end in -unsigned.msi; sign and approve the package separately.' }
if (Test-Path -LiteralPath $Output) { throw 'Refusing to overwrite an existing installer.' }
$toolSignature = Get-AuthenticodeSignature -LiteralPath $WixExe
if ($toolSignature.Status -ne 'Valid' -or $toolSignature.SignerCertificate.Subject -notmatch 'CN=WiX Toolset \(\.NET Foundation\)') { throw 'WiX tool publisher verification failed.' }
$toolVersion = (& $WixExe --version | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $toolVersion -notmatch '^5\.0\.2(?:\+|$)') { throw 'WiX 5.0.2 is required.' }
if (-not $UnsignedTest) {
    $technicianSignature = Get-AuthenticodeSignature -LiteralPath $TechnicianExe
    $agentSignature = Get-AuthenticodeSignature -LiteralPath $AgentExe
    if ($technicianSignature.Status -ne 'Valid' -or $agentSignature.Status -ne 'Valid' -or $technicianSignature.SignerCertificate.Thumbprint -ne $agentSignature.SignerCertificate.Thumbprint) { throw 'Production payloads must have valid signatures from the same release publisher.' }
}
$launcher = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'Open-Technician.ps1'))
$license = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../LICENCE'))
$source = Join-Path $PSScriptRoot 'msi/Technician.wxs'
New-Item -ItemType Directory -Force ([IO.Path]::GetDirectoryName($Output)) | Out-Null
& $WixExe build $source -arch x64 -d ('Version='+$Version) -d ('TechnicianExe='+$TechnicianExe) -d ('AgentExe='+$AgentExe) -d ('Launcher='+$launcher) -d ('License='+$license) -o $Output
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $Output)) { throw 'Technician MSI build failed.' }
[pscustomobject]@{version=$Version;wix=$toolVersion;unsigned_test=[bool]$UnsignedTest;sha256=(Get-FileHash -LiteralPath $Output -Algorithm SHA256).Hash.ToLowerInvariant();technician_sha256=(Get-FileHash -LiteralPath $TechnicianExe -Algorithm SHA256).Hash.ToLowerInvariant();agent_sha256=(Get-FileHash -LiteralPath $AgentExe -Algorithm SHA256).Hash.ToLowerInvariant()} | ConvertTo-Json | Set-Content -LiteralPath ($Output+'.build.json')
Write-Host 'Built unsigned technician MSI. Production distribution requires signing, signed release metadata and clean-machine verification.'
