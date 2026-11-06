[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$FirstMsi,
    [Parameter(Mandatory=$true)][string]$SecondMsi,
    [Parameter(Mandatory=$true)][string]$WorkDirectory,
    [Parameter(Mandatory=$true)][long]$SourceDateEpoch,
    [string]$SignedMsi
)
$ErrorActionPreference='Stop'
$FirstMsi=(Resolve-Path -LiteralPath $FirstMsi).Path
$SecondMsi=(Resolve-Path -LiteralPath $SecondMsi).Path
if (Test-Path -LiteralPath $WorkDirectory) { throw 'Use a fresh test directory to preserve earlier evidence.' }
[void](New-Item -ItemType Directory -Path $WorkDirectory)
$before=@((Get-FileHash -LiteralPath $FirstMsi).Hash,(Get-FileHash -LiteralPath $SecondMsi).Hash)
$normalizer=Join-Path $PSScriptRoot 'Set-ReproducibleMsiMetadata.ps1'
$first=Join-Path $WorkDirectory 'first-unsigned.msi'
$second=Join-Path $WorkDirectory 'second-unsigned.msi'
$repeat=Join-Path $WorkDirectory 'repeat-unsigned.msi'
$different=Join-Path $WorkDirectory 'different-unsigned.msi'
$a=& $normalizer -InputMsi $FirstMsi -OutputMsi $first -SourceDateEpoch $SourceDateEpoch
$b=& $normalizer -InputMsi $SecondMsi -OutputMsi $second -SourceDateEpoch $SourceDateEpoch
if ($a.sha256 -ne $b.sha256 -or $a.package_code -ne $b.package_code) { throw 'Independent unsigned MSI bytes still differ after metadata preparation.' }
$c=& $normalizer -InputMsi $first -OutputMsi $repeat -SourceDateEpoch $SourceDateEpoch
if ($c.sha256 -ne $a.sha256) { throw 'Metadata preparation is not idempotent.' }
$d=& $normalizer -InputMsi $FirstMsi -OutputMsi $different -SourceDateEpoch ($SourceDateEpoch+2)
if ($d.sha256 -eq $a.sha256 -or $d.package_code -eq $a.package_code) { throw 'Changed metadata reused the package identity.' }
$changedInput=Join-Path $WorkDirectory 'changed-input.msi'
[IO.File]::Copy($FirstMsi,$changedInput,$false)
$installer=$null; $database=$null; $view=$null
try {
    $installer=New-Object -ComObject WindowsInstaller.Installer
    $database=$installer.OpenDatabase($changedInput,1)
    $view=$database.OpenView("INSERT INTO ``Property`` (``Property``, ``Value``) VALUES ('SWAN_METADATA_TEST', 'changed')")
    [void]$view.Execute()
    [void]$database.Commit()
} finally {
    foreach($com in @($view,$database,$installer)) { if ($null -ne $com) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($com) } }
}
$changed=& $normalizer -InputMsi $changedInput -OutputMsi (Join-Path $WorkDirectory 'changed-unsigned.msi') -SourceDateEpoch $SourceDateEpoch
if ($changed.sha256 -eq $a.sha256 -or $changed.package_code -eq $a.package_code) { throw 'Changed installer tables reused the package identity.' }
$rejected=$false
try { & $normalizer -InputMsi $FirstMsi -OutputMsi $first -SourceDateEpoch $SourceDateEpoch | Out-Null } catch { $rejected=$true }
if (-not $rejected -or (Get-FileHash -LiteralPath $first).Hash.ToLowerInvariant() -ne $a.sha256) { throw 'Existing output was not protected.' }
if ((Get-FileHash -LiteralPath $FirstMsi).Hash -ne $before[0] -or (Get-FileHash -LiteralPath $SecondMsi).Hash -ne $before[1]) { throw 'Original evidence was modified.' }
$signedRejected=$null
if ($SignedMsi) {
    $signedHash=(Get-FileHash -LiteralPath $SignedMsi).Hash
    if ((Get-AuthenticodeSignature -LiteralPath $SignedMsi).Status -eq 'NotSigned') { throw 'Signed rejection fixture is unsigned.' }
    $signedOutput=Join-Path $WorkDirectory 'signed-rejected-unsigned.msi'
    $signedRejected=$false
    try { & $normalizer -InputMsi $SignedMsi -OutputMsi $signedOutput -SourceDateEpoch $SourceDateEpoch | Out-Null }
    catch { if ($_.Exception.Message -eq 'Only unsigned MSI inputs may be normalized.') { $signedRejected=$true } else { throw } }
    if (-not $signedRejected -or (Test-Path -LiteralPath $signedOutput) -or (Test-Path -LiteralPath ($signedOutput+'.partial')) -or (Get-FileHash -LiteralPath $SignedMsi).Hash -ne $signedHash) { throw 'Signed input protection failed.' }
}
[pscustomobject]@{same_bytes=$true;idempotent=$true;epoch_changes_package=$true;table_change_changes_package=$true;overwrite_rejected=$true;originals_unchanged=$true;signed_input_rejected=$signedRejected;first=$a;second=$b} |
    ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $WorkDirectory 'result.json')
Write-Host 'Unsigned MSI metadata checks passed; installation and signing qualification remain separate.'
