[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Directory,[Parameter(Mandatory=$true)][ValidateSet('cancel','restore')][string]$Operation)
$ErrorActionPreference='Stop'
$marker=Join-Path $Directory 'pending-uninstall'
$rollback=Join-Path $Directory 'uninstall-cancellation-rollback.json'
if ($Operation -eq 'cancel') {
    if (Test-Path -LiteralPath $rollback) { throw 'Previous uninstall cancellation requires explicit recovery.' }
    $bytes=[Text.Encoding]::UTF8.GetBytes((@{marker_existed=(Test-Path -LiteralPath $marker)} | ConvertTo-Json -Compress))
    $temporary=Join-Path $Directory ('uninstall-cancellation-'+[Guid]::NewGuid().ToString('N')+'.tmp')
    try {
        $stream=[IO.File]::Open($temporary,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
        try { $stream.Write($bytes,0,$bytes.Length);$stream.Flush($true) } finally { $stream.Dispose() }
        [IO.File]::Move($temporary,$rollback)
    } finally { if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary } }
    if (-not (Test-Path -LiteralPath $marker)) {
        $stream=[IO.File]::Open($marker,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
        try { $stream.Flush($true) } finally { $stream.Dispose() }
    }
} else {
    if (-not (Test-Path -LiteralPath $rollback)) { return }
    $previous=Get-Content -LiteralPath $rollback -Raw | ConvertFrom-Json
    if ($previous.marker_existed -isnot [bool]) { throw 'Invalid uninstall cancellation rollback state.' }
    if (-not $previous.marker_existed -and (Test-Path -LiteralPath $marker)) { Remove-Item -LiteralPath $marker }
    Remove-Item -LiteralPath $rollback
}
