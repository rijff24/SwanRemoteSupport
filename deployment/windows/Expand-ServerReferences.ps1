[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$FrameworkPackage,
 [Parameter(Mandatory)][string]$PowerShellPackage,
 [Parameter(Mandatory)][string]$OutputDirectory
)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$pins=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'server-references.json') -Raw|ConvertFrom-Json
if($pins.schema -ne 1){throw 'Unsupported server reference pins.'}
$packages=@(@{path=(Resolve-Path -LiteralPath $FrameworkPackage).Path;pin=$pins.framework},@{path=(Resolve-Path -LiteralPath $PowerShellPackage).Path;pin=$pins.powershell})
# Check both archives before extracting anything; package install scripts never run.
foreach($package in $packages){if((Get-FileHash -LiteralPath $package.path).Hash -ine $package.pin.sha256){throw 'Pinned server reference package mismatch.'}}
$OutputDirectory=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $OutputDirectory){throw 'Reference output already exists; preserve it.'}
[void][IO.Directory]::CreateDirectory($OutputDirectory)
$references=[ordered]@{}
foreach($package in $packages){
 $archive=[IO.Compression.ZipFile]::OpenRead($package.path)
 try {
  foreach($file in $package.pin.reference_files.PSObject.Properties){
   $name=[IO.Path]::GetFileName($file.Name)
   if($name -notin @('mscorlib.dll','System.dll','System.Core.dll','System.Security.dll','System.Management.Automation.dll') -or $references.Contains($name)){throw 'Unexpected or duplicate reference assembly.'}
   $entries=@($archive.Entries|Where-Object{$_.FullName -ceq $file.Name})
   if($entries.Count -ne 1 -or $entries[0].Length -lt 2 -or $entries[0].Length -gt 16MB){throw 'Reference assembly missing or oversized.'}
   # Only fixed basenames are written, never paths supplied by the archive.
   $target=Join-Path $OutputDirectory $name
   [IO.Compression.ZipFileExtensions]::ExtractToFile($entries[0],$target,$false)
   if((Get-FileHash -LiteralPath $target).Hash -ine $file.Value){throw 'Reference assembly hash mismatch.'}
   $references[$name]=$target
  }
 } finally {$archive.Dispose()}
}
if($references.Count -ne 5){throw 'Incomplete server reference set.'}
$references
