param(
 [Parameter(Mandatory=$true)][string]$BuildRoot,
 [string]$VisualStudioPath='C:\Program Files\Microsoft Visual Studio\2022\Enterprise'
)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$recipe=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'flutter-engine.lock.json') -Raw | ConvertFrom-Json
if($recipe.schema -ne 1 -or $recipe.engine.repository -cne 'https://github.com/flutter-team-archive/engine.git' -or $recipe.depot_tools.repository -cne 'https://chromium.googlesource.com/chromium/tools/depot_tools.git'){throw 'Unexpected engine source recipe'}
if($recipe.buildroot.repository -cne 'https://github.com/flutter/buildroot.git'){throw 'Unexpected Flutter buildroot source'}
foreach($component in @($recipe.engine,$recipe.depot_tools,$recipe.buildroot)){if($component.revision -cnotmatch '^[0-9a-f]{40}$'){throw 'An immutable source revision is required'}}
if($recipe.windows_toolchain.msvc_tools_version -cne '14.38.33130' -or $recipe.windows_toolchain.windows_sdk_version -cne '10.0.22621.0'){throw 'Unexpected engine Windows toolchain'}
$overrideProperties=@($recipe.dependency_overrides.PSObject.Properties)
if($overrideProperties.Count -ne 1 -or $overrideProperties[0].Name -cne 'src/flutter/third_party/tinygltf' -or $overrideProperties[0].Value.repository -cne 'https://github.com/syoyo/tinygltf.git' -or $overrideProperties[0].Value.revision -cne '9bb5806df4055ac973b970ba5b3e27ce27d98148'){throw 'Dependency override must preserve the reviewed original DEPS commit'}
$root=[IO.Path]::GetFullPath($BuildRoot)
if(Test-Path -LiteralPath $root){throw 'Build root must be a new dedicated directory'}
$parent=Get-Item -LiteralPath ([IO.Path]::GetDirectoryName($root))
if($parent.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Build parent cannot be a reparse point'}
if((Get-PSDrive -Name $parent.PSDrive.Name).Free -lt 40GB){throw 'Engine build requires at least 40 GiB free on its build drive'}
$env:DEPOT_TOOLS_UPDATE='0'
$env:DEPOT_TOOLS_WIN_TOOLCHAIN='0'
$env:GIT_CONFIG_COUNT='1'
$env:GIT_CONFIG_KEY_0='core.autocrlf'
$env:GIT_CONFIG_VALUE_0='false'
$env:PYTHONIOENCODING='utf-8'
$env:GYP_MSVS_OVERRIDE_PATH=[IO.Path]::GetFullPath($VisualStudioPath)
$env:WINDOWSSDKDIR='C:\Program Files (x86)\Windows Kits\10'
if(-not(Test-Path -LiteralPath $env:GYP_MSVS_OVERRIDE_PATH)){throw 'Visual Studio 2022 C++ build tools are required'}
$msvc=Join-Path $env:GYP_MSVS_OVERRIDE_PATH ('VC\Tools\MSVC\'+$recipe.windows_toolchain.msvc_tools_version)
if(-not(Test-Path -LiteralPath (Join-Path $msvc 'lib\x64\libcpmt.lib'))){throw 'The pinned Microsoft C++ 14.38.33130 toolset is required'}
if(-not(Test-Path -LiteralPath (Join-Path $msvc 'ATLMFC\lib\x64\atls.lib'))){throw 'The matching Microsoft C++ 14.38 ATL component is required'}
if(-not(Test-Path -LiteralPath (Join-Path $env:WINDOWSSDKDIR ('Include\'+$recipe.windows_toolchain.windows_sdk_version)))){throw 'The pinned Windows SDK 10.0.22621.0 is required'}
New-Item -ItemType Directory -Path $root | Out-Null
function Invoke-Checked([string]$Executable,[string[]]$Arguments){
 & $Executable @Arguments
 if($LASTEXITCODE -ne 0){throw ('Engine build command failed: '+$Executable+' exit '+$LASTEXITCODE)}
}
$depot=Join-Path $root 'depot_tools'
Invoke-Checked git @('init',$depot)
Invoke-Checked git @('-C',$depot,'remote','add','origin',$recipe.depot_tools.repository)
Invoke-Checked git @('-C',$depot,'fetch','--depth=1','origin',$recipe.depot_tools.revision)
Invoke-Checked git @('-C',$depot,'checkout','--detach','FETCH_HEAD')
$actualDepot=(& git -C $depot rev-parse HEAD).Trim()
if($LASTEXITCODE -ne 0 -or $actualDepot -cne $recipe.depot_tools.revision){throw 'Depot-tools checkout mismatch'}
# Disabling automatic source updates also skips the usual Windows bootstrap.
# Initialize the tools from this revision's own pinned CIPD manifest so gclient
# can find its generated git.bat without changing the reviewed depot-tools pin.
Invoke-Checked (Join-Path $depot 'bootstrap\win_tools.bat') @()
if(-not(Test-Path -LiteralPath (Join-Path $depot 'git.bat'))){throw 'Pinned Windows Git bootstrap did not produce git.bat'}
$env:PATH=$depot+';'+$env:PATH
$checkout=Join-Path $root 'checkout'
New-Item -ItemType Directory -Path $checkout | Out-Null
$customDeps=@{}
foreach($entry in $overrideProperties){$customDeps[$entry.Name]=$entry.Value.repository+'@'+$entry.Value.revision}
$customDepsJson=$customDeps | ConvertTo-Json -Compress
$solution='solutions = [{"name": "src/flutter", "url": "'+$recipe.engine.repository+'@'+$recipe.engine.revision+'", "deps_file": "DEPS", "managed": False, "custom_deps": '+$customDepsJson+'}]'
[IO.File]::WriteAllText((Join-Path $checkout '.gclient'),$solution+"`ntarget_os = ['win']`n",[Text.UTF8Encoding]::new($false))
Push-Location $checkout
try {
 Invoke-Checked (Join-Path $depot 'gclient.bat') @('sync','--no-history','--jobs=4')
 $revisions=& (Join-Path $depot 'gclient.bat') revinfo --actual 2>&1
 if($LASTEXITCODE -ne 0){throw 'Could not record actual engine dependency revisions'}
 [IO.File]::WriteAllText((Join-Path $root 'dependency-revisions.txt'),($revisions -join "`n"),[Text.UTF8Encoding]::new($false))
 $engine=Join-Path $checkout 'src\flutter'
 Invoke-Checked python @((Join-Path $PSScriptRoot 'flutter-engine-source.py'),'patch',$engine)
 Invoke-Checked python @((Join-Path $PSScriptRoot 'flutter-engine-source.py'),'patch-buildroot',(Join-Path $checkout 'src'))
 if((Get-PSDrive -Name $parent.PSDrive.Name).Free -lt 12GB){throw 'Insufficient disk reserve after engine dependency checkout'}
 $compiler=Join-Path $engine 'buildtools\windows-x64\clang\bin\clang-cl.exe'
 if(-not(Test-Path -LiteralPath $compiler)){throw 'Pinned engine compiler is missing'}
 $compilerVersion=& $compiler --version | Out-String
 if($LASTEXITCODE -ne 0){throw 'Pinned engine compiler version could not be recorded'}
 $linker=Join-Path $engine 'buildtools\windows-x64\clang\bin\lld-link.exe'
 if($recipe.linker.path -cne 'buildtools/windows-x64/clang/bin/lld-link.exe' -or $recipe.linker.threads -ne 1 -or -not(Test-Path -LiteralPath $linker)){throw 'Pinned experimental LLVM linker is unavailable'}
 $linkerVersion=& $linker --version | Out-String
 if($LASTEXITCODE -ne 0 -or $linkerVersion -notmatch 'LLD'){throw 'Pinned LLVM linker version could not be recorded'}
 $toolchain=@{runner_os=$env:RUNNER_OS;runner_image=$env:ImageOS;runner_image_version=$env:ImageVersion;git=(& git --version);python=(& python --version);compiler=$compilerVersion;compiler_sha256=(Get-FileHash -LiteralPath $compiler).Hash;visual_studio=$env:GYP_MSVS_OVERRIDE_PATH;msvc_tools_version=$recipe.windows_toolchain.msvc_tools_version;msvc_libcmt_sha256=(Get-FileHash -LiteralPath (Join-Path $msvc 'lib\x64\libcmt.lib')).Hash;msvc_libcpmt_sha256=(Get-FileHash -LiteralPath (Join-Path $msvc 'lib\x64\libcpmt.lib')).Hash;msvc_atls_sha256=(Get-FileHash -LiteralPath (Join-Path $msvc 'ATLMFC\lib\x64\atls.lib')).Hash;windows_sdk_version=$recipe.windows_toolchain.windows_sdk_version;msvc_directories=@(Get-ChildItem -Directory -LiteralPath (Join-Path $env:GYP_MSVS_OVERRIDE_PATH 'VC\Tools\MSVC') | Select-Object -ExpandProperty Name);windows_sdk_versions=@(Get-ChildItem -Directory -LiteralPath (Join-Path $env:WINDOWSSDKDIR 'Include') | Select-Object -ExpandProperty Name);gn_arguments=@('--runtime-mode=release');ninja_target='flutter_windows.dll';source_commit=$env:GITHUB_SHA;workflow_run=$env:GITHUB_RUN_ID}
 $toolchain.linker_relative_path=$recipe.linker.path
 $toolchain.linker_sha256=(Get-FileHash -LiteralPath $linker).Hash.ToLowerInvariant()
 $toolchain.linker_version=$linkerVersion
 $toolchain | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'toolchain.json') -Encoding utf8
 Write-Output $compilerVersion
 Push-Location (Join-Path $checkout 'src')
 try {
  Invoke-Checked python @((Join-Path $engine 'tools\gn'),'--runtime-mode=release')
  $commands=& (Join-Path $engine 'third_party\ninja\ninja.exe') -C out/host_release -t commands flutter_windows.dll
  if($LASTEXITCODE -ne 0){throw 'Could not record generated engine compile/link commands'}
  [IO.File]::WriteAllText((Join-Path $root 'ninja-commands.txt'),($commands -join "`n"),[Text.UTF8Encoding]::new($false))
  Invoke-Checked python @((Join-Path $PSScriptRoot 'flutter-engine-source.py'),'verify-commands',(Join-Path $root 'ninja-commands.txt'))
  # Retain response files so the final linker flags are auditable after success.
  try {
   Invoke-Checked (Join-Path $engine 'third_party\ninja\ninja.exe') @('-C','out/host_release','-d','keeprsp','-j4','flutter_windows.dll')
  } finally {
   if(Test-Path -LiteralPath 'out/host_release/flutter_windows.dll.rsp'){
    Copy-Item -LiteralPath 'out/host_release/flutter_windows.dll.rsp' -Destination (Join-Path $root 'link-response.txt')
   }
  }
  Invoke-Checked python @((Join-Path $PSScriptRoot 'flutter-engine-source.py'),'verify-link-response',(Join-Path $root 'link-response.txt'))
 } finally {Pop-Location}
 Invoke-Checked python @((Join-Path $PSScriptRoot 'flutter-engine-source.py'),'package',$root,(Join-Path $root 'artifacts'))
} finally {Pop-Location}
