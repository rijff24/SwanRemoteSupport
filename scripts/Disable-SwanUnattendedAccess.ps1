[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$ExecutablePath = "C:\Program Files\Swan Remote Support\Swan Remote Support.exe"
)

$ErrorActionPreference = "Stop"

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $WhatIfPreference -and
    -not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Run this script from an elevated PowerShell window."
}

$resolvedExecutable = (Resolve-Path -LiteralPath $ExecutablePath).Path

function Invoke-SwanCommand {
    param([Parameter(Mandatory)][string[]]$Arguments)

    $commandOutput = @(& $resolvedExecutable @Arguments 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "Swan Remote Support command failed with exit code ${LASTEXITCODE}: $($commandOutput -join [Environment]::NewLine)"
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

if ($PSCmdlet.ShouldProcess($resolvedExecutable, "Disable unattended access and restore per-session approval")) {
    Invoke-SwanCommand -Arguments @("--option", "approve-mode", "click")
    Invoke-SwanCommand -Arguments @("--option", "verification-method", "use-temporary-password")
    Invoke-SwanCommand -Arguments @("--option", "allow-only-conn-window-open", "Y")
    Assert-SwanOption -Name "approve-mode" -ExpectedValue "click"
    Assert-SwanOption -Name "verification-method" -ExpectedValue "use-temporary-password"
    Assert-SwanOption -Name "allow-only-conn-window-open" -ExpectedValue "Y"
    Write-Output "Unattended access is disabled. Swan Remote Support must be open and the customer must approve each session."
}
