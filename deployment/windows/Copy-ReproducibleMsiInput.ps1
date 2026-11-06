[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$InputPath,
    [Parameter(Mandatory=$true)][string]$Destination,
    [Parameter(Mandatory=$true)][ValidateRange(315532800,4102444799)][long]$SourceDateEpoch
)
$ErrorActionPreference='Stop'
$source=Get-Item -LiteralPath $InputPath -Force
$Destination=[IO.Path]::GetFullPath($Destination)
if (Test-Path -LiteralPath $Destination) { throw 'Refusing to overwrite staged input.' }
if ($source.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse points are not valid installer inputs.' }
if ($Destination.StartsWith($source.FullName.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Staging must be outside the source tree.' }
# Cabinet timestamps are local DOS wall-clock values. Give every staged copy
# the same wall-clock representation of the release epoch, independent of the
# builder's time zone. Summary-information timestamps remain UTC.
$timestamp=[DateTime]::SpecifyKind([DateTimeOffset]::FromUnixTimeSeconds($SourceDateEpoch).UtcDateTime,[DateTimeKind]::Unspecified)
function Copy-InputFile([string]$From,[string]$To) {
    $expected=(Get-FileHash -LiteralPath $From -Algorithm SHA256).Hash
    [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($To))
    [IO.File]::Copy($From,$To,$false)
    [IO.File]::SetAttributes($To,[IO.FileAttributes]::Normal)
    [IO.File]::SetLastWriteTime($To,$timestamp)
    if ((Get-FileHash -LiteralPath $To -Algorithm SHA256).Hash -ne $expected -or (Get-FileHash -LiteralPath $From -Algorithm SHA256).Hash -ne $expected) { throw 'Installer input changed during staging.' }
}
if (-not $source.PSIsContainer) {
    Copy-InputFile $source.FullName $Destination
} else {
    # Walk explicitly so junctions cannot silently pull unrelated host files
    # into a package. Inspect every entry, including hidden files/directories.
    $pending=New-Object 'System.Collections.Generic.Stack[string]'
    $pending.Push($source.FullName)
    [void][IO.Directory]::CreateDirectory($Destination)
    while ($pending.Count -gt 0) {
        $directory=$pending.Pop()
        foreach($item in Get-ChildItem -LiteralPath $directory -Force) {
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse points are not valid installer inputs.' }
            $relative=$item.FullName.Substring($source.FullName.TrimEnd('\').Length+1)
            $target=Join-Path $Destination $relative
            if ($item.PSIsContainer) { [void][IO.Directory]::CreateDirectory($target); $pending.Push($item.FullName) }
            else { Copy-InputFile $item.FullName $target }
        }
    }
}
$Destination
