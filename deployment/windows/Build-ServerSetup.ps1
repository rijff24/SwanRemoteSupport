[CmdletBinding()]
param([Parameter(Mandatory)][string]$ManagementExecutable,[Parameter(Mandatory)][string]$ComponentsDirectory,
      [Parameter(Mandatory)][string]$OutputDirectory,[switch]$UnsignedTest)
$ErrorActionPreference = 'Stop'
if (-not $UnsignedTest) { throw 'Only explicitly unsigned test server packages are supported until production signing and release verification are approved.' }
$management = (Resolve-Path -LiteralPath $ManagementExecutable).Path
if ([IO.Path]::GetExtension($management) -ine '.exe') { throw 'Select a Windows management executable.' }
$reader = [IO.BinaryReader]::new([IO.File]::OpenRead($management))
try {
    if ($reader.ReadUInt16() -ne 0x5a4d) { throw 'Management executable DOS header missing.' }
    $reader.BaseStream.Position = 60
    $offset = $reader.ReadUInt32()
    if ($offset+6 -gt $reader.BaseStream.Length) { throw 'Management executable PE header missing.' }
    $reader.BaseStream.Position = $offset
    if ($reader.ReadUInt32() -ne 0x4550 -or $reader.ReadUInt16() -ne 0x8664) { throw 'Expected a Windows x64 management executable.' }
} finally {$reader.Dispose()}
$components = (Resolve-Path -LiteralPath $ComponentsDirectory).Path
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $output) { throw 'Server package output directory already exists.' }
$compiler = Join-Path $env:WINDIR 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
if (-not (Test-Path -LiteralPath $compiler -PathType Leaf)) { throw 'The Windows x64 .NET Framework compiler is required.' }
$automation = & (Join-Path $env:WINDIR 'System32/WindowsPowerShell/v1.0/powershell.exe') -NoProfile -NonInteractive -Command '[System.Management.Automation.PSObject].Assembly.Location'
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $automation -PathType Leaf)) { throw 'Windows PowerShell 5 assembly is required.' }
$resources = [ordered]@{
    'Setup-Server.ps1'=(Join-Path $PSScriptRoot 'Setup-Server.ps1')
    'Install-Server.ps1'=(Join-Path $PSScriptRoot 'Install-Server.ps1')
    'swan-management.exe'=$management
    'LICENCE'=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../../LICENCE')).Path
}
foreach ($name in @('hbbs.exe','hbbr.exe','caddy.exe','Caddy-LICENSE.txt','RustDesk-LICENSE.txt','THIRD-PARTY.txt','server-components.json')) {
    $resources['components/'+$name] = Join-Path $components $name
}
$pins=@{'hbbs.exe'='2102e17d32af3ab313a4096d4a4db307984630eccf4f9ed03ef3dfbd4fdb8f83';'hbbr.exe'='5b5fe62f5b5f1fa7df521a243cbfadbeca67bd9df80d5017f23cf62aca59257e';'caddy.exe'='586d4a4cd74bdfd2951b6b81766a32904ffa69c4f1c3d521da3870a2120f1d31'}
$hashes=[ordered]@{}
foreach ($name in $resources.Keys) {
    if ($resources[$name].Contains(',')) { throw 'Compiler resource input paths must not contain commas.' }
    $hashes[$name]=(Get-FileHash -LiteralPath $resources[$name] -Algorithm SHA256).Hash.ToLowerInvariant()
    $basename=[IO.Path]::GetFileName($name)
    if ($pins.ContainsKey($basename) -and $hashes[$name] -ne $pins[$basename]) { throw 'Pinned component executable mismatch.' }
}
$revision = & git -C (Join-Path $PSScriptRoot '../..') rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $revision -notmatch '^[0-9a-f]{40}$') { throw 'Build from the public Git checkout.' }
$dirty = -not [string]::IsNullOrEmpty((& git -C (Join-Path $PSScriptRoot '../..') status --porcelain | Out-String).Trim())
New-Item -ItemType Directory -Path $output | Out-Null
$sourcePath = Join-Path $output 'SOURCE.json'
$sourceFiles=[ordered]@{}
foreach ($name in @('ServerSetupLauncher.cs','Build-ServerSetup.ps1')) { $sourceFiles[$name]=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name) -Algorithm SHA256).Hash.ToLowerInvariant() }
$source = @{schema=1;unsigned_test=$true;repository='https://github.com/rijff24/SwanRemoteSupport';setup_source_revision=$revision;working_tree_dirty=$dirty;resources=$hashes;build_sources=$sourceFiles}
[IO.File]::WriteAllText($sourcePath,($source | ConvertTo-Json -Depth 5),[Text.UTF8Encoding]::new($false))
$resources['SOURCE.json']=$sourcePath
$executable = Join-Path $output 'SwanServerSetup-UNSIGNED-TEST.exe'
$arguments = @('/nologo','/target:winexe','/platform:x64','/optimize+',('/out:'+$executable),('/reference:'+$automation),('/reference:System.Security.dll'))
foreach ($name in $resources.Keys) { $arguments += '/resource:'+$resources[$name]+',Swan.'+$name }
$arguments += Join-Path $PSScriptRoot 'ServerSetupLauncher.cs'
& $compiler @arguments
if ($LASTEXITCODE -ne 0) { throw 'Server setup compilation failed; output remains for inspection.' }
$assembly = [Reflection.Assembly]::LoadFile($executable)
foreach ($name in $hashes.Keys) {
    $stream = $assembly.GetManifestResourceStream('Swan.'+$name)
    if (-not $stream) { throw 'Compiled package resource missing.' }
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {$actual=[BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-','').ToLowerInvariant()} finally {$hasher.Dispose();$stream.Dispose()}
    if ($actual -ne $hashes[$name]) { throw 'Compiled package resource differs from the pinned input.' }
}
Copy-Item -LiteralPath $resources['LICENCE'] -Destination (Join-Path $output 'LICENCE')
Write-Output $executable
