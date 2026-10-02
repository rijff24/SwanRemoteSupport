[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$WixExe,
    [Parameter(Mandatory=$true)][string]$SourceDirectory,
    [Parameter(Mandatory=$true)][string]$AgentExe,
    [Parameter(Mandatory=$true)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [Parameter(Mandatory=$true)][string]$Output,
    [string]$PythonExe='python3',
    [switch]$UnsignedTest
)
$ErrorActionPreference = 'Stop'
$WixExe = (Resolve-Path -LiteralPath $WixExe).Path
$SourceDirectory = (Resolve-Path -LiteralPath $SourceDirectory).Path
$AgentExe = (Resolve-Path -LiteralPath $AgentExe).Path
$Output = [IO.Path]::GetFullPath($Output)
if ($Output -notmatch '-unsigned\.msi$') { throw 'Build output must end in -unsigned.msi; sign and approve it separately.' }
if (Test-Path -LiteralPath $Output) { throw 'Refusing to overwrite an existing installer.' }
$toolSignature = Get-AuthenticodeSignature -LiteralPath $WixExe
if ($toolSignature.Status -ne 'Valid' -or $toolSignature.SignerCertificate.Subject -notmatch 'CN=WiX Toolset \(\.NET Foundation\)') { throw 'WiX tool publisher verification failed.' }
$toolVersion = (& $WixExe --version | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $toolVersion -notmatch '^5\.0\.2(?:\+|$)') { throw 'WiX 5.0.2 is required.' }
$main = Join-Path $SourceDirectory 'rustdesk.exe'
if (-not $UnsignedTest) {
    $mainSignature = Get-AuthenticodeSignature -LiteralPath $main
    $agentSignature = Get-AuthenticodeSignature -LiteralPath $AgentExe
    if ($mainSignature.Status -ne 'Valid' -or $agentSignature.Status -ne 'Valid' -or $mainSignature.SignerCertificate.Thumbprint -ne $agentSignature.SignerCertificate.Thumbprint) { throw 'Production executable inputs require valid signatures from the same release publisher.' }
}
New-Item -ItemType Directory -Force ([IO.Path]::GetDirectoryName($Output)) | Out-Null
$authoring = $Output + '.wxs'
$manifest = $Output + '.installed-files.json'
& $PythonExe (Join-Path $PSScriptRoot 'generate-customer-msi.py') --source $SourceDirectory --agent $AgentExe --version $Version --output $authoring --manifest-output $manifest
if ($LASTEXITCODE -ne 0) { throw 'Customer payload authoring failed.' }
& $WixExe build $authoring -arch x64 -o $Output
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $Output)) { throw 'Customer MSI build failed.' }
[pscustomobject]@{version=$Version;wix=$toolVersion;unsigned_test=[bool]$UnsignedTest;sha256=(Get-FileHash -LiteralPath $Output -Algorithm SHA256).Hash.ToLowerInvariant();installed_manifest_sha256=(Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash.ToLowerInvariant();agent_sha256=(Get-FileHash -LiteralPath $AgentExe -Algorithm SHA256).Hash.ToLowerInvariant()} | ConvertTo-Json | Set-Content -LiteralPath ($Output+'.build.json')
Write-Host 'Built unsigned customer MSI. Signing, approved metadata and clean-machine verification remain required.'
