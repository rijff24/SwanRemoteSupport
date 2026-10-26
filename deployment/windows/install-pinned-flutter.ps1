param(
    [Parameter(Mandatory=$true)][string]$Destination,
    [string]$Archive,
    [string]$ExpectedVersion='3.24.5'
)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$version='3.24.5'
$revision='dec2ee5c1f98f8e84a7d5380c05eb8a3d0a81668'
$engine='a18df97ca57a249df5d8d68cd0820600223ce262'
$sha256='b8a7485acd3c6fb23a76b7ac09f89e8d93d62fbff7147c6f5f8c5686d949eeac'
# Pin from the publisher's releases_windows.json; never resolve a moving channel.
$url='https://storage.googleapis.com/flutter_infra_release/releases/stable/windows/flutter_windows_3.24.5-stable.zip'
if($ExpectedVersion -cne $version){throw 'Requested Flutter version differs from the pinned SDK'}
$Destination=[IO.Path]::GetFullPath($Destination)
if(Test-Path -LiteralPath $Destination){throw 'Refuse to overwrite an existing SDK destination'}
if(-not(Test-Path -LiteralPath (Split-Path -Parent $Destination) -PathType Container)){throw 'SDK parent directory is missing'}
if(-not $Archive){
    $Archive=$Destination+'.zip'
    if(Test-Path -LiteralPath $Archive){throw 'Existing SDK download must be inspected before retrying'}
    Invoke-WebRequest -Uri $url -OutFile $Archive -MaximumRetryCount 2 -RetryIntervalSec 2
}
$Archive=[IO.Path]::GetFullPath($Archive)
if((Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash -ine $sha256){throw 'Pinned Flutter archive hash mismatch'}
Expand-Archive -LiteralPath $Archive -DestinationPath $Destination
$sdk=Join-Path $Destination 'flutter'
$actual=(git -C $sdk rev-parse HEAD | Out-String).Trim()
if($LASTEXITCODE -ne 0 -or $actual -cne $revision){throw 'Flutter SDK source revision differs from its pin'}
if(([IO.File]::ReadAllText((Join-Path $sdk 'bin/internal/engine.version'))).Trim() -cne $engine){throw 'Flutter SDK engine revision differs from its pin'}
if(-not(Test-Path -LiteralPath (Join-Path $sdk 'bin/cache/dart-sdk/bin/dart.exe') -PathType Leaf)){throw 'Pinned Dart SDK is missing'}
$receipt=@{version=$version;framework_revision=$actual;engine_revision=$engine;archive_sha256=$sha256;archive_url=$url}
$receipt|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Destination 'sdk-identity.json') -Encoding utf8
if($env:GITHUB_ENV){
    Add-Content -LiteralPath $env:GITHUB_ENV -Value ('FLUTTER_ROOT='+$sdk) -Encoding utf8
    Add-Content -LiteralPath $env:GITHUB_ENV -Value ('PUB_CACHE='+$Destination+'/.pub-cache') -Encoding utf8
}
if($env:GITHUB_PATH){Add-Content -LiteralPath $env:GITHUB_PATH -Value (Join-Path $sdk 'bin') -Encoding utf8}
$receipt|ConvertTo-Json -Compress
