[CmdletBinding()]
param([Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Output){throw 'Preserve existing toolchain identity'}
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$installations=(& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json|Out-String)|ConvertFrom-Json
if($LASTEXITCODE -ne 0 -or @($installations).Count -ne 1){throw 'One selected MSVC installation required'}
$installation=@($installations)[0]
$versionFile=Join-Path $installation.installationPath 'VC\Auxiliary\Build\Microsoft.VCToolsVersion.default.txt'
$version=[IO.File]::ReadAllText($versionFile).Trim()
if($version -notmatch '^14\.[0-9]+\.[0-9]+$'){throw 'Unexpected default MSVC toolset version'}
$root=Join-Path $installation.installationPath ('VC\Tools\MSVC\'+$version)
$records=@()
foreach($directory in @('include','lib\x64','bin\Hostx64\x64')){
 $path=Join-Path $root $directory
 if(-not(Test-Path -LiteralPath $path -PathType Container)){throw 'Required compiler input directory missing'}
 foreach($file in Get-ChildItem -LiteralPath $path -File -Recurse|Sort-Object FullName){
  if(($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Unexpected toolchain reparse file'}
  $relative=$file.FullName.Substring($root.Length+1).Replace('\','/')
  $records+=@{path=$relative;bytes=$file.Length;sha256=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
 }
}
foreach($required in @('bin/Hostx64/x64/cl.exe','bin/Hostx64/x64/c1xx.dll','bin/Hostx64/x64/c2.dll','bin/Hostx64/x64/link.exe','include/typeinfo','include/memory')){
 if(-not @($records|Where-Object {$_.path -ceq $required}).Count){throw ('Required compiler input missing: '+$required)}
}
$canonical=(@($records|Sort-Object path|ForEach-Object {$_.path+"`0"+$_.sha256}) -join "`n")+"`n"
$hasher=[Security.Cryptography.SHA256]::Create()
try{$digest=([BitConverter]::ToString($hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($canonical)))).Replace('-','').ToLowerInvariant()}finally{$hasher.Dispose()}
@{
 scope='Default MSVC x64 compiler, library and header identity; SDK and actual generated compiler commands require separate evidence'
 visual_studio_version=$installation.installationVersion
 toolset_directory_version=$version
 input_identity_sha256=$digest
 files=$records
}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $Output -Encoding UTF8
Write-Output ('Recorded '+$records.Count+' MSVC inputs; identity '+$digest)
