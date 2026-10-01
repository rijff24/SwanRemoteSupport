param(
    [Parameter(Mandatory=$true)][string]$QemuDirectory,
    [Parameter(Mandatory=$true)][string]$IsoPath,
    [Parameter(Mandatory=$true)][string]$LabDirectory,
    [ValidatePattern('^[a-z0-9-]+$')][string]$Name = 'server-2025',
    [ValidateSet('whpx','tcg')][string]$Accelerator = 'whpx',
    [ValidateRange(2048,8192)][int]$MemoryMiB = 4096,
    [ValidateRange(32,128)][int]$DiskGiB = 64
)
$ErrorActionPreference = 'Stop'
$qemu = Join-Path $QemuDirectory 'qemu-system-x86_64.exe'
$qemuImg = Join-Path $QemuDirectory 'qemu-img.exe'
foreach ($required in @($qemu,$qemuImg,$IsoPath)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) { throw "Missing file: $required" }
}
$lab = [IO.Path]::GetFullPath($LabDirectory)
$vm = [IO.Path]::GetFullPath((Join-Path $lab $Name))
if (-not $vm.StartsWith($lab.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) {
    throw 'VM directory must stay within the test lab'
}
if (Test-Path -LiteralPath $vm) { throw 'VM already exists; refusing to overwrite its disk' }
$availableMiB = (Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory / 1024
if ($availableMiB -lt ($MemoryMiB + 1536)) { throw 'Insufficient available memory; leave at least 1.5 GiB for the host' }
New-Item -ItemType Directory -Path $vm | Out-Null
$disk = Join-Path $vm 'system.qcow2'
& $qemuImg create -f qcow2 $disk ($DiskGiB.ToString()+'G')
if ($LASTEXITCODE -ne 0) { throw 'QEMU disk creation failed' }
$arguments = @('-name',$Name,'-machine','q35','-accel',$Accelerator,
    '-cpu','max','-m',$MemoryMiB.ToString(),'-smp','2','-drive',"file=$disk,format=qcow2",
    '-cdrom',[IO.Path]::GetFullPath($IsoPath),'-boot','order=d',
    '-nic','user,model=e1000','-display','none',
    '-vnc','127.0.0.1:20','-monitor','tcp:127.0.0.1:4444,server=on,wait=off')
# User-mode NAT avoids host firewall/bridge changes. Management sockets are loopback only.
# Only one guest may use these ports at a time. No incoming guest ports are forwarded.
[ordered]@{name=$Name;disk=$disk;iso=[IO.Path]::GetFullPath($IsoPath);
    accelerator=$Accelerator;arguments=$arguments;created_utc=[DateTime]::UtcNow.ToString('o')
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $vm 'vm.json') -Encoding UTF8
# Launch through PowerShell's native argument passing rather than string-built shell code.
& $qemu @arguments
if ($LASTEXITCODE -ne 0) { throw "QEMU exited with code $LASTEXITCODE; inspect acceleration availability before retrying" }
