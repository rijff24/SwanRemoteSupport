$ErrorActionPreference = 'Stop'
if (-not [Environment]::Is64BitOperatingSystem) { throw 'This release requires Windows x64.' }
$nativeArchitecture = (Get-ItemProperty -LiteralPath 'HKLM:/SYSTEM/CurrentControlSet/Control/Session Manager/Environment' -Name PROCESSOR_ARCHITECTURE).PROCESSOR_ARCHITECTURE
if ($nativeArchitecture -ne 'AMD64') { throw 'Native Windows x64 is required; ARM64 is not supported in this release.' }
$version = Get-ItemProperty -LiteralPath 'HKLM:/SOFTWARE/Microsoft/Windows NT/CurrentVersion'
$build = [int]$version.CurrentBuildNumber
if ($version.InstallationType -eq 'Server Core') { throw 'Windows Server Core is not supported.' }
if ($version.InstallationType -eq 'Server') {
    foreach ($year in @(2016, 2019, 2022, 2025)) {
        if ($version.ProductName -match ('Windows Server ' + $year + '\b')) { Write-Output ('server_' + $year); exit 0 }
    }
    throw 'This Windows Server release is not supported.'
}
if ($version.InstallationType -ne 'Client') { throw 'This Windows installation type is not supported.' }
if ($build -ge 22000) { Write-Output 'windows_11' }
elseif ($build -ge 10240) { Write-Output 'windows_10' }
else { throw 'Windows 10 or 11 is required.' }
