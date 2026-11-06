[CmdletBinding()]
param(
    [Parameter(Mandatory)][uri]$Uri,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$Sha256,
    [Parameter(Mandatory)][string]$Output
)
$ErrorActionPreference = 'Stop'
if (-not $Uri.IsAbsoluteUri -or $Uri.Scheme -cne 'https' -or $Uri.UserInfo) { throw 'Pinned archive downloads require HTTPS without embedded credentials.' }
$Output = [IO.Path]::GetFullPath($Output)
if (Test-Path -LiteralPath $Output) {
    $existing = Get-Item -LiteralPath $Output
    if ($existing.PSIsContainer -or ($existing.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Existing archive path is unsafe.' }
    if ((Get-FileHash -LiteralPath $Output -Algorithm SHA256).Hash -ine $Sha256) { throw 'Existing archive differs from the reviewed pin; preserve and inspect it.' }
    Write-Output $Output
    return
}
$parent = Get-Item -LiteralPath ([IO.Path]::GetDirectoryName($Output))
if (-not $parent.PSIsContainer -or ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Archive parent directory is unsafe.' }
$previousProtocol = [Net.ServicePointManager]::SecurityProtocol
try {
    [Net.ServicePointManager]::SecurityProtocol = $previousProtocol -bor [Net.SecurityProtocolType]::Tls12
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        $temporary = Join-Path $parent.FullName ([IO.Path]::GetFileName($Output)+'.'+[Guid]::NewGuid().ToString('N')+'.unverified')
        try {
            Invoke-WebRequest -Uri $Uri -OutFile $temporary -UseBasicParsing -TimeoutSec 180
        } catch {
            if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force }
            if ($attempt -eq 3) { throw 'Pinned archive transfer failed after three attempts; no output was published.' }
            Write-Warning ('Pinned archive transfer interrupted; retrying attempt '+($attempt+1)+' of 3.')
            Start-Sleep -Seconds (2 * $attempt)
            continue
        }
        if ((Get-Item -LiteralPath $temporary).Length -gt 512MB -or (Get-FileHash -LiteralPath $temporary -Algorithm SHA256).Hash -ine $Sha256) {
            throw ('Downloaded archive differs from the reviewed pin; unverified bytes retained at '+$temporary)
        }
        # Publish only a complete, hash-verified file. Never replace existing evidence.
        [IO.File]::Move($temporary,$Output)
        Write-Output $Output
        return
    }
} finally {
    [Net.ServicePointManager]::SecurityProtocol = $previousProtocol
}
