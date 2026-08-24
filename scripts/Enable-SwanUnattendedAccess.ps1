[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$ExecutablePath = "C:\Program Files\Swan Remote Support\Swan Remote Support.exe",
    [ValidateRange(16, 64)]
    [int]$PasswordLength = 24,
    [switch]$CopyCredentialsToClipboard,
    [switch]$ShowCredentials
)

$ErrorActionPreference = "Stop"

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $WhatIfPreference -and
    -not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Run this script from an elevated PowerShell window."
}
if (-not $WhatIfPreference -and -not $CopyCredentialsToClipboard -and -not $ShowCredentials) {
    throw "Choose -CopyCredentialsToClipboard or -ShowCredentials so the unique device password is not generated without a secure handoff."
}

$resolvedExecutable = (Resolve-Path -LiteralPath $ExecutablePath).Path
$alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#%+=_-"
$passwordCharacters = [Collections.Generic.List[char]]::new($PasswordLength)
$random = [Security.Cryptography.RandomNumberGenerator]::Create()
try {
    $randomBytes = [byte[]]::new(128)
    $unbiasedLimit = 256 - (256 % $alphabet.Length)
    while ($passwordCharacters.Count -lt $PasswordLength) {
        $random.GetBytes($randomBytes)
        foreach ($randomByte in $randomBytes) {
            if ($randomByte -lt $unbiasedLimit) {
                $passwordCharacters.Add($alphabet[$randomByte % $alphabet.Length])
                if ($passwordCharacters.Count -eq $PasswordLength) {
                    break
                }
            }
        }
    }
} finally {
    $random.Dispose()
}
$password = -join $passwordCharacters

function Invoke-SwanCommand {
    param([Parameter(Mandatory)][string[]]$Arguments)

    $commandOutput = @(& $resolvedExecutable @Arguments 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "Swan Remote Support command failed with exit code ${LASTEXITCODE}: $($commandOutput -join [Environment]::NewLine)"
    }
    if ($Arguments[0] -eq "--password" -and ($commandOutput -join "`n") -notmatch "Done!") {
        throw "Swan Remote Support did not confirm the permanent-password change: $($commandOutput -join [Environment]::NewLine)"
    }
}

function Assert-SwanOption {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$ExpectedValue
    )

    $actualValue = (& $resolvedExecutable --option $Name 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $actualValue -ne $ExpectedValue) {
        throw "Option '$Name' verification failed. Expected '$ExpectedValue'; received '$actualValue'."
    }
}

if ($PSCmdlet.ShouldProcess($resolvedExecutable, "Enable consented unattended access with a unique device password")) {
    Invoke-SwanCommand -Arguments @("--password", $password)
    Invoke-SwanCommand -Arguments @("--option", "approve-mode", "password")
    Invoke-SwanCommand -Arguments @("--option", "verification-method", "use-permanent-password")
    Invoke-SwanCommand -Arguments @("--option", "allow-only-conn-window-open", "N")
    Assert-SwanOption -Name "approve-mode" -ExpectedValue "password"
    Assert-SwanOption -Name "verification-method" -ExpectedValue "use-permanent-password"
    Assert-SwanOption -Name "allow-only-conn-window-open" -ExpectedValue "N"

    $deviceId = $null
    $idDeadline = [DateTime]::UtcNow.AddSeconds(45)
    do {
        $candidateId = (& $resolvedExecutable --get-id 2>&1 | Out-String).Trim()
        if ($LASTEXITCODE -eq 0 -and $candidateId -match '^\d+$') {
            $deviceId = $candidateId
            break
        }
        Start-Sleep -Seconds 2
    } while ([DateTime]::UtcNow -lt $idDeadline)

    if ([string]::IsNullOrWhiteSpace($deviceId)) {
        throw "Unattended access was configured, but the device ID could not be retrieved. Confirm that Tailscale is connected and the Swan RustDesk server is reachable."
    }

    $credentialReceipt = @"
Customer computer: $env:COMPUTERNAME
Swan Remote Support ID: $deviceId
Unique password: $password
"@.Trim()

    Write-Output "Unattended access is enabled on this device with the customer's explicit consent."
    if ($ShowCredentials) {
        Write-Output $credentialReceipt
        Write-Warning "Visible credentials may be captured by terminal transcripts, RMM logs, screenshots, or screen recordings."
    }
    if ($CopyCredentialsToClipboard) {
        Set-Clipboard -Value $credentialReceipt
        Write-Output "The credential receipt was copied to the clipboard. Paste it into the Swan password manager, then clear the clipboard."
    }
    Write-Warning "Transfer these credentials to Swan Computing through a secure channel, store them in a password manager, and do not reuse the password on another device."
}
