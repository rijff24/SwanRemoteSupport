[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$InputMsi,
    [Parameter(Mandatory=$true)][string]$OutputMsi,
    [Parameter(Mandatory=$true)][ValidateRange(315532800,4102444799)][long]$SourceDateEpoch
)
$ErrorActionPreference='Stop'
$InputMsi=(Resolve-Path -LiteralPath $InputMsi).Path
$OutputMsi=[IO.Path]::GetFullPath($OutputMsi)
if ($OutputMsi -notmatch '-unsigned\.msi$') { throw 'Output must end in -unsigned.msi; normalize before signing.' }
if ((Get-AuthenticodeSignature -LiteralPath $InputMsi).Status -ne 'NotSigned') { throw 'Only unsigned MSI inputs may be normalized.' }
if (Test-Path -LiteralPath $OutputMsi) { throw 'Refusing to overwrite an installer.' }
$temporary=$OutputMsi+'.partial'
if (Test-Path -LiteralPath $temporary) { throw 'An unfinished output exists; inspect it before retrying.' }
$epoch=[DateTimeOffset]::FromUnixTimeSeconds($SourceDateEpoch).UtcDateTime

function Clear-RootStorageTime([string]$Path) {
    # MS-CFB 2.6.2 permits zero modified time on the root entry. Validate the
    # format and directory location instead of assuming WiX's current offsets.
    $stream=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
    try {
        $header=New-Object byte[] 512
        if ($stream.Read($header,0,512) -ne 512 -or [BitConverter]::ToString($header,0,8) -ne 'D0-CF-11-E0-A1-B1-1A-E1') { throw 'Invalid compound-file header.' }
        $major=[BitConverter]::ToUInt16($header,26)
        $shift=[BitConverter]::ToUInt16($header,30)
        if ([BitConverter]::ToUInt16($header,28) -ne 65534 -or -not (($major -eq 3 -and $shift -eq 9) -or ($major -eq 4 -and $shift -eq 12))) { throw 'Unsupported compound-file format.' }
        $sectorSize=[long]1 -shl $shift
        $directory=[BitConverter]::ToUInt32($header,48)
        $offset=([long]$directory+1)*$sectorSize
        if ($directory -ge 4294967290 -or $offset -lt $sectorSize -or $offset+$sectorSize -gt $stream.Length) { throw 'Invalid root directory sector.' }
        $entry=New-Object byte[] 128
        $stream.Position=$offset
        if ($stream.Read($entry,0,128) -ne 128 -or $entry[66] -ne 5 -or [BitConverter]::ToUInt16($entry,64) -ne 22 -or [Text.Encoding]::Unicode.GetString($entry,0,22) -cne "Root Entry`0") { throw 'Invalid root directory entry.' }
        if ([BitConverter]::ToUInt64($entry,100) -ne 0) { throw 'Unexpected nonzero root creation time.' }
        $stream.Position=$offset+108
        $stream.Write((New-Object byte[] 8),0,8)
        $stream.Flush()
    } finally { $stream.Dispose() }
}

function Set-Summary([string]$Path,[string]$PackageCode) {
    $installer=$null; $summary=$null
    try {
        $installer=New-Object -ComObject WindowsInstaller.Installer
        $summary=$installer.SummaryInformation($Path,3)
        $summary.Property(9)=$PackageCode
        $summary.Property(12)=$epoch
        $summary.Property(13)=$epoch
        [void]$summary.Persist()
    } finally {
        if ($null -ne $summary) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($summary) }
        if ($null -ne $installer) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($installer) }
    }
    Clear-RootStorageTime $Path
}

# Canonicalize variable metadata, then derive a package UUIDv5 from every
# remaining byte. Different payloads, installer tables or epochs get new codes.
# The fixed namespace is a build-format identifier, never a published PackageCode.
[IO.File]::Copy($InputMsi,$temporary,$false)
$namespace=[Guid]'eac92fca-0170-53c1-a207-dfbe3ab9c907'
Set-Summary $temporary ('{'+$namespace.ToString('D').ToUpperInvariant()+'}')
$canonicalHash=(Get-FileHash -LiteralPath $temporary -Algorithm SHA256).Hash.ToLowerInvariant()
$bytes=$namespace.ToByteArray()
foreach($segment in @(@(0,4),@(4,2),@(6,2))){[Array]::Reverse($bytes,$segment[0],$segment[1])}
$name=[Text.Encoding]::UTF8.GetBytes('swan-msi-metadata-v1|'+$canonicalHash)
$sha=[Security.Cryptography.SHA1]::Create()
try{$digest=$sha.ComputeHash([byte[]]($bytes+$name))}finally{$sha.Dispose()}
[byte[]]$result=$digest[0..15]
$result[6]=($result[6] -band 15) -bor 80
$result[8]=($result[8] -band 63) -bor 128
foreach($segment in @(@(0,4),@(4,2),@(6,2))){[Array]::Reverse($result,$segment[0],$segment[1])}
$packageCode='{'+([Guid]::new($result)).ToString('D').ToUpperInvariant()+'}'
Set-Summary $temporary $packageCode
[IO.File]::Move($temporary,$OutputMsi)
[pscustomobject]@{package_code=$packageCode;source_date_epoch=$SourceDateEpoch;canonical_sha256=$canonicalHash;sha256=(Get-FileHash -LiteralPath $OutputMsi -Algorithm SHA256).Hash.ToLowerInvariant()}
