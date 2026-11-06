[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$SourceArchive,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$SourceArchiveSha256,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{40}$')][string]$SourceRevision,
    [Parameter(Mandatory=$true)][string]$SourceDirectory,
    [Parameter(Mandatory=$true)][string]$PayloadManifest,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$PublisherCertificateSha256,
    [Parameter(Mandatory=$true)][string]$CargoExe,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$CargoSha256,
    [Parameter(Mandatory=$true)][string]$RustcExe,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$RustcSha256,
    [Parameter(Mandatory=$true)][string]$GitExe,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$GitSha256,
    [Parameter(Mandatory=$true)][string]$LinkExe,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$LinkSha256,
    [Parameter(Mandatory=$true)][string]$PythonExe,
    [Parameter(Mandatory=$true)][string]$CargoHome,
    [Parameter(Mandatory=$true)][string]$WorkDirectory,
    [Parameter(Mandatory=$true)][string]$Output
)
$ErrorActionPreference='Stop'
function Invoke-BuildTool([string]$Executable,[string[]]$ToolArguments) {
    # Windows PowerShell 5.1 turns native stderr into ErrorRecords. Cargo uses
    # stderr for successful progress too; preserve the output and use its exit
    # code, while keeping PowerShell failures terminating outside this call.
    $previousPreference=$ErrorActionPreference
    $toolExitCode=$null
    $global:LASTEXITCODE=$null
    try {
        $ErrorActionPreference='Continue'
        & $Executable @ToolArguments 2>&1 | ForEach-Object { Write-Host $_.ToString() }
        $toolExitCode=$LASTEXITCODE
    } finally { $ErrorActionPreference=$previousPreference }
    if ($null -eq $toolExitCode -or $toolExitCode -ne 0) {
        throw ('Build tool failed ('+[string]$toolExitCode+'): '+[IO.Path]::GetFileName($Executable))
    }
}
if (-not [Environment]::Is64BitProcess -or $env:OS -cne 'Windows_NT') { throw 'Use a Windows x64 release build environment.' }
$Output=[IO.Path]::GetFullPath($Output)
$WorkDirectory=[IO.Path]::GetFullPath($WorkDirectory)
$CargoHome=[IO.Path]::GetFullPath($CargoHome)
if ($CargoHome -notmatch '^[A-Za-z]:\\' -or $CargoHome.Length -gt 24 -or $CargoHome -eq [IO.Path]::GetPathRoot($CargoHome)) { throw 'Use a dedicated short local Cargo cache, at most24 characters, for locked Git submodule checkouts.' }
if ($Output -notmatch '-unsigned-install\.exe$') { throw 'Build an -unsigned-install.exe; sign and approve it separately.' }
foreach ($path in @($Output,$Output+'.build.json',$Output+'.installed-files.json',$Output+'.payload-verification.json',$WorkDirectory)) {
    if (Test-Path -LiteralPath $path) { throw 'Preserve earlier build state instead of overwriting it.' }
}
$SourceArchive=(Resolve-Path -LiteralPath $SourceArchive).Path
$SourceDirectory=(Resolve-Path -LiteralPath $SourceDirectory).Path
$PayloadManifest=(Resolve-Path -LiteralPath $PayloadManifest).Path
$CargoExe=(Resolve-Path -LiteralPath $CargoExe).Path
$RustcExe=(Resolve-Path -LiteralPath $RustcExe).Path
$GitExe=(Resolve-Path -LiteralPath $GitExe).Path
$LinkExe=(Resolve-Path -LiteralPath $LinkExe).Path
$PythonExe=(Resolve-Path -LiteralPath $PythonExe).Path
if ((Get-FileHash -LiteralPath $CargoExe).Hash -ine $CargoSha256 -or
    (Get-FileHash -LiteralPath $RustcExe).Hash -ine $RustcSha256 -or
    (Get-FileHash -LiteralPath $LinkExe).Hash -ine $LinkSha256 -or
    (Get-FileHash -LiteralPath $GitExe).Hash -ine $GitSha256) { throw 'Pinned compiler/linker/Git binaries differ.' }
if ((& $CargoExe --version | Out-String).Trim() -notmatch '^cargo 1\.90\.0 ' -or $LASTEXITCODE) { throw 'Cargo1.90.0 is required.' }
if ((& $RustcExe --version | Out-String).Trim() -notmatch '^rustc 1\.90\.0 ' -or $LASTEXITCODE) { throw 'Rust1.90.0 is required.' }
$gitVersion=(& $GitExe --version | Out-String).Trim()
if ($LASTEXITCODE -or $gitVersion -notmatch '^git version 2\.') { throw 'Pinned Git2.x is required for locked workspace source dependencies.' }
if ((& $PythonExe --version | Out-String).Trim() -cne 'Python 3.12.10' -or $LASTEXITCODE) { throw 'Python3.12.10 is required.' }
$pythonSignature=Get-AuthenticodeSignature -LiteralPath $PythonExe
if ($pythonSignature.Status -ne 'Valid' -or $pythonSignature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName,$false) -cne 'Python Software Foundation') { throw 'Python tool publisher verification failed.' }
$brotliVersion=(& $PythonExe -c 'import brotli; print(brotli.__version__)' | Out-String).Trim()
if ($LASTEXITCODE -or $brotliVersion -cne '1.1.0') { throw 'Pinned Python brotli1.1.0 is required.' }
$main=Join-Path $SourceDirectory 'rustdesk.exe'
$signature=Get-AuthenticodeSignature -LiteralPath $main
if ($signature.Status -ne 'Valid') { throw 'Sign the inner customer executable before producing its installer.' }
$hash=[Security.Cryptography.SHA256]::Create()
try { $certificateHash=([BitConverter]::ToString($hash.ComputeHash($signature.SignerCertificate.RawData))).Replace('-','').ToLowerInvariant() }
finally { $hash.Dispose() }
if ($certificateHash -cne $PublisherCertificateSha256) { throw 'Inner executable does not match the intended release publisher.' }
$resourceCompiler=(Get-Command rc.exe -CommandType Application -ErrorAction Stop).Source
$resourceSignature=Get-AuthenticodeSignature -LiteralPath $resourceCompiler
if ($resourceSignature.Status -ne 'Valid' -or $resourceSignature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName,$false) -cne 'Microsoft Corporation') { throw 'Use the trusted Microsoft Windows SDK resource compiler.' }
$resourceCompilerHash=(Get-FileHash $resourceCompiler).Hash.ToLowerInvariant()
$sdkRegistry=[Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::LocalMachine,[Microsoft.Win32.RegistryView]::Registry32)
try {
    $sdkRoots=$sdkRegistry.OpenSubKey('SOFTWARE\Microsoft\Windows Kits\Installed Roots')
    try { if ($null -eq $sdkRoots -or $sdkRoots.GetValueNames().Count -eq 0) { throw 'A registered Windows SDK is required; winres does not locate MSVC resources through PATH alone.' } }
    finally { if ($null -ne $sdkRoots) { $sdkRoots.Dispose() } }
} finally { $sdkRegistry.Dispose() }
$saved=@{}
foreach ($name in @('PATH','RUSTC','CARGO_HOME','CARGO_TARGET_DIR','CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER','SOURCE_DATE_EPOCH','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','PYTHONDONTWRITEBYTECODE')) {
    $saved[$name]=[Environment]::GetEnvironmentVariable($name,'Process')
}
try {
    $env:CARGO_HOME=$CargoHome
    Invoke-BuildTool $PythonExe @((Join-Path $PSScriptRoot 'prepare-customer-exe.py'),'--source-archive',$SourceArchive,'--source-sha256',$SourceArchiveSha256,'--source-revision',$SourceRevision,'--payload',$SourceDirectory,'--payload-manifest',$PayloadManifest,'--work-directory',$WorkDirectory)
    $prepared=[IO.File]::ReadAllText((Join-Path $WorkDirectory 'prepared.json'))|ConvertFrom-Json
    $stagedMain=Join-Path $WorkDirectory 'payload/Swan Remote Support.exe'
    $stagedSignature=Get-AuthenticodeSignature -LiteralPath $stagedMain
    if ($stagedSignature.Status -ne 'Valid') { throw 'Staged inner executable is not signed.' }
    $hash=[Security.Cryptography.SHA256]::Create()
    try { $stagedPublisher=([BitConverter]::ToString($hash.ComputeHash($stagedSignature.SignerCertificate.RawData))).Replace('-','').ToLowerInvariant() }
    finally { $hash.Dispose() }
    if ($stagedPublisher -cne $certificateHash -or (Get-FileHash $stagedMain).Hash -ine $prepared.main_sha256) { throw 'Staged inner executable or publisher changed.' }
    $stagedManifest=Join-Path $WorkDirectory 'payload-manifest.json'
    $env:PATH=[IO.Path]::GetDirectoryName($CargoExe)+';'+[IO.Path]::GetDirectoryName($GitExe)+';'+$env:PATH
    if ((Get-Command cargo -CommandType Application).Source -ine $CargoExe) { throw 'Pinned Cargo is not selected.' }
    if ((Get-Command git -CommandType Application).Source -ine $GitExe) { throw 'Pinned Git is not selected.' }
    $env:RUSTC=$RustcExe
    $env:CARGO_TARGET_DIR=Join-Path $WorkDirectory 'target'
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=$LinkExe
    $env:SOURCE_DATE_EPOCH=[string]$prepared.source_date_epoch
    $env:RUSTFLAGS=$null
    $env:CARGO_ENCODED_RUSTFLAGS=@('-Ctarget-feature=+crt-static','-Clink-arg=/Brepro',('--remap-path-prefix='+$WorkDirectory+'=.')) -join [char]31
    $env:PYTHONDONTWRITEBYTECODE='1'
    $packer=Join-Path $WorkDirectory 'source/libs/portable'
    # Execute only the exact corresponding project source selected by the maintainer.
    # Company profiles and the installer worker cannot supply build commands.
    Invoke-BuildTool $PythonExe @((Join-Path $packer 'generate.py'),'-f',(Join-Path $WorkDirectory 'payload'),'-o',$packer,'-e','Swan Remote Support.exe','-t','x86_64-pc-windows-msvc')
    Push-Location $packer
    try {
        Invoke-BuildTool $CargoExe @('test','--locked','--release','--target','x86_64-pc-windows-msvc')
    } finally { Pop-Location }
    $resourceOutputs=@(Get-ChildItem -LiteralPath (Join-Path $env:CARGO_TARGET_DIR 'x86_64-pc-windows-msvc/release/build') -Directory -Filter 'rustdesk-portable-packer-*' | ForEach-Object {
        $path=Join-Path $_.FullName 'output'
        if (Test-Path -LiteralPath $path) { [IO.File]::ReadAllLines($path) | Where-Object { $_ -match "^Selected RC path: '(.*)'$" } }
    })
    if ($resourceOutputs.Count -eq 0) { throw 'Missing actual portable resource-compiler selection evidence.' }
    foreach ($line in $resourceOutputs) {
        $selected=[regex]::Match($line,"^Selected RC path: '(.*)'$").Groups[1].Value
        if ([IO.Path]::GetFullPath($selected) -ine $resourceCompiler -or (Get-FileHash -LiteralPath $selected).Hash -ine $resourceCompilerHash) { throw 'Registered Windows SDK resource compiler differs from the configured trusted tool.' }
    }
    $built=Join-Path $env:CARGO_TARGET_DIR 'x86_64-pc-windows-msvc/release/rustdesk-portable-packer.exe'
    [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Output))
    Invoke-BuildTool $PythonExe @((Join-Path $PSScriptRoot 'verify-portable-payload.py'),'--package',$built,'--manifest',$stagedManifest,'--output',($Output+'.payload-verification.json'))
    [IO.File]::Copy($built,$Output,$false)
    [IO.File]::Copy($stagedManifest,($Output+'.installed-files.json'),$false)
    [ordered]@{
        source_revision=$SourceRevision
        source_archive_sha256=$SourceArchiveSha256
        source_date_epoch=$prepared.source_date_epoch
        sha256=(Get-FileHash $Output).Hash.ToLowerInvariant()
        installed_manifest_sha256=(Get-FileHash ($Output+'.installed-files.json')).Hash.ToLowerInvariant()
        inner_executable_sha256=$prepared.main_sha256
        publisher_certificate_sha256=$certificateHash
        cargo_sha256=$CargoSha256
        cargo_home_path_length=$CargoHome.Length
        rustc_sha256=$RustcSha256
        git_sha256=$GitSha256
        git_version=$gitVersion
        linker_sha256=$LinkSha256
        resource_compiler_sha256=$resourceCompilerHash
        actual_resource_compiler_selection_verified=$true
        producer_script_sha256=(Get-FileHash $PSCommandPath).Hash.ToLowerInvariant()
        source_stager_sha256=(Get-FileHash (Join-Path $PSScriptRoot 'prepare-customer-exe.py')).Hash.ToLowerInvariant()
        payload_verifier_sha256=(Get-FileHash (Join-Path $PSScriptRoot 'verify-portable-payload.py')).Hash.ToLowerInvariant()
        portable_manifest_transform=$prepared.portable_manifest_transform
        signed_inner_payload_verified=$true
        outer_signature_required=$true
        independent_reproduction_verified=$false
        production_publication=$false
    }|ConvertTo-Json -Depth 6|Set-Content -LiteralPath ($Output+'.build.json')
    Write-Host 'Built unsigned outer installer with verified signed inner payload. Outer signing, approved release metadata and clean-machine tests remain required.'
} finally {
    foreach ($name in $saved.Keys) { [Environment]::SetEnvironmentVariable($name,$saved[$name],'Process') }
}
