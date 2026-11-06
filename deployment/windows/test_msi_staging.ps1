[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$WorkDirectory)
$ErrorActionPreference='Stop'
if (Test-Path -LiteralPath $WorkDirectory) { throw 'Use a fresh staging test directory.' }
$WorkDirectory=[IO.Path]::GetFullPath($WorkDirectory)
$source=Join-Path $WorkDirectory 'source'
$hidden=Join-Path $source 'hidden'
[void][IO.Directory]::CreateDirectory($hidden)
[IO.File]::WriteAllBytes((Join-Path $source 'binary.dat'),[byte[]](0,255,42,13,10))
[IO.File]::WriteAllText((Join-Path $hidden 'nested.txt'),'nested input')
[IO.File]::SetAttributes($hidden,[IO.FileAttributes]::Hidden)
$originals=@{}
foreach($file in Get-ChildItem -LiteralPath $source -Recurse -Force -File) {
    [IO.File]::SetLastWriteTimeUtc($file.FullName,[datetime]'2020-01-02T03:04:05Z')
    [IO.File]::SetAttributes($file.FullName,[IO.FileAttributes]::ReadOnly)
    $file.Refresh()
    $originals[$file.FullName]=@{hash=(Get-FileHash -LiteralPath $file.FullName).Hash;time=$file.LastWriteTimeUtc.Ticks;attributes=$file.Attributes}
}
$copy=Join-Path $PSScriptRoot 'Copy-ReproducibleMsiInput.ps1'
$destination=& $copy -InputPath $source -Destination (Join-Path $WorkDirectory 'staged') -SourceDateEpoch 1792648409
$expected=[DateTime]::SpecifyKind([DateTimeOffset]::FromUnixTimeSeconds(1792648409).UtcDateTime,[DateTimeKind]::Unspecified)
foreach($name in $originals.Keys) {
    $relative=$name.Substring($source.Length+1)
    $staged=Get-Item -LiteralPath (Join-Path $destination $relative) -Force
    $original=Get-Item -LiteralPath $name -Force
    if ((Get-FileHash -LiteralPath $staged.FullName).Hash -ne $originals[$name].hash -or $staged.LastWriteTime -ne $expected) { throw 'Staged bytes or timestamps differ.' }
    if ((Get-FileHash -LiteralPath $name).Hash -ne $originals[$name].hash -or $original.LastWriteTimeUtc.Ticks -ne $originals[$name].time -or $original.Attributes -ne $originals[$name].attributes) { throw 'Original bytes, timestamps or attributes changed.' }
}
foreach($invalid in @($destination,(Join-Path $source 'child-output'))) {
    $rejected=$false
    try { & $copy -InputPath $source -Destination $invalid -SourceDateEpoch 1792648409 | Out-Null } catch { $rejected=$true }
    if (-not $rejected) { throw 'Unsafe staging destination was accepted.' }
}
# Junction targets are also disposable fixtures; never traverse a host folder.
[void](New-Item -ItemType Junction -Path (Join-Path $source 'junction') -Target $hidden)
$rejected=$false
try { & $copy -InputPath $source -Destination (Join-Path $WorkDirectory 'rejected-link') -SourceDateEpoch 1792648409 | Out-Null }
catch { if ($_.Exception.Message -eq 'Reparse points are not valid installer inputs.') { $rejected=$true } else { throw } }
if (-not $rejected) { throw 'Junction input was accepted.' }
[pscustomobject]@{files=$originals.Count;bytes_preserved=$true;timestamps_staged=$true;originals_unchanged=$true;overwrite_rejected=$true;nested_destination_rejected=$true;junction_rejected=$true} |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $WorkDirectory 'result.json')
Write-Host 'Installer staging checks passed.'
