$ErrorActionPreference = 'Stop'
$base = [IO.Path]::GetFullPath($env:TEMP).TrimEnd('\')
$root = [IO.Path]::GetFullPath((Join-Path $base ('swan-pinned-download-'+[Guid]::NewGuid().ToString('N'))))
if (-not $root.StartsWith($base+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Test directory escaped temporary storage.' }
New-Item -ItemType Directory -Path $root | Out-Null
if(Test-Path variable:global:SwanPinnedArchiveTestContext){throw 'Test context already exists'};$global:SwanPinnedArchiveTestContext=@{calls=0;mode='transient';delays=@()}
$global:SwanPinnedArchiveTestContext.reviewed=[Text.Encoding]::UTF8.GetBytes('reviewed complete archive fixture')
$hasher=[Security.Cryptography.SHA256]::Create()
try {$expected=[BitConverter]::ToString($hasher.ComputeHash($global:SwanPinnedArchiveTestContext.reviewed)).Replace('-','').ToLowerInvariant()} finally {$hasher.Dispose()}
function Invoke-WebRequest {
    param($Uri,$OutFile,[switch]$UseBasicParsing,[int]$TimeoutSec)
    $global:SwanPinnedArchiveTestContext.calls++
    if ($global:SwanPinnedArchiveTestContext.mode -eq 'permanent' -or ($global:SwanPinnedArchiveTestContext.mode -eq 'transient' -and $global:SwanPinnedArchiveTestContext.calls -eq 1)) {
        [IO.File]::WriteAllBytes($OutFile,[byte[]](1,2,3));throw 'Simulated connection reset'
    }
    [byte[]]$body=if ($global:SwanPinnedArchiveTestContext.mode -eq 'corrupt') {[byte[]](7,8,9)}else{$global:SwanPinnedArchiveTestContext.reviewed};[IO.File]::WriteAllBytes($OutFile,$body)
}
function Start-Sleep {param([int]$Seconds);$global:SwanPinnedArchiveTestContext.delays+= $Seconds}
$helper=Join-Path $PSScriptRoot 'Get-PinnedArchive.ps1'
try {
    $protocol=[Net.ServicePointManager]::SecurityProtocol
    $path=Join-Path $root 'valid.zip'
    $null=& $helper -Uri https://example.invalid/reviewed.zip -Sha256 $expected -Output $path
    if ($global:SwanPinnedArchiveTestContext.calls -ne 2 -or -not (Test-Path -LiteralPath $path) -or [Net.ServicePointManager]::SecurityProtocol -ne $protocol) { throw 'Retry or protocol restoration failed.' }
    $global:SwanPinnedArchiveTestContext.mode='permanent';$global:SwanPinnedArchiveTestContext.calls=0
    $null=& $helper -Uri https://example.invalid/reviewed.zip -Sha256 $expected -Output $path
    if ($global:SwanPinnedArchiveTestContext.calls -ne 0) { throw 'A verified existing archive was downloaded again.' }
    $failed=$false
    try {$null=& $helper -Uri https://example.invalid/reviewed.zip -Sha256 ('0'*64) -Output $path} catch {$failed=$true}
    if (-not $failed -or $global:SwanPinnedArchiveTestContext.calls -ne 0) { throw 'Mismatched existing evidence was replaced.' }
    $failed=$false;$missing=Join-Path $root 'missing.zip'
    try {$null=& $helper -Uri https://example.invalid/reviewed.zip -Sha256 $expected -Output $missing} catch {$failed=$true}
    if (-not $failed -or $global:SwanPinnedArchiveTestContext.calls -ne 3 -or (Test-Path -LiteralPath $missing)) { throw 'Failed transfer was published or retry bound was ignored.' }
    $global:SwanPinnedArchiveTestContext.mode='corrupt';$global:SwanPinnedArchiveTestContext.calls=0;$failed=$false;$corrupt=Join-Path $root 'corrupt.zip'
    try {$null=& $helper -Uri https://example.invalid/reviewed.zip -Sha256 $expected -Output $corrupt} catch {$failed=$true}
    if (-not $failed -or $global:SwanPinnedArchiveTestContext.calls -ne 1 -or (Test-Path -LiteralPath $corrupt) -or @(Get-ChildItem -LiteralPath $root -Filter '*.unverified').Count -ne 1) { throw 'Corrupt bytes were retried, published or discarded.' }
    $global:SwanPinnedArchiveTestContext.calls=0;$failed=$false
    try {$null=& $helper -Uri http://example.invalid/reviewed.zip -Sha256 $expected -Output (Join-Path $root 'http.zip')} catch {$failed=$true}
    if (-not $failed -or $global:SwanPinnedArchiveTestContext.calls -ne 0) { throw 'Plain HTTP was accepted.' }
    Write-Output 'Pinned download retry, hash rejection, existing evidence, HTTPS and atomic publication checks passed. Network failures were simulated; supplier end-to-end download is separate.'
} finally {
    $resolved=(Get-Item -LiteralPath $root).FullName
    if (-not $resolved.StartsWith($base+'\',[StringComparison]::OrdinalIgnoreCase) -or ((Get-Item -LiteralPath $root).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Refusing unsafe test cleanup.' }
    Remove-Variable -Name SwanPinnedArchiveTestContext -Scope Global
    Remove-Item -LiteralPath $resolved -Recurse -Force
}

