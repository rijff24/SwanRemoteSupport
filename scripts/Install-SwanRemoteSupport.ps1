[CmdletBinding()]
param(
    [string]$InstallerPath,
    [ValidateRange(30, 300)]
    [int]$InstallTimeoutSeconds = 120,
    [ValidateRange(60, 1800)]
    [int]$TailscaleLoginTimeoutSeconds = 600,
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$TailscaleVersion = "1.102.3",
    [ValidatePattern('^[A-Fa-f0-9]{64}$')]
    [string]$TailscaleMsiSha256 = "03ac8183c6e3ce276e9b44281ebe7e4c02aef28a971034ca170c4b665df42dce",
    [ValidatePattern('^tag:[a-z0-9-]+$')]
    [string]$RequiredTailscaleTag = "tag:swan-customer",
    [ValidatePattern('^(?:\d{1,3}\.){3}\d{1,3}$')]
    [string]$ServerAddress = "100.82.236.84",
    [string]$ExpectedSwanSignerOrganization = "SignPath Foundation",
    [switch]$CustomerConsentConfirmed,
    [switch]$AllowUnsignedSwanInstaller,
    [switch]$SkipTailscaleCheck,
    [switch]$Interactive
)

$ErrorActionPreference = "Stop"
$installedExecutable = "C:\Program Files\Swan Remote Support\Swan Remote Support.exe"
$scriptDirectory = Split-Path -Parent $PSCommandPath
$tailscaleInstallPath = Join-Path $env:ProgramFiles "Tailscale\tailscale.exe"
$tailscaleDownloadUrl = "https://pkgs.tailscale.com/stable/tailscale-setup-$TailscaleVersion-amd64.msi"

function Complete-InteractiveRun {
    if ($Interactive) {
        [void](Read-Host "Press Enter to close")
    }
}

function Get-SignerOrganization {
    param([Parameter(Mandatory)][Security.Cryptography.X509Certificates.X509Certificate2]$Certificate)

    foreach ($part in ($Certificate.Subject -split ',\s*')) {
        if ($part -like 'O=*') {
            return $part.Substring(2)
        }
    }
    return $null
}

function Assert-TrustedAuthenticodePublisher {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$ExpectedOrganization,
        [switch]$AllowUnsigned
    )

    $signature = Get-AuthenticodeSignature -LiteralPath $Path
    if ($signature.Status -eq [Management.Automation.SignatureStatus]::Valid -and $signature.SignerCertificate) {
        $organization = Get-SignerOrganization -Certificate $signature.SignerCertificate
        if ($organization -ne $ExpectedOrganization) {
            throw "'$Path' has a valid signature, but its publisher organization is '$organization' instead of '$ExpectedOrganization'."
        }
        return
    }

    if ($AllowUnsigned) {
        Write-Warning "TEST ONLY: '$Path' does not have the required trusted signature. Do not use -AllowUnsignedSwanInstaller for a customer installation."
        return
    }

    throw "'$Path' is not signed with a valid trusted Authenticode certificate. Status: $($signature.Status). Refusing to install it."
}

function Get-TailscaleExecutable {
    $candidates = @(
        (Get-Command tailscale.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -First 1),
        $tailscaleInstallPath
    ) | Where-Object { $_ -and (Test-Path -LiteralPath $_) } | Select-Object -Unique

    if ($candidates) {
        return $candidates[0]
    }
    return $null
}

function Get-TailscaleVersion {
    param([Parameter(Mandatory)][string]$ExecutablePath)

    $versionText = (& $ExecutablePath version 2>&1 | Select-Object -First 1).ToString().Trim()
    if ($LASTEXITCODE -ne 0 -or $versionText -notmatch '^(\d+\.\d+\.\d+)') {
        throw "Could not determine the installed Tailscale version from '$ExecutablePath'."
    }
    return [version]$Matches[1]
}

function Install-OfficialTailscale {
    if (-not [Environment]::Is64BitOperatingSystem) {
        throw "This deployment bundle currently supports only 64-bit Windows."
    }

    $temporaryMsi = Join-Path ([IO.Path]::GetTempPath()) "Swan-Tailscale-$PID.msi"
    if (Test-Path -LiteralPath $temporaryMsi) {
        throw "The exact temporary Tailscale path already exists: '$temporaryMsi'. Remove it after checking that no other Swan setup is running."
    }

    try {
        Write-Output "Downloading the official Tailscale $TailscaleVersion Windows MSI from pkgs.tailscale.com..."
        $previousProgressPreference = $ProgressPreference
        $ProgressPreference = 'SilentlyContinue'
        try {
            Invoke-WebRequest -Uri $tailscaleDownloadUrl -OutFile $temporaryMsi -UseBasicParsing
        } finally {
            $ProgressPreference = $previousProgressPreference
        }

        $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $temporaryMsi).Hash
        if ($actualHash -ne $TailscaleMsiSha256) {
            throw "The downloaded Tailscale MSI checksum did not match the pinned release. Expected $TailscaleMsiSha256; received $actualHash."
        }
        Assert-TrustedAuthenticodePublisher -Path $temporaryMsi -ExpectedOrganization "Tailscale Inc."

        Write-Output "Installing the verified official Tailscale package..."
        $msiArguments = @('/i', ('"{0}"' -f $temporaryMsi), '/qn', '/norestart', 'TS_NOLAUNCH=1')
        $msiProcess = Start-Process -FilePath "msiexec.exe" -ArgumentList $msiArguments -PassThru -Wait
        if ($msiProcess.ExitCode -notin 0, 3010) {
            throw "The official Tailscale MSI returned exit code $($msiProcess.ExitCode)."
        }
        if ($msiProcess.ExitCode -eq 3010) {
            Write-Warning "Tailscale requested a Windows restart. Complete this setup, then restart the computer before the support handoff."
        }
    } finally {
        if (Test-Path -LiteralPath $temporaryMsi) {
            Remove-Item -LiteralPath $temporaryMsi -Force
        }
    }
}

function Get-TailscaleStatus {
    param([Parameter(Mandatory)][string]$ExecutablePath)

    $statusOutput = @(& $ExecutablePath status --json 2>&1)
    if ($LASTEXITCODE -ne 0) {
        return $null
    }
    try {
        return (($statusOutput -join [Environment]::NewLine) | ConvertFrom-Json)
    } catch {
        throw "Tailscale returned status data that could not be parsed as JSON."
    }
}

function Test-TailscaleOnline {
    param($Status)

    return ($Status -and $Status.BackendState -eq 'Running' -and $Status.Self -and $Status.Self.Online)
}

function Wait-ForTailscaleState {
    param(
        [Parameter(Mandatory)][string]$ExecutablePath,
        [Parameter(Mandatory)][DateTime]$Deadline,
        [switch]$RequireTag
    )

    do {
        $status = Get-TailscaleStatus -ExecutablePath $ExecutablePath
        if (Test-TailscaleOnline -Status $status) {
            $tags = @($status.Self.Tags | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
            if (-not $RequireTag -or $RequiredTailscaleTag -in $tags) {
                return $status
            }
        }
        Start-Sleep -Seconds 5
    } while ([DateTime]::UtcNow -lt $Deadline)

    return $null
}

function Test-TcpPort {
    param(
        [Parameter(Mandatory)][string]$Address,
        [Parameter(Mandatory)][int]$Port
    )

    $client = [Net.Sockets.TcpClient]::new()
    try {
        $connectTask = $client.ConnectAsync($Address, $Port)
        try {
            return ($connectTask.Wait(3000) -and $client.Connected)
        } catch [AggregateException] {
            return $false
        }
    } finally {
        $client.Dispose()
    }
}

try {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        if ($Interactive) {
            $elevationArguments = @(
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-File", ('"{0}"' -f $PSCommandPath),
                "-Interactive",
                "-InstallTimeoutSeconds", $InstallTimeoutSeconds,
                "-TailscaleLoginTimeoutSeconds", $TailscaleLoginTimeoutSeconds,
                "-TailscaleVersion", $TailscaleVersion,
                "-TailscaleMsiSha256", $TailscaleMsiSha256,
                "-RequiredTailscaleTag", $RequiredTailscaleTag,
                "-ServerAddress", $ServerAddress,
                "-ExpectedSwanSignerOrganization", ('"{0}"' -f $ExpectedSwanSignerOrganization)
            )
            if (-not [string]::IsNullOrWhiteSpace($InstallerPath)) {
                $elevationArguments += @("-InstallerPath", ('"{0}"' -f $InstallerPath))
            }
            if ($AllowUnsignedSwanInstaller) {
                $elevationArguments += "-AllowUnsignedSwanInstaller"
            }
            if ($CustomerConsentConfirmed) {
                $elevationArguments += "-CustomerConsentConfirmed"
            }
            if ($SkipTailscaleCheck) {
                $elevationArguments += "-SkipTailscaleCheck"
            }
            $elevatedProcess = Start-Process -FilePath "powershell.exe" -ArgumentList $elevationArguments -Verb RunAs -PassThru -Wait
            exit $elevatedProcess.ExitCode
        }
        throw "Express setup must be run as Administrator."
    }

    if (-not $CustomerConsentConfirmed) {
        if (-not $Interactive) {
            throw "Customer authorization is required. Review CUSTOMER_INSTALL.md and PRIVACY.md, then rerun with -CustomerConsentConfirmed only after the computer owner has approved unattended support."
        }

        Write-Output "This setup will install Tailscale if needed, join this PC to Swan's restricted tailnet role, install a background Swan Remote Support service, and enable password-protected unattended support."
        Write-Output "Review CUSTOMER_INSTALL.md and PRIVACY.md in this folder, including the customer's right to stop or uninstall support."
        $consentResponse = Read-Host "With the computer owner's authorization, type YES to continue"
        if ($consentResponse -cne "YES") {
            throw "Installation was cancelled because explicit customer authorization was not confirmed."
        }
        $CustomerConsentConfirmed = $true
    }

    if ([string]::IsNullOrWhiteSpace($InstallerPath)) {
        $installers = @(Get-ChildItem -LiteralPath $scriptDirectory -Filter "SwanRemoteSupport-*-install.exe" -File)
        if ($installers.Count -ne 1) {
            throw "Expected exactly one SwanRemoteSupport-*-install.exe beside this script; found $($installers.Count)."
        }
        $InstallerPath = $installers[0].FullName
    }
    $resolvedInstaller = (Resolve-Path -LiteralPath $InstallerPath).Path
    Assert-TrustedAuthenticodePublisher -Path $resolvedInstaller -ExpectedOrganization $ExpectedSwanSignerOrganization -AllowUnsigned:$AllowUnsignedSwanInstaller

    if ($SkipTailscaleCheck) {
        Write-Warning "TEST ONLY: Tailscale installation, tag enforcement, and network checks were skipped. Do not use -SkipTailscaleCheck for a customer installation."
    } else {
        $tailscaleExecutable = Get-TailscaleExecutable
        if ($tailscaleExecutable) {
            Assert-TrustedAuthenticodePublisher -Path $tailscaleExecutable -ExpectedOrganization "Tailscale Inc."
            $installedTailscaleVersion = Get-TailscaleVersion -ExecutablePath $tailscaleExecutable
        }

        if (-not $tailscaleExecutable -or $installedTailscaleVersion -lt [version]$TailscaleVersion) {
            Install-OfficialTailscale
            $tailscaleExecutable = Get-TailscaleExecutable
            if (-not $tailscaleExecutable) {
                throw "Tailscale installation completed, but tailscale.exe was not found."
            }
            Assert-TrustedAuthenticodePublisher -Path $tailscaleExecutable -ExpectedOrganization "Tailscale Inc."
            $installedTailscaleVersion = Get-TailscaleVersion -ExecutablePath $tailscaleExecutable
            if ($installedTailscaleVersion -lt [version]$TailscaleVersion) {
                throw "Tailscale $installedTailscaleVersion is installed, but this bundle requires at least $TailscaleVersion."
            }
        } else {
            Write-Output "Using verified Tailscale $installedTailscaleVersion."
        }

        $tailscaleService = Get-Service -Name "Tailscale" -ErrorAction SilentlyContinue
        if (-not $tailscaleService) {
            throw "The Tailscale Windows service was not found after installation."
        }
        if ($tailscaleService.Status -ne [ServiceProcess.ServiceControllerStatus]::Running) {
            Start-Service -InputObject $tailscaleService
            $tailscaleService.WaitForStatus([ServiceProcess.ServiceControllerStatus]::Running, [TimeSpan]::FromSeconds(30))
        }

        $tailscaleStatus = Get-TailscaleStatus -ExecutablePath $tailscaleExecutable
        if (-not (Test-TailscaleOnline -Status $tailscaleStatus)) {
            Write-Output "Tailscale needs a one-time manual login. Sign in to the Swan-managed tailnet in the browser window."
            Write-Output "The login requests the restricted $RequiredTailscaleTag role; no reusable auth key is stored in this installer."
            & $tailscaleExecutable login "--advertise-tags=$RequiredTailscaleTag" --unattended "--timeout=${TailscaleLoginTimeoutSeconds}s"
            if ($LASTEXITCODE -ne 0) {
                throw "Manual Tailscale login did not complete successfully. Exit code: $LASTEXITCODE."
            }
        }

        $tagDeadline = [DateTime]::UtcNow.AddSeconds($TailscaleLoginTimeoutSeconds)
        $tailscaleStatus = Wait-ForTailscaleState -ExecutablePath $tailscaleExecutable -Deadline $tagDeadline -RequireTag
        if (-not $tailscaleStatus) {
            throw "Tailscale is not online with required tag '$RequiredTailscaleTag'. In the Tailscale admin console, confirm the device joined the correct tailnet and assign/approve that tag, then run setup again."
        }

        $preferenceOutput = @(& $tailscaleExecutable set --unattended=true --accept-routes=false --ssh=false --advertise-exit-node=false --shields-up=false --webclient=false --auto-update=true --update-check=true 2>&1)
        if ($LASTEXITCODE -ne 0) {
            throw "Tailscale safe-preference configuration failed: $($preferenceOutput -join [Environment]::NewLine)"
        }

        $unreachablePorts = @(21115, 21116, 21117 | Where-Object { -not (Test-TcpPort -Address $ServerAddress -Port $_) })
        if ($unreachablePorts.Count -gt 0) {
            throw "The Swan RustDesk server at $ServerAddress is not reachable on required TCP port(s): $($unreachablePorts -join ', '). Check the Tailscale policy, server containers/services, and Windows/Linux firewall before installing. UDP 21116 must also be allowed by policy; Swan registration is verified after installation."
        }
    }

    Write-Output "Installing Swan Remote Support..."
    $installProcess = Start-Process -FilePath $resolvedInstaller -ArgumentList "--silent-install", "printer=0" -PassThru -Wait
    if ($installProcess.ExitCode -ne 0) {
        throw "The installer returned exit code $($installProcess.ExitCode)."
    }

    $deadline = [DateTime]::UtcNow.AddSeconds($InstallTimeoutSeconds)
    while (-not (Test-Path -LiteralPath $installedExecutable) -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Seconds 2
    }
    if (-not (Test-Path -LiteralPath $installedExecutable)) {
        throw "The installed executable did not appear at '$installedExecutable' within $InstallTimeoutSeconds seconds."
    }

    $service = $null
    while ([DateTime]::UtcNow -lt $deadline) {
        $service = Get-Service -Name "Swan Remote Support" -ErrorAction SilentlyContinue
        if ($service -and $service.Status -eq [ServiceProcess.ServiceControllerStatus]::Running) {
            break
        }
        if ($service) {
            Start-Service -InputObject $service
        }
        Start-Sleep -Seconds 2
    }
    if (-not $service -or $service.Status -ne [ServiceProcess.ServiceControllerStatus]::Running) {
        throw "The Swan Remote Support Windows service did not reach the Running state within $InstallTimeoutSeconds seconds."
    }

    $enableScript = Join-Path $scriptDirectory "Enable-SwanUnattendedAccess.ps1"
    if (-not (Test-Path -LiteralPath $enableScript)) {
        throw "The unattended-access helper is missing: '$enableScript'."
    }

    & $enableScript -ExecutablePath $installedExecutable -CopyCredentialsToClipboard
    if (-not $?) {
        throw "Unattended-access configuration failed."
    }

    Write-Output "Express setup completed successfully."
    Write-Output "Open Swan Remote Support once to confirm that it shows Ready for unattended support and Online."
    Complete-InteractiveRun
} catch {
    Write-Error -ErrorRecord $_ -ErrorAction Continue
    Complete-InteractiveRun
    exit 1
}
