param(
    [Parameter(Mandatory=$true)][string]$QemuDirectory,
    [Parameter(Mandatory=$true)][string]$IsoPath,
    [Parameter(Mandatory=$true)][string]$LabDirectory,
    [ValidatePattern('^[a-z0-9-]+$')][string]$Name = 'server-2025',
    [ValidateSet('whpx','tcg')][string]$Accelerator = 'tcg',
    [ValidateRange(2048,8192)][int]$MemoryMiB = 4096
)
$ErrorActionPreference = 'Stop'
$lab = [IO.Path]::GetFullPath($LabDirectory)
$vm = [IO.Path]::GetFullPath((Join-Path $lab $Name))
if (-not $vm.StartsWith($lab.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) {
    throw 'VM directory must stay within the test lab'
}
$disk = Join-Path $vm 'system.qcow2'
$qemu = Join-Path $QemuDirectory 'qemu-system-x86_64.exe'
foreach ($required in @($disk,$qemu,$IsoPath)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) { throw "Missing file: $required" }
}
$iso = Get-Item -LiteralPath $IsoPath
$completionPath = $iso.FullName + '.download-complete.json'
if (-not (Test-Path -LiteralPath $completionPath -PathType Leaf)) { throw 'Missing ISO completion record' }
$completion = Get-Content -LiteralPath $completionPath -Raw | ConvertFrom-Json
if ($completion.bytes -ne $iso.Length -or $completion.sha256 -notmatch '^[a-fA-F0-9]{64}$') { throw 'Invalid ISO completion record' }
if ((Get-FileHash -LiteralPath $iso.FullName -Algorithm SHA256).Hash -ne $completion.sha256) { throw 'ISO changed after download completion' }
if ((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory / 1024 -lt ($MemoryMiB + 1536)) { throw 'Insufficient memory for the guest and host reserve' }
$running = Get-CimInstance Win32_Process -Filter "Name='qemu-system-x86_64.exe'"
if ($running | Where-Object { $_.CommandLine -and $_.CommandLine.Contains($disk) }) { throw 'Guest disk is already in use' }
# Reconstruct fixed arguments; saved metadata must never supply arbitrary QEMU options.
$arguments = @('-name',$Name,'-machine','q35','-accel',$Accelerator,
    '-m',$MemoryMiB.ToString(),'-smp','2','-drive',"file=$disk,format=qcow2",
    '-cdrom',$iso.FullName,'-boot','order=d','-nic','user,model=e1000',
    '-display','none','-vnc','127.0.0.1:20','-monitor','tcp:127.0.0.1:4444,server=on,wait=off')
[ordered]@{ accelerator=$Accelerator;started_utc=[DateTime]::UtcNow.ToString('o') } |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $vm 'last-start.json') -Encoding UTF8
& $qemu @arguments
if ($LASTEXITCODE -ne 0) { throw "QEMU exited with code $LASTEXITCODE" }
