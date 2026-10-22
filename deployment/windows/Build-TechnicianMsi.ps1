[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$WixExe,
    [Parameter(Mandatory=$true)][string]$TechnicianExe,
    [Parameter(Mandatory=$true)][string]$AgentExe,
    [Parameter(Mandatory=$true)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [Parameter(Mandatory=$true)][string]$Output,
    [switch]$UnsignedTest,
    [ValidateRange(0,4102444799)][long]$SourceDateEpoch=0
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
$productCode = & (Join-Path $PSScriptRoot 'Get-MsiProductCode.ps1') -Edition technician -Version $Version
New-Item -ItemType Directory -Force ([IO.Path]::GetDirectoryName($Output)) | Out-Null
$buildOutput=$Output
if ($SourceDateEpoch -ne 0) {
    if ($SourceDateEpoch -lt 315532800) { throw 'SourceDateEpoch must be a release timestamp from 1980 onward.' }
    $buildOutput=$Output+'.raw-unsigned.msi'
    if (Test-Path -LiteralPath $buildOutput) { throw 'Refusing to overwrite an intermediate installer.' }
    $stage=$Output+'.inputs'
    if (Test-Path -LiteralPath $stage) { throw 'Staged inputs already exist; inspect the earlier build.' }
    [void][IO.Directory]::CreateDirectory($stage)
    $copy=Join-Path $PSScriptRoot 'Copy-ReproducibleMsiInput.ps1'
    $TechnicianExe=& $copy -InputPath $TechnicianExe -Destination (Join-Path $stage 'technician.exe') -SourceDateEpoch $SourceDateEpoch
    $AgentExe=& $copy -InputPath $AgentExe -Destination (Join-Path $stage 'swan-agent.exe') -SourceDateEpoch $SourceDateEpoch
    $launcher=& $copy -InputPath $launcher -Destination (Join-Path $stage 'Open-Technician.ps1') -SourceDateEpoch $SourceDateEpoch
    $license=& $copy -InputPath $license -Destination (Join-Path $stage 'LICENSE.txt') -SourceDateEpoch $SourceDateEpoch
}
& $WixExe build $source -arch x64 -d ('Version='+$Version) -d ('ProductCode='+$productCode) -d ('TechnicianExe='+$TechnicianExe) -d ('AgentExe='+$AgentExe) -d ('Launcher='+$launcher) -d ('License='+$license) -o $buildOutput
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $buildOutput)) { throw 'Technician MSI build failed.' }
$metadata=$null
if ($SourceDateEpoch -ne 0) { $metadata=& (Join-Path $PSScriptRoot 'Set-ReproducibleMsiMetadata.ps1') -InputMsi $buildOutput -OutputMsi $Output -SourceDateEpoch $SourceDateEpoch }
[pscustomobject]@{version=$Version;wix=$toolVersion;unsigned_test=[bool]$UnsignedTest;metadata=$metadata;sha256=(Get-FileHash -LiteralPath $Output -Algorithm SHA256).Hash.ToLowerInvariant();technician_sha256=(Get-FileHash -LiteralPath $TechnicianExe -Algorithm SHA256).Hash.ToLowerInvariant();agent_sha256=(Get-FileHash -LiteralPath $AgentExe -Algorithm SHA256).Hash.ToLowerInvariant()} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath ($Output+'.build.json')
Write-Host 'Built unsigned technician MSI. Production distribution requires signing, signed release metadata and clean-machine verification.'
