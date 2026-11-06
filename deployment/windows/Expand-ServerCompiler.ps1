[CmdletBinding()]
param([Parameter(Mandatory)][string]$Package,[Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$pin=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'server-compiler.json') -Raw|ConvertFrom-Json
$Package=(Resolve-Path -LiteralPath $Package).Path
if($pin.schema -ne 1 -or $pin.id -cne 'Microsoft.Net.Compilers.Toolset.Framework' -or $pin.version -cne '4.12.0' -or
   (Get-FileHash -LiteralPath $Package -Algorithm SHA256).Hash -ine $pin.sha256){throw 'Pinned server compiler package mismatch.'}
$OutputDirectory=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $OutputDirectory){throw 'Compiler output already exists; preserve it.'}
$archive=[IO.Compression.ZipFile]::OpenRead($Package)
try {
 $seen=@{};$total=0L
 foreach($entry in $archive.Entries){
  $name=$entry.FullName
  if($name -notmatch '^[A-Za-z0-9_\-./+\[\]]+$' -or $name.StartsWith('/') -or @($name.Split('/')|Where-Object{$_ -in @('.','..') -or $_.EndsWith('.')}).Count -or $seen.ContainsKey($name) -or (($entry.ExternalAttributes -shr 16) -band 61440) -eq 40960){throw 'Unsafe compiler archive entry.'}
  $seen[$name]=$true;$total+=$entry.Length
  if($seen.Count -gt 4096 -or $entry.Length -gt 50MB -or $total -gt 150MB){throw 'Compiler archive exceeds limits.'}
 }
 if(-not $seen.ContainsKey($pin.compiler)){throw 'Compiler missing from pinned archive.'}
 [void][IO.Directory]::CreateDirectory($OutputDirectory)
 foreach($entry in $archive.Entries){
  if($entry.FullName.EndsWith('/')){continue}
  $target=[IO.Path]::GetFullPath((Join-Path $OutputDirectory $entry.FullName))
  if(-not $target.StartsWith($OutputDirectory.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Compiler entry escaped output.'}
  [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target))
  [IO.Compression.ZipFileExtensions]::ExtractToFile($entry,$target,$false)
 }
} finally {$archive.Dispose()}
$compiler=Join-Path $OutputDirectory $pin.compiler
if((Get-FileHash -LiteralPath $compiler).Hash -ine $pin.compiler_sha256){throw 'Compiler executable pin mismatch.'}
$signature=Get-AuthenticodeSignature -LiteralPath $compiler
if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation(?:,|$)'){throw 'Microsoft compiler signature invalid.'}
$compiler
