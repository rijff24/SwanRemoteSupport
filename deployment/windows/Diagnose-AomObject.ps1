[CmdletBinding()]
param([Parameter(Mandatory)][string]$WorkDirectory,[ValidateRange(0,64)][int]$RepeatCompiles=0)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath($WorkDirectory)
if(Test-Path -LiteralPath $root){throw 'Preserve existing diagnostic build state'}
if($env:USE_AOM_391){throw 'Unexpected alternative AV1 source selection'}
$baseline='120deac3062162151622ca4860575a33844ba10b'
$revision='10aece4157eb79315da205f39e19bf6ab3ee30d0'
$sourceHash='e36cdda0a1090348a297082be1e3c328885790244db7683ba1e3f025914933d8'
$vcpkg=Join-Path $env:VCPKG_ROOT 'vcpkg.exe'
if(-not(Test-Path -LiteralPath $vcpkg -PathType Leaf)){throw 'Pinned vcpkg unavailable'}
$actual=(& git -C $env:VCPKG_ROOT rev-parse HEAD|Out-String).Trim()
if($LASTEXITCODE -or $actual -cne $baseline){throw 'vcpkg source differs from application pin'}
$overlay=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../res/vcpkg'))
New-Item -ItemType Directory -Path (Join-Path $root 'evidence') -Force|Out-Null
@{image_version=$env:ImageVersion;processors=@(Get-CimInstance Win32_Processor|Select-Object Name,Manufacturer,Architecture,NumberOfCores,NumberOfLogicalProcessors);os=Get-CimInstance Win32_OperatingSystem|Select-Object Version,BuildNumber}|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $root 'evidence/runner-compiler-context.json') -Encoding UTF8
& (Join-Path $PSScriptRoot 'Record-MsvcToolchain.ps1') -Output (Join-Path $root 'evidence/native-msvc-toolchain.json')
$env:VCPKG_BINARY_SOURCES='clear'
$env:VCPKG_DEFAULT_HOST_TRIPLET='x64-windows-static'
$runs=@()
foreach($label in @('first','second')){
 $directory=Join-Path $root $label
 $manifest=Join-Path $directory 'manifest'
 $install=Join-Path $directory 'installed'
 $trees=Join-Path $directory 'buildtrees'
 $packages=Join-Path $directory 'packages'
 $evidence=Join-Path $root ('evidence/'+$label)
 New-Item -ItemType Directory -Path $manifest,$evidence -Force|Out-Null
 @{name='swan-aom-object-diagnostic';'version-string'='1';dependencies=@('aom');'vcpkg-configuration'=@{
   'default-registry'=@{kind='builtin';baseline=$baseline};'overlay-ports'=@($overlay)
 }}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $manifest 'vcpkg.json') -Encoding UTF8
 & $vcpkg install --triplet x64-windows-static "--x-manifest-root=$manifest" "--x-install-root=$install" "--x-buildtrees-root=$trees" "--x-packages-root=$packages" 2>&1|Tee-Object -FilePath (Join-Path $evidence 'vcpkg-build.log')
 if($LASTEXITCODE){throw ('Pinned AV1 diagnostic source build failed: '+$label)}
 $sources=@(Get-ChildItem -LiteralPath (Join-Path $trees 'aom/src') -Filter encodeframe_utils.c -Recurse -File)
 if($sources.Count -ne 1){throw 'Expected one pinned AV1 source unit'}
 $canonical=[IO.File]::ReadAllText($sources[0].FullName).Replace("`r`n","`n")
 $hasher=[Security.Cryptography.SHA256]::Create()
 try{$hash=([BitConverter]::ToString($hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($canonical)))).Replace('-','').ToLowerInvariant()}finally{$hasher.Dispose()}
 if($hash -cne $sourceHash){throw 'Actual AV1 source unit differs from reviewed upstream commit'}
 Copy-Item -LiteralPath $sources[0].FullName -Destination (Join-Path $evidence 'encodeframe_utils.c')
 $objects=@(Get-ChildItem -LiteralPath (Join-Path $trees 'aom') -Filter '*encodeframe_utils*.obj' -Recurse -File)
 if($objects.Count -lt 1 -or $objects.Count -gt 4){throw 'Expected bounded AV1 object set'}
 $records=@()
 foreach($file in $objects){
  $relative=$file.FullName.Substring($trees.Length+1)
  $target=Join-Path $evidence $relative
  New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force|Out-Null
  Copy-Item -LiteralPath $file.FullName -Destination $target
  $records+=@{path=$relative.Replace('\','/');bytes=$file.Length;sha256=(Get-FileHash -LiteralPath $file.FullName).Hash.ToLowerInvariant()}
 }
 foreach($file in Get-ChildItem -LiteralPath (Join-Path $trees 'aom') -Recurse -File|Where-Object {$_.Name -in @('CMakeCache.txt','build.ninja','rules.ninja') -or $_.Name -like 'aom_av1_encoder*.vcxproj' -or $_.Name -like 'av1_encoder*.vcxproj'}){
  $target=Join-Path $evidence ($file.FullName.Substring($trees.Length+1))
  New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force|Out-Null
  Copy-Item -LiteralPath $file.FullName -Destination $target
 }
 if($label -ceq 'first' -and $RepeatCompiles -gt 0){
  $releaseObjects=@($objects|Where-Object {$_.FullName -match '[\\/]x64-windows-static-rel[\\/]'})
  $releaseCaches=@(Get-ChildItem -LiteralPath (Join-Path $trees 'aom') -Filter CMakeCache.txt -Recurse -File|Where-Object {$_.Directory.Name -ceq 'x64-windows-static-rel'})
  if($releaseObjects.Count -ne 1 -or $releaseCaches.Count -ne 1){throw 'One actual release translation unit and cache required'}
  $release=$releaseCaches[0].Directory.FullName
  $cache=[IO.File]::ReadAllText($releaseCaches[0].FullName)
  $make=[regex]::Match($cache,'(?m)^CMAKE_MAKE_PROGRAM:FILEPATH=(.+)\r?$').Groups[1].Value.Trim()
  if(-not $make -or -not(Test-Path -LiteralPath $make -PathType Leaf) -or [IO.Path]::GetFileName($make) -ine 'ninja.exe'){throw 'Actual pinned Ninja executable unavailable'}
  $target=$releaseObjects[0].FullName.Substring($release.Length+1).Replace('\','/')
  if(-not $target.StartsWith('CMakeFiles/') -or $target.Contains('..')){throw 'Unexpected translation-unit target'}
  $repeatDirectory=Join-Path $evidence 'translation-unit-repeats'
  New-Item -ItemType Directory -Path $repeatDirectory|Out-Null
  & $make -C $release -t commands $target 2>&1|Set-Content -LiteralPath (Join-Path $repeatDirectory 'original-ninja-commands.log') -Encoding UTF8
  if($LASTEXITCODE){throw 'Cannot capture actual translation-unit commands'}
  $sourceFile=$sources[0]
  $originalTime=$sourceFile.LastWriteTimeUtc
  $compiler=[regex]::Match($cache,'(?m)^CMAKE_C_COMPILER:FILEPATH=(.+)\r?$').Groups[1].Value.Trim().Replace('\','/')
  $selected=[regex]::Match($compiler,'^(.*)/VC/Tools/MSVC/(14\.[0-9]+\.[0-9]+)/bin/Hostx64/x64/cl\.exe$')
  if(-not $selected.Success){throw 'Actual cached C compiler is not the selected x64 MSVC toolset'}
  $vcvars=Join-Path $selected.Groups[1].Value 'VC/Auxiliary/Build/vcvarsall.bat'
  if(-not(Test-Path -LiteralPath $vcvars -PathType Leaf) -or $vcvars -match '["&|<>^\r\n]'){throw 'Unexpected compiler environment helper path'}
  $environmentLines=& $env:ComSpec /d /c ('call "'+$vcvars+'" x64 -vcvars_ver='+$selected.Groups[2].Value+' >nul && set INCLUDE && set LIB && set PATH')
  if($LASTEXITCODE){throw 'Unable to obtain the selected native compiler environment'}
  $originalEnvironment=@{}
  $compilerEnvironment=@{}
  foreach($line in $environmentLines){
   if($line -match '^(INCLUDE|LIB|LIBPATH|PATH)=(.*)$'){
    $key=$matches[1].ToUpperInvariant()
    $originalEnvironment[$key]=[Environment]::GetEnvironmentVariable($key,'Process')
    $compilerEnvironment[$key]=$matches[2]
   }
  }
  if(-not $compilerEnvironment.ContainsKey('INCLUDE') -or -not $compilerEnvironment.ContainsKey('PATH')){throw 'Selected compiler includes or executable search path missing'}
  $repeats=@()
  try{
   foreach($key in $compilerEnvironment.Keys){[Environment]::SetEnvironmentVariable($key,$compilerEnvironment[$key],'Process')}
   $compilerEnvironment|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $repeatDirectory 'selected-compiler-environment.json') -Encoding UTF8
   for($index=1;$index -le $RepeatCompiles;$index++){
    $before=(Get-FileHash -LiteralPath $sourceFile.FullName).Hash
    # Force this one real Ninja target to recompile without changing its source.
    $sourceFile.LastWriteTimeUtc=[DateTime]::UtcNow.AddSeconds($index+1)
    $name='compile-'+$index.ToString('D2')
    & $make -C $release -v $target 2>&1|Tee-Object -FilePath (Join-Path $repeatDirectory ($name+'.log'))
    if($LASTEXITCODE){throw 'Exact translation-unit recompilation failed'}
    $log=Get-Content -LiteralPath (Join-Path $repeatDirectory ($name+'.log')) -Raw
    if($log -notmatch 'encodeframe_utils' -or $log -match 'no work to do'){throw 'Expected actual compiler execution did not occur'}
    if((Get-FileHash -LiteralPath $sourceFile.FullName).Hash -cne $before){throw 'Original source bytes changed during repeat'}
    $object=Join-Path $repeatDirectory ($name+'-encodeframe_utils.obj')
    Copy-Item -LiteralPath $releaseObjects[0].FullName -Destination $object
    $repeats+=@{index=$index;object=[IO.Path]::GetFileName($object);sha256=(Get-FileHash -LiteralPath $object).Hash.ToLowerInvariant();original_source_bytes_unchanged=$true;actual_compiler_invocation_observed=$true}
   }
   $dependencyLines=& $make -C $release -t deps $target
   if($LASTEXITCODE){throw 'Cannot inspect actual MSVC dependency records'}
   $dependencyLines|Set-Content -LiteralPath (Join-Path $repeatDirectory 'actual-ninja-dependencies.log') -Encoding UTF8
   $headers=@($dependencyLines|Where-Object {$_ -match '^    (.+)$'}|ForEach-Object {$_.Trim()}|Sort-Object -Unique)
   if($headers.Count -lt 10 -or $headers.Count -gt 3000){throw 'Unexpected actual translation-unit dependency set'}
   $inputRecords=@()
   foreach($path in $headers){
    $resolved=if([IO.Path]::IsPathRooted($path)){$path}else{Join-Path $release $path}
    $item=Get-Item -LiteralPath $resolved
    if($item.PSIsContainer){throw 'Unexpected directory in compiler dependency set'}
    $inputRecords+=@{path=$item.FullName;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $item.FullName).Hash.ToLowerInvariant()}
   }
   @{scope='Actual files recorded by Ninja from MSVC showIncludes; source/header contents are not published';files=$inputRecords}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $repeatDirectory 'actual-compiler-inputs.json') -Encoding UTF8
  }finally{
   $sourceFile.LastWriteTimeUtc=$originalTime
   foreach($key in $originalEnvironment.Keys){[Environment]::SetEnvironmentVariable($key,$originalEnvironment[$key],'Process')}
  }
  @{scope='Repeated compilation of the same actual pinned AV1 translation unit with original Ninja flags; no function or compiler fix applied';count=$repeats.Count;source_commit=$revision;source_sha256=$sourceHash;records=$repeats;shipping_bytes_rewritten=$false}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $repeatDirectory 'repeat-report.json') -Encoding UTF8
 }
 $runs+=@{label=$label;source_commit=$revision;canonical_source_sha256=$hash;objects=$records;compiled_cache_disabled=$true}
}
@{scope='Two real pinned AV1 dependency builds only; not a full app reproduction or runtime qualification';vcpkg_commit=$baseline;source_commit=$revision;runs=$runs;shipping_bytes_rewritten=$false;production_publication=$false}|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $root 'evidence/report.json') -Encoding UTF8
Write-Output 'Two isolated AV1 source builds completed; original object bytes retained for symbol and code comparison.'
