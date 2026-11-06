[CmdletBinding()]
param([Parameter(Mandatory)][string]$Path,[Parameter(Mandatory)][string]$Publisher,[Parameter(Mandatory)][string]$CertificateSha256,[string]$Sha256)
$ErrorActionPreference = 'Stop'
$resolved = (Resolve-Path -LiteralPath $Path).Path
if ($Sha256 -and (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash -ne $Sha256) { throw 'Package hash does not match signed release metadata.' }
$signature = Get-AuthenticodeSignature -LiteralPath $resolved
if ($signature.Status -ne 'Valid' -or -not $signature.SignerCertificate) { throw 'Package requires a valid trusted Authenticode signature.' }
$actualPublisher = $signature.SignerCertificate.GetNameInfo([System.Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false)
if ($actualPublisher -cne $Publisher) { throw 'Package publisher does not match the approved release publisher.' }
$hasher = [Security.Cryptography.SHA256]::Create()
try { $certificateHash = [BitConverter]::ToString($hasher.ComputeHash($signature.SignerCertificate.RawData)).Replace('-', '') } finally { $hasher.Dispose() }
if ($certificateHash -ine $CertificateSha256) { throw 'Signing certificate does not match the approved release certificate.' }
Write-Output 'Package signature and publisher verified.'
