[CmdletBinding()]
param([switch]$UnattendedConsent,[switch]$Repair,[switch]$ReplaceFailedSetup,[string]$BootstrapPath,[string]$ConfirmedCompanyDomain)
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
if (-not $BootstrapPath) { $BootstrapPath = Join-Path $root 'bootstrap.json' }
$BootstrapPath = [IO.Path]::GetFullPath($BootstrapPath)
$bootstrap = Get-Content -LiteralPath $BootstrapPath -Raw | ConvertFrom-Json
if ($bootstrap.edition -notin @('customer','technician')) { throw 'Unknown package edition.' }
$domain = [uri]$bootstrap.management_url
if ($domain.Scheme -ne 'https' -or $domain.UserInfo -or -not $domain.Host) { throw 'A company HTTPS endpoint is required.' }
Write-Host "Company server: $($domain.AbsoluteUri)"
Write-Host 'Confirm this is the support company you intended to install. The company must approve your device before remote access.'
if (-not $ConfirmedCompanyDomain) { $ConfirmedCompanyDomain = Read-Host 'Type the company server hostname to confirm' }
if ($ConfirmedCompanyDomain -cne $domain.Host) { throw 'Company verification cancelled.' }
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
    & $agent setup $BootstrapPath --accept-company
    if ($LASTEXITCODE -ne 0) { throw 'Company configuration verification failed.' }
}
& $agent verify-bootstrap $BootstrapPath
if ($LASTEXITCODE -ne 0) { throw 'Installer does not match the configured company, edition or trust keys.' }
$installers = @(Get-ChildItem -LiteralPath $root -File | Where-Object { $_.Name -match '^SwanRemoteSupport-install\.(exe|msi)$' })
if ($installers.Count -ne 1) { throw 'Expected one company installer.' }
$packageArguments = @('verify-package',(Join-Path $root 'release.json'),$installers[0].FullName)
$recordArguments = @('record-installation',(Join-Path $root 'release.json'),$installers[0].FullName)
if ($Repair) { $packageArguments += '--repair'; $recordArguments += '--repair' }
& $agent @packageArguments
if ($LASTEXITCODE -ne 0) { throw 'Installer metadata, hash or publisher validation failed.' }
$prepareArguments = @('prepare-installation',(Join-Path $root 'release.json'),$installers[0].FullName)
if ($Repair) { $prepareArguments += '--repair' }
if ($ReplaceFailedSetup) { $prepareArguments += '--replace-failed-setup' }
& $agent @prepareArguments
if ($LASTEXITCODE -ne 0) { throw 'Cannot prepare explicit installation or preserve cancelled update evidence.' }
$systemDirectory = [Environment]::GetFolderPath('System')
$msiExecutable = Join-Path $systemDirectory 'msiexec.exe'
$msiOperation = '/i'
if ($installers[0].Extension -eq '.msi') {
    $msiOperation = (& (Join-Path $systemDirectory 'WindowsPowerShell/v1.0/powershell.exe') -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File (Join-Path $root 'Get-MsiInstallMode.ps1') -Package $installers[0].FullName | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $msiOperation -notin @('/i','/fvamus')) { throw 'Cannot select a safe MSI installation or repair mode.' }
}
$installationMarker = Join-Path $editionDirectory 'pending-install.json'
$activityStream = [IO.File]::Open((Join-Path $editionDirectory 'activity.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite)
$activityLocked = $false
try {
    # LockFile's exclusive byte range overlaps fs2's session/update lock range.
    # Refuse active sessions; do not terminate them to perform setup.
    $activityStream.Lock(0,1)
    $activityLocked = $true
    $prepared = Get-Content -LiteralPath $installationMarker -Raw | ConvertFrom-Json
    $requested = Get-Content -LiteralPath (Join-Path $root 'release.json') -Raw | ConvertFrom-Json
    if ($prepared.payload -cne $requested.payload -or $prepared.signature -cne $requested.signature) { throw 'Another company setup owns the installation recovery marker.' }
if ($bootstrap.edition -eq 'customer') {
    $configurationTask = @(Get-ScheduledTask | Where-Object { $_.TaskName -eq 'Swan Company Configuration' })
    if ($configurationTask.Count -gt 1) { throw 'Ambiguous existing configuration task.' }
    if ($configurationTask.Count -eq 1) {
        $expectedAgent = [IO.Path]::GetFullPath((Join-Path $editionDirectory 'swan-agent.exe'))
        if ($configurationTask[0].Actions.Count -ne 1 -or [IO.Path]::GetFullPath($configurationTask[0].Actions[0].Execute) -ine $expectedAgent -or $configurationTask[0].Actions[0].Arguments -cne 'watch' -or $configurationTask[0].Principal.UserId -notin @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM')) { throw 'Existing configuration task does not match this installation.' }
        $configurationTask[0] | Disable-ScheduledTask | Out-Null
        $configurationTask[0] | Stop-ScheduledTask
        $stopDeadline = [DateTime]::UtcNow.AddSeconds(10)
        while (@(Get-CimInstance Win32_Process -Filter "Name='swan-agent.exe'" | Where-Object { $_.ExecutablePath -ieq $expectedAgent }).Count -gt 0) {
            if ([DateTime]::UtcNow -ge $stopDeadline) { throw 'Installed configuration agent did not stop. Close its running processes and retry setup.' }
            Start-Sleep -Milliseconds 100
        }
    }
    if ($installers[0].Extension -eq '.msi') {
        $process = Start-Process -FilePath $msiExecutable -ArgumentList @($msiOperation,('"' + $installers[0].FullName + '"'),'/passive','/norestart') -PassThru -Wait -WindowStyle Hidden
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
} else {
    $releaseEnvelope = Get-Content -LiteralPath (Join-Path $root 'release.json') -Raw | ConvertFrom-Json
    # The agent already verified this signed metadata and installer before any copy.
    $release = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($releaseEnvelope.payload)) | ConvertFrom-Json
    $portable = Join-Path $editionDirectory 'SwanRemoteSupport-Technician.exe'
    if ($release.format -eq 'msi') {
        $process = Start-Process -FilePath $msiExecutable -ArgumentList @($msiOperation,('"' + $installers[0].FullName + '"'),'/passive','/norestart',('INSTALLFOLDER="' + $editionDirectory + '"')) -PassThru -Wait -WindowStyle Hidden
        if ($process.ExitCode -notin @(0,3010)) { throw "Technician MSI installation failed ($($process.ExitCode))." }
        if (-not (Test-Path -LiteralPath $portable)) { throw 'Technician MSI did not install the expected executable.' }
    } elseif ($release.format -eq 'exe' -and $release.sha256 -ceq $release.installed_sha256) {
        Copy-Item -LiteralPath $installers[0].FullName -Destination $portable -Force
    } else { throw 'Technician setup requires an MSI or a portable EXE with matching installed identity.' }
    Copy-Item -LiteralPath $agent -Destination (Join-Path $editionDirectory 'swan-agent.exe') -Force
}
} finally {
    try { if ($activityLocked) { $activityStream.Unlock(0,1) } }
    finally { $activityStream.Dispose() }
}
# Sessions remain blocked by the durable marker while the agent verifies the
# complete installed identity under its own exclusive lock.
& $agent @recordArguments
if ($LASTEXITCODE -ne 0) { throw 'Installed application or agent verification failed. Installation recovery is required.' }
if (Test-Path -LiteralPath $installationMarker) { throw 'Company installation recovery marker remains; support sessions stay blocked.' }
if ($bootstrap.edition -eq 'customer') {
    $action = New-ScheduledTaskAction -Execute (Join-Path $editionDirectory 'swan-agent.exe') -Argument 'watch'
    $trigger = New-ScheduledTaskTrigger -AtStartup
    $settings = New-ScheduledTaskSettingsSet -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero)
    Register-ScheduledTask -TaskName 'Swan Company Configuration' -Action $action -Trigger $trigger -Settings $settings -User 'SYSTEM' -RunLevel Highest -Force | Out-Null
    Enable-ScheduledTask -TaskName 'Swan Company Configuration' | Out-Null
    Start-ScheduledTask -TaskName 'Swan Company Configuration'
    Write-Host 'Installed and enrolled. Your company must approve the device. Support status and stop controls remain available.'
} else {
    Copy-Item -LiteralPath (Join-Path $root 'Open-Technician.ps1') -Destination (Join-Path $editionDirectory 'Open-Technician.ps1') -Force
    $shortcutPath = Join-Path ([Environment]::GetFolderPath('Programs')) 'Swan Remote Support Technician.lnk'
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
    $shortcut.Arguments = '-NoLogo -NoProfile -ExecutionPolicy Bypass -File "' + (Join-Path $editionDirectory 'Open-Technician.ps1') + '"'
    $shortcut.WorkingDirectory = $editionDirectory
    $shortcut.IconLocation = $portable
    $shortcut.Save()
    if ($release.format -eq 'exe') {
        Copy-Item -LiteralPath (Join-Path $root 'Uninstall-Technician.ps1') -Destination (Join-Path $editionDirectory 'Uninstall-Technician.ps1') -Force
        $uninstallShortcut=$shell.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Programs')) 'Uninstall Swan Remote Support Technician.lnk'))
        $uninstallShortcut.TargetPath=Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
        $uninstallShortcut.Arguments='-NoLogo -NoProfile -ExecutionPolicy Bypass -File "'+(Join-Path $editionDirectory 'Uninstall-Technician.ps1')+'"'
        $uninstallShortcut.WorkingDirectory=$editionDirectory
        $uninstallShortcut.IconLocation=$portable
        $uninstallShortcut.Save()
    }
    Write-Host 'Technician application installed. Use the Start menu shortcut to sign in and request support sessions.'
}
