[CmdletBinding()]
param([Parameter(Mandatory)][string]$WorkDirectory)
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
 foreach($file in Get-ChildItem -LiteralPath (Join-Path $trees 'aom') -Recurse -File|Where-Object {$_.Name -eq 'CMakeCache.txt' -or $_.Name -like 'aom_av1_encoder*.vcxproj' -or $_.Name -like 'av1_encoder*.vcxproj'}){
  $target=Join-Path $evidence ($file.FullName.Substring($trees.Length+1))
  New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force|Out-Null
  Copy-Item -LiteralPath $file.FullName -Destination $target
 }
 $runs+=@{label=$label;source_commit=$revision;canonical_source_sha256=$hash;objects=$records;compiled_cache_disabled=$true}
}
@{scope='Two real pinned AV1 dependency builds only; not a full app reproduction or runtime qualification';vcpkg_commit=$baseline;source_commit=$revision;runs=$runs;shipping_bytes_rewritten=$false;production_publication=$false}|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $root 'evidence/report.json') -Encoding UTF8
Write-Output 'Two isolated AV1 source builds completed; original object bytes retained for symbol and code comparison.'
