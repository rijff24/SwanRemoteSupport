[CmdletBinding()]
param([switch]$UnattendedConsent)
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$bootstrap = Get-Content -LiteralPath (Join-Path $root 'bootstrap.json') -Raw | ConvertFrom-Json
if ($bootstrap.edition -notin @('customer','technician')) { throw 'Unknown package edition.' }
$domain = [uri]$bootstrap.management_url
if ($domain.Scheme -ne 'https' -or $domain.UserInfo -or -not $domain.Host) { throw 'A company HTTPS endpoint is required.' }
Write-Host "Company server: $($domain.AbsoluteUri)"
Write-Host 'Confirm this is the support company you intended to install. The company must approve your device before remote access.'
if ((Read-Host 'Type the company server hostname to confirm') -cne $domain.Host) { throw 'Company verification cancelled.' }
if ($UnattendedConsent) {
    Write-Host 'Unattended access allows approved company technicians to connect while you are absent, including the sign-in screen.'
    if ((Read-Host 'Type ALLOW ONGOING SUPPORT to consent') -cne 'ALLOW ONGOING SUPPORT') { throw 'Ongoing-access consent not granted.' }
}
$agent = Join-Path $root 'swan-agent.exe'
# The executable verifies signed company profiles itself. Never use unsigned setup files for a production release.
$signature = Get-AuthenticodeSignature -LiteralPath $agent
if ($signature.Status -ne 'Valid') { throw 'Configuration agent is not trusted.' }
$editionDirectory = if ($bootstrap.edition -eq 'technician') { Join-Path $env:LOCALAPPDATA 'SwanRemoteSupport-Technician' } else { Join-Path $env:ProgramData 'SwanRemoteSupport' }
if ($bootstrap.edition -eq 'customer') {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Run customer setup as administrator.' }
}
New-Item -ItemType Directory -Force -Path $editionDirectory | Out-Null
if ($bootstrap.edition -eq 'customer') {
    & icacls.exe $editionDirectory '/inheritance:r' '/grant:r' '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot protect customer configuration directory.' }
}
$env:SWAN_STATE_DIR = $editionDirectory
if (-not (Test-Path -LiteralPath (Join-Path $editionDirectory 'managed-state.json'))) {
    & $agent setup (Join-Path $root 'bootstrap.json') --accept-company
    if ($LASTEXITCODE -ne 0) { throw 'Company configuration verification failed.' }
}
& $agent verify-bootstrap (Join-Path $root 'bootstrap.json')
if ($LASTEXITCODE -ne 0) { throw 'Installer does not match the configured company, edition or trust keys.' }
$installers = @(Get-ChildItem -LiteralPath $root -File | Where-Object { $_.Name -match '^SwanRemoteSupport-install\.(exe|msi)$' })
if ($installers.Count -ne 1) { throw 'Expected one company installer.' }
& $agent verify-package (Join-Path $root 'release.json') $installers[0].FullName
if ($LASTEXITCODE -ne 0) { throw 'Installer metadata, hash or publisher validation failed.' }
if ($bootstrap.edition -eq 'customer') {
    if ($installers[0].Extension -eq '.msi') {
        $process = Start-Process -FilePath 'msiexec.exe' -ArgumentList @('/i',('"' + $installers[0].FullName + '"'),'/passive','/norestart') -PassThru -Wait -WindowStyle Hidden
    } else {
        $process = Start-Process -FilePath $installers[0].FullName -ArgumentList '--silent-install','printer=0' -PassThru -Wait -WindowStyle Hidden
    }
    if ($process.ExitCode -notin @(0,3010)) { throw "Installation failed ($($process.ExitCode))." }
    $installed = Join-Path $env:ProgramFiles 'Swan Remote Support/Swan Remote Support.exe'
    if (-not (Test-Path -LiteralPath $installed)) { throw 'Installed executable not found. Verify the package install directory.' }
    $deviceId = (& $installed --get-id | Out-String).Trim()
    $enrollmentArguments = @('enroll', $env:COMPUTERNAME, $deviceId)
    if ($UnattendedConsent) { $enrollmentArguments += '--unattended-consent' }
    & $agent @enrollmentArguments
    if ($LASTEXITCODE -ne 0) { throw 'Device enrollment failed. Access remains blocked.' }
    Copy-Item -LiteralPath $agent -Destination (Join-Path $editionDirectory 'swan-agent.exe') -Force
    & $agent record-installation (Join-Path $root 'release.json')
    if ($LASTEXITCODE -ne 0) { throw 'Installed application or configuration agent verification failed. Background updates were not enabled.' }
    $action = New-ScheduledTaskAction -Execute (Join-Path $editionDirectory 'swan-agent.exe') -Argument 'watch'
    $trigger = New-ScheduledTaskTrigger -AtStartup
    $settings = New-ScheduledTaskSettingsSet -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero)
    Register-ScheduledTask -TaskName 'Swan Company Configuration' -Action $action -Trigger $trigger -Settings $settings -User 'SYSTEM' -RunLevel Highest -Force | Out-Null
    Start-ScheduledTask -TaskName 'Swan Company Configuration'
    Write-Host 'Installed and enrolled. Your company must approve the device. Support status and stop controls remain available.'
} else {
    $releaseEnvelope = Get-Content -LiteralPath (Join-Path $root 'release.json') -Raw | ConvertFrom-Json
    # The agent already verified this signed metadata and installer before any copy.
    $release = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($releaseEnvelope.payload)) | ConvertFrom-Json
    if ($release.format -ne 'exe' -or $release.sha256 -cne $release.installed_sha256) { throw 'Technician setup requires a portable EXE with matching installed identity.' }
    $portable = Join-Path $editionDirectory 'SwanRemoteSupport-Technician.exe'
    Copy-Item -LiteralPath $installers[0].FullName -Destination $portable -Force
    Copy-Item -LiteralPath $agent -Destination (Join-Path $editionDirectory 'swan-agent.exe') -Force
    & $agent record-installation (Join-Path $root 'release.json')
    if ($LASTEXITCODE -ne 0) { throw 'Technician installed application or agent verification failed.' }
    Copy-Item -LiteralPath (Join-Path $root 'Open-Technician.ps1') -Destination (Join-Path $editionDirectory 'Open-Technician.ps1') -Force
    $shortcutPath = Join-Path ([Environment]::GetFolderPath('Programs')) 'Swan Remote Support Technician.lnk'
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
    $shortcut.Arguments = '-NoLogo -NoProfile -ExecutionPolicy Bypass -File "' + (Join-Path $editionDirectory 'Open-Technician.ps1') + '"'
    $shortcut.WorkingDirectory = $editionDirectory
    $shortcut.IconLocation = $portable
    $shortcut.Save()
    Write-Host 'Technician application installed. Use the Start menu shortcut to sign in and request support sessions.'
}
