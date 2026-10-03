param([Parameter(Mandatory=$true)][string]$EnvironmentFile,[switch]$Resume)
$ErrorActionPreference = 'Stop'
# Machine paths belong in an untracked file. Never evaluate its contents as code.
$settings = @{}
$allowed = @('SWAN_VM_LAB_ROOT','SWAN_QEMU_DIRECTORY','SWAN_VM_ISO',
    'SWAN_VM_NAME','SWAN_VM_MEMORY_MIB','SWAN_VM_ACCELERATOR')
foreach ($line in [IO.File]::ReadAllLines([IO.Path]::GetFullPath($EnvironmentFile))) {
    if ([string]::IsNullOrWhiteSpace($line) -or $line.TrimStart().StartsWith('#')) { continue }
    $parts = $line.Split('=',2)
    if ($parts.Length -ne 2 -or $allowed -notcontains $parts[0]) { throw 'Invalid VM environment entry' }
    if ($settings.ContainsKey($parts[0])) { throw 'Duplicate VM environment entry' }
    $settings[$parts[0]] = $parts[1]
}
foreach ($key in $allowed) {
    if ([string]::IsNullOrWhiteSpace($settings[$key])) { throw "Missing VM setting: $key" }
}
$iso = Get-Item -LiteralPath $settings['SWAN_VM_ISO']
$completionPath = $iso.FullName + '.download-complete.json'
if (-not (Test-Path -LiteralPath $completionPath -PathType Leaf)) {
    throw 'Evaluation ISO has no completed-download record; wait for a successful download before creating the guest'
}
$completion = Get-Content -LiteralPath $completionPath -Raw | ConvertFrom-Json
if ($completion.bytes -ne $iso.Length -or $completion.sha256 -notmatch '^[a-fA-F0-9]{64}$') {
    throw 'Invalid evaluation ISO completion record'
}
if ((Get-FileHash -LiteralPath $iso.FullName -Algorithm SHA256).Hash -ne $completion.sha256) {
    throw 'Evaluation ISO changed after download completion'
}
$parameters = @{
    QemuDirectory = $settings['SWAN_QEMU_DIRECTORY']
    IsoPath = $settings['SWAN_VM_ISO']
    LabDirectory = $settings['SWAN_VM_LAB_ROOT']
    Name = $settings['SWAN_VM_NAME']
    MemoryMiB = [int]$settings['SWAN_VM_MEMORY_MIB']
    Accelerator = $settings['SWAN_VM_ACCELERATOR']
}
$script = if ($Resume) { 'Start-TestVm.ps1' } else { 'New-TestVm.ps1' }
& (Join-Path $PSScriptRoot $script) @parameters
