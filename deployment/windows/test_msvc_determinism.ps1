[CmdletBinding()]
param([Parameter(Mandatory)][string]$WorkDirectory)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$root=[IO.Path]::GetFullPath($WorkDirectory)
if(Test-Path -LiteralPath $root){throw 'Preserve existing compiler fixture evidence'}
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs=(& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath|Out-String).Trim()
if($LASTEXITCODE -ne 0 -or -not $vs){throw 'MSVC installation unavailable'}
$version=([IO.File]::ReadAllText((Join-Path $vs 'VC\Auxiliary\Build\Microsoft.VCToolsVersion.default.txt'))).Trim()
if($version -notmatch '^14\.[0-9]+\.[0-9]+$'){throw 'Unexpected MSVC version'}
$vc=Join-Path $vs ('VC\Tools\MSVC\'+$version)
$bin=Join-Path $vc 'bin\Hostx64\x64'
$compiler=Join-Path $bin 'cl.exe'
$sdk=(Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots').KitsRoot10
$sdkVersion=Get-ChildItem -LiteralPath (Join-Path $sdk 'Include') -Directory|Where-Object {$_.Name -match '^10\.0\.[0-9]+\.0$'}|Sort-Object {[version]$_.Name} -Descending|Select-Object -First 1 -ExpandProperty Name
if(-not $sdkVersion){throw 'Windows SDK unavailable'}
$include=@((Join-Path $vc 'include'))+@('ucrt','shared','um','winrt'|ForEach-Object {Join-Path $sdk ('Include\'+$sdkVersion+'\'+$_)})
$libraries=@((Join-Path $vc 'lib\x64'))+@('ucrt','um'|ForEach-Object {Join-Path $sdk ('Lib\'+$sdkVersion+'\'+$_+'\x64')})
foreach($path in @($compiler)+$include+$libraries){if(-not(Test-Path -LiteralPath $path)){throw 'Selected compiler input missing'}}
New-Item -ItemType Directory -Path $root|Out-Null
$source=@'
#include <cstdint>
#include <cstdio>
#include <functional>
#include <typeinfo>
int main() {
  unsigned calls = 0;
  auto consumer = [&calls](const std::uint8_t* data, std::size_t size) {
    calls += (data != nullptr && size != 0) ? 1 : 0;
  };
  std::function<void(const std::uint8_t*, std::size_t)> callback = consumer;
  const std::uint8_t data = 1;
  callback(&data, 1);
  std::printf("%s\n%s\n%u\n", __FILE__, typeid(consumer).name(), calls);
  return calls == 1 ? 0 : 1;
}
'@
function Build-Fixture([string]$label,[string]$folder,[bool]$deterministic,[string]$mode='default') {
 $directory=Join-Path $root $folder
 if(-not(Test-Path -LiteralPath $directory)){New-Item -ItemType Directory -Path $directory|Out-Null}
 $cpp=Join-Path $directory 'fixture.cpp'
 $body=$source
 if($mode -ceq 'named'){
  $body=$body.Replace('int main() {',@'
namespace SwanCompilerFixture {
struct Consumer {
  unsigned* calls;
  void operator()(const std::uint8_t* data, std::size_t size) {
    *calls += (data != nullptr && size != 0) ? 1 : 0;
  }
};
}
int main() {
'@)
  $lambda=@'
  auto consumer = [&calls](const std::uint8_t* data, std::size_t size) {
    calls += (data != nullptr && size != 0) ? 1 : 0;
  };
'@
  if(-not $body.Contains($lambda)){throw 'Named callable fixture preimage differs'}
  $body=$body.Replace($lambda,'  SwanCompilerFixture::Consumer consumer{&calls};')
 }
 if($mode -ceq 'sdk-reply'){
  & python3 (Join-Path $PSScriptRoot 'emit-flutter-reply-fixture.py') --output $cpp
  if($LASTEXITCODE){throw 'Actual SDK reply callable fixture generation failed'}
 }else{[IO.File]::WriteAllText($cpp,$body,[Text.UTF8Encoding]::new($false))}
 $arguments=@('/nologo','/WX','/std:c++17','/O2','/EHsc','/MT','/GR','/DNDEBUG')+@($include|ForEach-Object {'/I'+$_})
 if($deterministic){$arguments+=@('/experimental:deterministic',('/pathmap:'+$directory+'=C:\swan-repro-fixture'))}
 if($mode -ceq 'brepro'){$arguments+='/Brepro'}
 if($mode -ceq 'trim'){$arguments+=('/d1trimfile:'+$directory+'\')}
 $sourceArgument=if($mode -ceq 'relative'){'fixture.cpp'}else{$cpp}
 $arguments+=@(('/Fo'+(Join-Path $directory 'fixture.obj')),('/Fe'+(Join-Path $directory 'fixture.exe')),$sourceArgument,'/link','/Brepro','/INCREMENTAL:NO','/PDBALTPATH:fixture.pdb','/MANIFEST:NO')+@($libraries|ForEach-Object {'/LIBPATH:'+$_})
 if(@($arguments|Where-Object {$_ -ceq $sourceArgument}).Count -ne 1 -or @($arguments|Where-Object {$_ -ceq '/link'}).Count -ne 1){throw 'Compiler argument boundaries invalid'}
 $arguments|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $root ($label+'.arguments.json')) -Encoding utf8
 $start=[Diagnostics.ProcessStartInfo]::new()
 $start.FileName=$compiler
 $start.WorkingDirectory=$directory
 $start.UseShellExecute=$false
 $start.RedirectStandardOutput=$true
 $start.RedirectStandardError=$true
 $start.Environment['PATH']=$bin+';'+$env:PATH
 foreach($argument in $arguments){$start.ArgumentList.Add($argument)}
 $process=[Diagnostics.Process]::Start($start)
 $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
 if(-not $process.WaitForExit(120000)){$process.Kill($true);throw 'Compiler fixture timed out'}
 $log=$stdout.GetAwaiter().GetResult()+$stderr.GetAwaiter().GetResult()
 [IO.File]::WriteAllText((Join-Path $root ($label+'.log')),$log)
 if($process.ExitCode -ne 0){throw ('Real MSVC fixture failed: '+$label)}
 $process.Dispose()
 $exe=Join-Path $directory 'fixture.exe'
 $runStart=[Diagnostics.ProcessStartInfo]::new()
 $runStart.FileName=$exe
 $runStart.UseShellExecute=$false
 $runStart.RedirectStandardOutput=$true
 $runStart.RedirectStandardError=$true
 $run=[Diagnostics.Process]::Start($runStart)
 $runOutput=$run.StandardOutput.ReadToEndAsync();$runError=$run.StandardError.ReadToEndAsync()
 if(-not $run.WaitForExit(30000)){$run.Kill($true);throw 'Compiled callback fixture timed out'}
 $result=@($runOutput.GetAwaiter().GetResult().Split([char[]]"`r`n",[StringSplitOptions]::RemoveEmptyEntries))
 [IO.File]::WriteAllText((Join-Path $root ($label+'.runtime.log')),$runError.GetAwaiter().GetResult())
 if($run.ExitCode -ne 0 -or $result.Count -ne 3 -or $result[2] -cne '1'){throw 'Compiled callback fixture did not run correctly'}
 $run.Dispose()
 $saved=Join-Path $root ($label+'.exe')
 Copy-Item -LiteralPath $exe -Destination $saved
 return @{label=$label;sha256=(Get-FileHash -LiteralPath $saved).Hash.ToLowerInvariant();bytes=(Get-Item -LiteralPath $saved).Length;source_sha256=(Get-FileHash -LiteralPath $cpp).Hash.ToLowerInvariant();embedded_source_path=$result[0];lambda_type=$result[1];callback_passed=$true;compiler_arguments=$arguments}
}
$runs=@(
 (Build-Fixture 'link-only-same-path-first' 'repeat-control' $false),
 (Build-Fixture 'link-only-same-path-second' 'repeat-control' $false),
 (Build-Fixture 'link-only-relocated-first' 'control-a' $false),
 (Build-Fixture 'link-only-relocated-second' 'control-b' $false),
 (Build-Fixture 'deterministic-relocated-first' 'fixed-a' $true),
 (Build-Fixture 'deterministic-relocated-second' 'fixed-b' $true),
 (Build-Fixture 'compiler-brepro-first' 'brepro-a' $true 'brepro'),
 (Build-Fixture 'compiler-brepro-second' 'brepro-b' $true 'brepro'),
 (Build-Fixture 'trimfile-first' 'trim-a' $true 'trim'),
 (Build-Fixture 'trimfile-second' 'trim-b' $true 'trim'),
 (Build-Fixture 'relative-source-first' 'relative-a' $false 'relative'),
 (Build-Fixture 'relative-source-second' 'relative-b' $false 'relative'),
 (Build-Fixture 'named-callable-first' 'named-a' $true 'named'),
 (Build-Fixture 'named-callable-second' 'named-b' $true 'named'),
 (Build-Fixture 'sdk-reply-first' 'sdk-a' $true 'sdk-reply'),
 (Build-Fixture 'sdk-reply-second' 'sdk-b' $true 'sdk-reply')
)
$controlDiffers=$runs[2].sha256 -cne $runs[3].sha256
$fixedMatches=$runs[4].sha256 -ceq $runs[5].sha256
$candidates=@{experimental_and_pathmap=$fixedMatches;compiler_brepro=($runs[6].sha256 -ceq $runs[7].sha256);trimfile=($runs[8].sha256 -ceq $runs[9].sha256);relative_source=($runs[10].sha256 -ceq $runs[11].sha256);named_callable=($runs[12].sha256 -ceq $runs[13].sha256);actual_sdk_reply_callable=($runs[14].sha256 -ceq $runs[15].sha256)}
$report=@{scope='Real standalone MSVC callback/RTTI/path fixture; not the Flutter app or complete native release';compiler_sha256=(Get-FileHash -LiteralPath $compiler).Hash.ToLowerInvariant();toolset=$version;windows_sdk=$sdkVersion;same_path_link_only_matches=($runs[0].sha256 -ceq $runs[1].sha256);relocated_link_only_control_differs=$controlDiffers;relocated_compiler_deterministic_matches=$fixedMatches;candidate_raw_byte_matches=$candidates;shipping_binary_normalization=$false;runs=$runs;full_native_reproduction_required=$true;production_approved=$false}
$report|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $root 'report.json') -Encoding utf8
if(-not $controlDiffers){throw 'Relocation control did not demonstrate the embedded path difference'}
if(-not $candidates.named_callable -or $runs[12].lambda_type -cne $runs[13].lambda_type){throw 'Named callable did not produce identical raw relocated fixture bytes and type identity'}
if(-not $candidates.actual_sdk_reply_callable -or $runs[14].lambda_type -cne $runs[15].lambda_type){throw 'Actual SDK reply callable behavior or raw relocated bytes failed'}
Write-Output 'Real MSVC relocated callback/RTTI fixture passed with original bytes; full native reproduction remains required.'
