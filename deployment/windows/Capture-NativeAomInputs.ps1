[CmdletBinding()]
param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$output=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $output){throw 'Preserve original native input capture'}
$root=[IO.Path]::GetFullPath($env:VCPKG_ROOT)
$installed=Join-Path $root 'installed'
if($env:VCPKG_INSTALLED_ROOT -and [IO.Path]::GetFullPath($env:VCPKG_INSTALLED_ROOT) -ine $installed){throw 'Unexpected alternate native-library root'}
$library=Join-Path $installed 'x64-windows-static/lib/aom.lib'
$copyright=Join-Path $installed 'x64-windows-static/share/aom/copyright'
foreach($path in @($library,$copyright)){if(-not(Test-Path -LiteralPath $path -PathType Leaf)){throw 'Expected actual AV1 library or attribution missing'}}
$tree=Join-Path $root 'buildtrees/aom'
$sources=@(Get-ChildItem -LiteralPath (Join-Path $tree 'src') -Filter encodeframe_utils.c -File -Recurse)
if($sources.Count -ne 1){throw 'Expected exactly one freshly built AV1 source unit'}
$source=[IO.File]::ReadAllText($sources[0].FullName).Replace("`r`n","`n")
$hasher=[Security.Cryptography.SHA256]::Create()
try{$sourceHash=([BitConverter]::ToString($hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($source)))).Replace('-','').ToLowerInvariant()}finally{$hasher.Dispose()}
if($sourceHash -cne 'e36cdda0a1090348a297082be1e3c328885790244db7683ba1e3f025914933d8'){throw 'Actual AV1 source differs from reviewed pinned upstream unit'}
$release=Join-Path $tree 'x64-windows-static-rel'
$objects=@(Get-ChildItem -LiteralPath $release -Filter '*encodeframe_utils*.obj' -File -Recurse)
if($objects.Count -ne 1){throw 'Expected one actual release object before endpoint link'}
$cache=Join-Path $release 'CMakeCache.txt'
$cacheText=[IO.File]::ReadAllText($cache)
$ninja=[regex]::Match($cacheText,'(?m)^CMAKE_MAKE_PROGRAM:FILEPATH=(.+)\r?$').Groups[1].Value.Trim()
if(-not(Test-Path -LiteralPath $ninja -PathType Leaf) -or [IO.Path]::GetFileName($ninja) -ine 'ninja.exe'){throw 'Actual AV1 Ninja executable missing'}
$target=$objects[0].FullName.Substring($release.Length+1).Replace('\','/')
if(-not $target.StartsWith('CMakeFiles/') -or $target.Contains('..')){throw 'Unexpected AV1 object target'}
New-Item -ItemType Directory -Path $output|Out-Null
Copy-Item -LiteralPath $library -Destination (Join-Path $output 'aom.lib')
Copy-Item -LiteralPath $copyright -Destination (Join-Path $output 'AOM-COPYRIGHT.txt')
Copy-Item -LiteralPath $sources[0].FullName -Destination (Join-Path $output 'encodeframe_utils.c')
Copy-Item -LiteralPath $objects[0].FullName -Destination (Join-Path $output 'encodeframe_utils.obj')
Copy-Item -LiteralPath $cache -Destination (Join-Path $output 'CMakeCache.txt')
foreach($relative in @('build.ninja','CMakeFiles/rules.ninja')){
 $path=Join-Path $release $relative
 if(-not(Test-Path -LiteralPath $path -PathType Leaf)){throw 'Actual AV1 compiler command file missing'}
 Copy-Item -LiteralPath $path -Destination (Join-Path $output ([IO.Path]::GetFileName($relative)))
}
& $ninja -C $release -t commands $target 2>&1|Set-Content -LiteralPath (Join-Path $output 'original-compile-commands.log') -Encoding UTF8
if($LASTEXITCODE){throw 'Unable to capture actual AV1 compiler command'}
$dependencies=& $ninja -C $release -t deps $target
if($LASTEXITCODE){throw 'Unable to capture actual compiler dependencies'}
$dependencies|Set-Content -LiteralPath (Join-Path $output 'actual-compiler-dependencies.log') -Encoding UTF8
$paths=@($dependencies|Where-Object {$_ -match '^    (.+)$'}|ForEach-Object {$_.Trim()}|Sort-Object -Unique)
if($paths.Count -lt 10 -or $paths.Count -gt 3000){throw 'Unexpected actual AV1 include set'}
$inputs=@()
foreach($path in $paths){
 $resolved=if([IO.Path]::IsPathRooted($path)){$path}else{Join-Path $release $path}
 $item=Get-Item -LiteralPath $resolved
 if($item.PSIsContainer){throw 'Unexpected compiler dependency directory'}
 $inputs+=@{path=$item.FullName;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $item.FullName).Hash.ToLowerInvariant()}
}
$records=@{}
foreach($name in @('aom.lib','encodeframe_utils.obj','encodeframe_utils.c','AOM-COPYRIGHT.txt','CMakeCache.txt','build.ninja','rules.ninja','original-compile-commands.log','actual-compiler-dependencies.log')){
 $item=Get-Item -LiteralPath (Join-Path $output $name)
 $records[$name]=@{bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $item.FullName).Hash.ToLowerInvariant()}
}
if((Get-FileHash -LiteralPath $library).Hash -ine $records['aom.lib'].sha256){throw 'Installed AV1 library changed during capture'}
@{scope='Actual AV1 source, release object and installed static library before endpoint compilation; not a reproducibility claim';source_commit='10aece4157eb79315da205f39e19bf6ab3ee30d0';canonical_source_sha256=$sourceHash;installed_library_path=$library;files=$records;actual_compiler_inputs=$inputs;shipping_bytes_rewritten=$false}|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $output 'capture.json') -Encoding UTF8
Write-Output 'Retained actual AV1 object, static library, compiler commands and include hashes before the endpoint build.'
