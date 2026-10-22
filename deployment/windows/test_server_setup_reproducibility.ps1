[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$ManagementExecutable,
 [Parameter(Mandatory)][string]$ComponentsDirectory,
 [Parameter(Mandatory)][string]$CompilerPackage,
 [Parameter(Mandatory)][string]$WorkDirectory
)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $WorkDirectory){throw 'Use a fresh reproducibility evidence directory.'}
[void][IO.Directory]::CreateDirectory([IO.Path]::GetFullPath($WorkDirectory))
$builder=Join-Path $PSScriptRoot 'Build-ServerSetup.ps1'
$shell=Join-Path $env:WINDIR 'System32/WindowsPowerShell/v1.0/powershell.exe'
$first=Join-Path $WorkDirectory 'first';$second=Join-Path $WorkDirectory 'second'
& $shell -NoProfile -NonInteractive -File $builder -ManagementExecutable $ManagementExecutable -ComponentsDirectory $ComponentsDirectory -CompilerPackage $CompilerPackage -OutputDirectory $first -UnsignedTest | Out-Host
if($LASTEXITCODE -ne 0){throw 'First server setup compilation failed.'}
& $builder -ManagementExecutable $ManagementExecutable -ComponentsDirectory $ComponentsDirectory -CompilerPackage $CompilerPackage -OutputDirectory $second -UnsignedTest | Out-Host
$a=Join-Path $first 'SwanServerSetup-UNSIGNED-TEST.exe';$b=Join-Path $second 'SwanServerSetup-UNSIGNED-TEST.exe'
$hash=(Get-FileHash -LiteralPath $a).Hash.ToLowerInvariant()
if((Get-FileHash -LiteralPath $b).Hash.ToLowerInvariant() -cne $hash){throw 'Actual unsigned server installer bytes differ.'}
$sourceHash=(Get-FileHash -LiteralPath (Join-Path $first 'SOURCE.json')).Hash
if((Get-FileHash -LiteralPath (Join-Path $second 'SOURCE.json')).Hash -cne $sourceHash){throw 'Server source manifests differ.'}
$wrong=Join-Path $WorkDirectory 'wrong-package.nupkg';[IO.File]::WriteAllText($wrong,'Not the pinned Microsoft compiler package.')
$rejected=Join-Path $WorkDirectory 'rejected-compiler';$blocked=$false
try{& (Join-Path $PSScriptRoot 'Expand-ServerCompiler.ps1') -Package $wrong -OutputDirectory $rejected | Out-Null}
catch{if($_.Exception.Message -eq 'Pinned server compiler package mismatch.'){$blocked=$true}else{throw}}
if(-not $blocked -or (Test-Path -LiteralPath $rejected)){throw 'Wrong compiler package was not rejected before extraction.'}
$source=Get-Content -LiteralPath (Join-Path $first 'SOURCE.json') -Raw|ConvertFrom-Json
@{passed=$true;sha256=$hash;source_sha256=$sourceHash.ToLowerInvariant();compiler=$source.compiler;reference_sha256=$source.reference_sha256;setup_source_revision=$source.setup_source_revision;working_tree_dirty=$source.working_tree_dirty;wrong_compiler_rejected=$true;executables_installed=$false;scope='Two actual local unsigned server launcher compilations with the same resources and reference assemblies. No independent CI, deployment, signing, or server acceptance claim.'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $WorkDirectory 'result.json')
Write-Host 'Server installer repeat-build and compiler rejection checks passed.'
