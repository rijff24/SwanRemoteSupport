[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$ManagementExecutable,
 [Parameter(Mandatory)][string]$ComponentsDirectory,
 [Parameter(Mandatory)][string]$CompilerPackage,
 [Parameter(Mandatory)][string]$FrameworkReferencesPackage,
 [Parameter(Mandatory)][string]$PowerShellReferencesPackage,
 [Parameter(Mandatory)][string]$WorkDirectory
)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $WorkDirectory){throw 'Use a fresh reproducibility evidence directory.'}
[void][IO.Directory]::CreateDirectory([IO.Path]::GetFullPath($WorkDirectory))
$builder=Join-Path $PSScriptRoot 'Build-ServerSetup.ps1'
$shell=Join-Path $env:WINDIR 'System32/WindowsPowerShell/v1.0/powershell.exe'
$first=Join-Path $WorkDirectory 'first';$second=Join-Path $WorkDirectory 'second'
& $shell -NoProfile -NonInteractive -File $builder -ManagementExecutable $ManagementExecutable -ComponentsDirectory $ComponentsDirectory -CompilerPackage $CompilerPackage -FrameworkReferencesPackage $FrameworkReferencesPackage -PowerShellReferencesPackage $PowerShellReferencesPackage -OutputDirectory $first -UnsignedTest | Out-Host
if($LASTEXITCODE -ne 0){throw 'First server setup compilation failed.'}
& $builder -ManagementExecutable $ManagementExecutable -ComponentsDirectory $ComponentsDirectory -CompilerPackage $CompilerPackage -FrameworkReferencesPackage $FrameworkReferencesPackage -PowerShellReferencesPackage $PowerShellReferencesPackage -OutputDirectory $second -UnsignedTest | Out-Host
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
$referencePins=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'server-references.json') -Raw|ConvertFrom-Json
foreach($kind in @('framework','powershell')){
 foreach($file in $referencePins.$kind.reference_files.PSObject.Properties){
  $name=[IO.Path]::GetFileName($file.Name)
  if($source.reference_sha256.$name -cne $file.Value){throw 'Build used a reference outside the reviewed pins.'}
 }
 $rejectedReference=Join-Path $WorkDirectory ('rejected-'+$kind);$blocked=$false
 $framework=if($kind -eq 'framework'){$wrong}else{$FrameworkReferencesPackage}
 $powershell=if($kind -eq 'powershell'){$wrong}else{$PowerShellReferencesPackage}
 try{& (Join-Path $PSScriptRoot 'Expand-ServerReferences.ps1') -FrameworkPackage $framework -PowerShellPackage $powershell -OutputDirectory $rejectedReference|Out-Null}
 catch{if($_.Exception.Message -eq 'Pinned server reference package mismatch.'){$blocked=$true}else{throw}}
 if(-not $blocked -or (Test-Path -LiteralPath $rejectedReference)){throw 'Wrong reference package was not rejected before extraction.'}
}
$preview=[IO.Path]::GetFullPath((Join-Path $WorkDirectory 'packaged-preview.png'))
$start=[Diagnostics.ProcessStartInfo]::new()
$start.FileName=[IO.Path]::GetFullPath($a)
$start.Arguments='--render-preview "'+$preview+'"'
$start.UseShellExecute=$false
$start.CreateNoWindow=$true
$start.WindowStyle=[Diagnostics.ProcessWindowStyle]::Hidden
# Own the process handle directly so Windows PowerShell retains ExitCode even
# when a short-lived child has already exited by the next observation.
$process=[Diagnostics.Process]::Start($start)
@{pid=$process.Id;executable_sha256=$hash;mode='preview-only'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $WorkDirectory 'preview-process.json')
if(-not $process.WaitForExit(60000)){throw 'Packaged preview is still running; inspect its recorded PID before retrying.'}
if($process.ExitCode -ne 0 -or -not(Test-Path -LiteralPath $preview)){throw 'Packaged preview runtime failed.'}
$png=[IO.File]::ReadAllBytes($preview)
if($png.Length -lt 24 -or [BitConverter]::ToString($png,0,8) -cne '89-50-4E-47-0D-0A-1A-0A'){throw 'Packaged preview did not produce a PNG.'}
@{passed=$true;sha256=$hash;source_sha256=$sourceHash.ToLowerInvariant();compiler=$source.compiler;reference_packages=$source.reference_packages;reference_sha256=$source.reference_sha256;setup_source_revision=$source.setup_source_revision;working_tree_dirty=$source.working_tree_dirty;wrong_compiler_rejected=$true;wrong_references_rejected=$true;packaged_preview_passed=$true;executables_installed=$false;scope='Two actual local unsigned server launcher compilations with pinned compiler/reference packages and preview-only runtime. No independent CI, deployment, signing, or complete server acceptance claim.'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $WorkDirectory 'result.json')
Write-Host 'Server installer repeat-build and compiler rejection checks passed.'
