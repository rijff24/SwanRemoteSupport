[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$ArtifactPath,
    [Parameter(Mandatory=$true)][string]$ExpectedSha256,
    [Parameter(Mandatory=$true)][string]$OutputPath,
    [Parameter(Mandatory=$true)][string]$CertificateThumbprint,
    [ValidateSet('CurrentUser','LocalMachine')][string]$CertificateStore='CurrentUser',
    [Parameter(Mandatory=$true)][string]$Publisher,
    [Parameter(Mandatory=$true)][string]$SignToolPath,
    [Parameter(Mandatory=$true)][string]$TimestampUrl
)
$ErrorActionPreference='Stop'
if ($ExpectedSha256 -notmatch '^[0-9a-fA-F]{64}$' -or $CertificateThumbprint -notmatch '^[0-9a-fA-F]{40}$') { throw 'Invalid artifact hash or certificate thumbprint.' }
$source=(Resolve-Path -LiteralPath $ArtifactPath).Path
$extension=[IO.Path]::GetExtension($source).ToLowerInvariant()
if ($extension -notin @('.exe','.msi')) { throw 'Only locally prepared EXE or MSI artifacts can be signed.' }
if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ine $ExpectedSha256) { throw 'Input artifact hash mismatch.' }
$destination=[IO.Path]::GetFullPath($OutputPath)
if ([IO.Path]::GetExtension($destination).ToLowerInvariant() -ne $extension -or (Test-Path -LiteralPath $destination) -or (Test-Path -LiteralPath ($destination+'.signing.json'))) { throw 'Use a new output filename with the same artifact extension.' }
if (-not (Test-Path -LiteralPath ([IO.Path]::GetDirectoryName($destination)) -PathType Container)) { throw 'Output directory must already exist.' }
$timestamp=[Uri]$TimestampUrl
if (-not $timestamp.IsAbsoluteUri -or $timestamp.Scheme -ne 'https' -or -not $timestamp.Host -or $timestamp.UserInfo -or $timestamp.Fragment) { throw 'Use an HTTPS RFC3161 timestamp endpoint without embedded credentials.' }
$tool=(Resolve-Path -LiteralPath $SignToolPath).Path
$toolSignature=Get-AuthenticodeSignature -LiteralPath $tool
if ($toolSignature.Status -ne 'Valid' -or $toolSignature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName,$false) -ne 'Microsoft Corporation') { throw 'SignTool must be a trusted Microsoft-signed Windows SDK executable.' }
$certificate=Get-Item -LiteralPath ('Cert:\'+$CertificateStore+'\My\'+$CertificateThumbprint)
if (-not $certificate.HasPrivateKey -or $certificate.NotBefore.ToUniversalTime() -gt [DateTime]::UtcNow -or $certificate.NotAfter.ToUniversalTime() -le [DateTime]::UtcNow) { throw 'An active company certificate with a local private-key provider is required.' }
if (-not ($certificate.EnhancedKeyUsageList | Where-Object {$_.ObjectId -eq '1.3.6.1.5.5.7.3.3'})) { throw 'Certificate must permit code signing.' }
if ($certificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName,$false) -cne $Publisher) { throw 'Selected certificate does not match the explicit company publisher.' }
$hasher=[Security.Cryptography.SHA256]::Create()
try {$certificateHash=[BitConverter]::ToString($hasher.ComputeHash($certificate.RawData)).Replace('-','')} finally {$hasher.Dispose()}
$staging=$destination+'.'+[Guid]::NewGuid().ToString('N')+'.partial'+$extension
try {
    [IO.File]::Copy($source,$staging,$false)
    if ((Get-FileHash -LiteralPath $staging -Algorithm SHA256).Hash -ine $ExpectedSha256) { throw 'Staged artifact differs from the pinned input.' }
    $arguments=@('sign','/sha1',$CertificateThumbprint,'/fd','SHA256','/tr',$timestamp.AbsoluteUri,'/td','SHA256')
    if ($CertificateStore -eq 'LocalMachine') {$arguments+='/sm'}
    & $tool @arguments $staging
    if ($LASTEXITCODE -ne 0) { throw 'Company signing failed; no output artifact was published.' }
    $null=& (Join-Path $PSScriptRoot 'Verify-Package.ps1') -Path $staging -Publisher $Publisher -CertificateSha256 $certificateHash
    if (-not (Get-AuthenticodeSignature -LiteralPath $staging).TimeStamperCertificate) { throw 'Company signature requires a timestamp.' }
    $hash=(Get-FileHash -LiteralPath $staging -Algorithm SHA256).Hash
    [IO.File]::Move($staging,$destination)
    $record=@{schema=1;input_sha256=$ExpectedSha256.ToLowerInvariant();sha256=$hash.ToLowerInvariant();publisher=$Publisher;publisher_certificate_sha256=$certificateHash.ToLowerInvariant();timestamp_url=$timestamp.AbsoluteUri;artifact=[IO.Path]::GetFileName($destination)}
    [IO.File]::WriteAllText($destination+'.signing.json',($record | ConvertTo-Json))
    Write-Output ('Verified company-signed artifact: '+$destination)
} catch {
    if (Test-Path -LiteralPath $staging) {Remove-Item -LiteralPath $staging -Force}
    throw
}
