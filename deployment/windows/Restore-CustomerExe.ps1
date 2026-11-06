[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Package,[Parameter(Mandatory=$true)][string]$Directory)
$ErrorActionPreference='Stop'
function Assert-CustomerServiceRecord($Service,[string]$Expected,[bool]$Required) {
    if ($Service.PathName -ine $Expected -or $Service.StartName -ne 'LocalSystem') { throw 'Customer service belongs to another installation.' }
    if ($Required -and ($Service.State -ne 'Running' -or $Service.StartMode -ne 'Auto')) { throw 'Restored customer service is not running automatically.' }
}
function Assert-Service([bool]$Required) {
    $services=@(Get-CimInstance -ClassName Win32_Service -Filter "Name='Swan Remote Support'")
    if ($services.Count -eq 0 -and -not $Required) { return }
    if ($services.Count -ne 1) { throw 'Expected exactly one customer service.' }
    $expected='"'+[IO.Path]::GetFullPath((Join-Path $Directory 'Swan Remote Support.exe'))+'" --service'
    Assert-CustomerServiceRecord $services[0] $expected $Required
}
$installer=New-Object -ComObject WindowsInstaller.Installer
if (@($installer.RelatedProducts('{32A585D7-9A78-4AD2-AF72-D0266EFC709D}')).Count -ne 0) { throw 'EXE rollback cannot modify an MSI-managed installation.' }
if (Test-Path -LiteralPath 'HKLM:\Software\SwanRemoteSupport\Customer') {
    $marker=Get-ItemProperty -LiteralPath 'HKLM:\Software\SwanRemoteSupport\Customer'
    if ($null -ne $marker.PSObject.Properties['ProductCode']) { throw 'EXE rollback cannot modify an MSI registration marker.' }
}
$uninstallKey='HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Swan Remote Support'
if (Test-Path -LiteralPath $uninstallKey) {
    $registered=Get-ItemProperty -LiteralPath $uninstallKey
    $expected='"'+[IO.Path]::GetFullPath((Join-Path $Directory 'Swan Remote Support.exe'))+'" --uninstall'
    if ($registered.WindowsInstaller -eq 1 -or $registered.UninstallString -ine $expected -or [IO.Path]::GetFullPath($registered.InstallLocation) -ine [IO.Path]::GetFullPath($Directory)) { throw 'Registered customer installation identity differs from recovery.' }
}
Assert-Service $false
# The agent verifies exact signed package bytes/publisher before calling this
# script; no uninstall command from registry or company profile is executed here.
$process=Start-Process -FilePath (Resolve-Path -LiteralPath $Package).Path -ArgumentList @('--silent-install','printer=0') -PassThru -Wait -WindowStyle Hidden
if ($process.ExitCode -ne 0) { throw "Customer restoration remains pending (exit $($process.ExitCode))." }
Assert-Service $true
