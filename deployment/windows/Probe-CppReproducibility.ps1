[CmdletBinding()]
param([Parameter(Mandatory)][string]$WorkDirectory)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath($WorkDirectory)
if(Test-Path -LiteralPath $root){throw 'Preserve prior compiler observations; choose a new directory'}
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$installation=(& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath|Out-String).Trim()
if($LASTEXITCODE -ne 0 -or -not $installation){throw 'MSVC x64 installation required'}
$vcvars=Join-Path $installation 'VC\Auxiliary\Build\vcvars64.bat'
if(-not(Test-Path -LiteralPath $vcvars)){throw 'MSVC environment unavailable'}
New-Item -ItemType Directory -Path $root|Out-Null
$source=@'
#include <typeinfo>
#include <memory>
namespace { struct Anonymous { int value; }; }
extern "C" __declspec(dllexport) const char* lambda_name() {
    auto object = std::make_shared<int>(42);
    auto action = [object]() { return *object; };
    return typeid(action).name();
}
extern "C" __declspec(dllexport) const char* anonymous_name() {
    return typeid(Anonymous).name();
}
'@
$records=@()
foreach($variant in @('link-only','compiler-brepro','compiler-deterministic','compiler-brepro-trim')){
 foreach($mode in @('same-path-new-mtime','different-paths')){
  $case=Join-Path $root ($variant+'-'+$mode)
  New-Item -ItemType Directory -Path $case|Out-Null
  $builds=@()
  foreach($iteration in 1,2){
   $working=Join-Path $case $(if($mode -eq 'same-path-new-mtime'){'build'}else{'build-'+$iteration})
   New-Item -ItemType Directory -Path $working -Force|Out-Null
   $sourcePath=Join-Path $working 'probe.cpp'
   [IO.File]::WriteAllText($sourcePath,$source,[Text.UTF8Encoding]::new($false))
   (Get-Item -LiteralPath $sourcePath).LastWriteTimeUtc=[DateTime]::new(2020+$iteration,1,1,0,0,0,[DateTimeKind]::Utc)
   $flags=switch($variant){
    'link-only' {''}
    'compiler-brepro' {'/Brepro'}
    'compiler-deterministic' {'/experimental:deterministic'}
    'compiler-brepro-trim' {'/Brepro /d1trimfile:"'+$working+'\\"'}
   }
   $command=@"
@echo off
call "$vcvars" >nul
if errorlevel 1 exit /b %errorlevel%
cd /d "$working"
cl.exe /LD /MT /O2 /EHsc /std:c++17 /W4 /WX $flags "$sourcePath" /Fo:probe.obj /link /Brepro /INCREMENTAL:NO /PDBALTPATH:probe.pdb /OUT:probe.dll
exit /b %errorlevel%
"@
   $batch=Join-Path $working 'compile.cmd'
   [IO.File]::WriteAllText($batch,$command,[Text.ASCIIEncoding]::new())
   $stdout=Join-Path $case ('compile-'+$iteration+'.stdout')
   $stderr=Join-Path $case ('compile-'+$iteration+'.stderr')
   $process=Start-Process -FilePath $env:ComSpec -ArgumentList @('/d','/c',('"'+$batch+'"')) -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
   # Wait only for cmd, not long-lived compiler helper descendants.
   if(-not $process.WaitForExit(120000)){
    Stop-Process -Id $process.Id -ErrorAction SilentlyContinue
    throw 'Compiler command exceeded two minutes; preserve partial observations'
   }
   $process.WaitForExit()
   $log=[IO.File]::ReadAllText($stdout)+[IO.File]::ReadAllText($stderr)
   $dll=Join-Path $working 'probe.dll'
   $usable=$process.ExitCode -eq 0 -and (Test-Path -LiteralPath $dll) -and $log -notmatch 'D9002'
   $entry=@{iteration=$iteration;compiler_exit=$process.ExitCode;usable=$usable}
   if($usable){
    $retained=Join-Path $case ('original-'+$iteration+'.dll')
    Copy-Item -LiteralPath $dll -Destination $retained
    $bytes=[IO.File]::ReadAllBytes($retained)
    $entry.sha256=(Get-FileHash -LiteralPath $retained).Hash.ToLowerInvariant()
    $entry.lambda_names=@([regex]::Matches([Text.Encoding]::ASCII.GetString($bytes),'<lambda_[0-9a-f]+>')|ForEach-Object Value|Sort-Object -Unique)
   }elseif($variant -eq 'link-only'){throw 'Baseline compiler probe failed; inspect retained logs'}
   $builds+=$entry
  }
  $records+=@{variant=$variant;mode=$mode;builds=$builds;original_dlls_identical=($builds[0].usable -and $builds[1].usable -and $builds[0].sha256 -ceq $builds[1].sha256)}
 }
}
$result=@{scope='Controlled compiler-only lambda/anonymous-namespace observations; no generated DLL executed; not a product rebuild';binaries_executed=$false;shipping_normalized=$false;cases=$records}
$result|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $root 'compiler-observations.json')
$result|ConvertTo-Json -Depth 8 -Compress
