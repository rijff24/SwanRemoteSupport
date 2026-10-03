[CmdletBinding()]
param([Parameter(Mandatory)][string]$Executable,[Parameter(Mandatory)][string]$ReleasePublicKey,
      [Parameter(Mandatory)][ValidatePattern('^[A-Fa-f0-9]{40}$')][string]$PublisherThumbprint,[int]$Port=8080,
      [string]$ComponentsDirectory,[string]$PublicHostname)
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Server installation requires administrator rights.' }
if ($Port -lt 1024 -or $Port -gt 65535) { throw 'Invalid management port.' }
$source = (Resolve-Path -LiteralPath $Executable).Path
$signature = Get-AuthenticodeSignature -LiteralPath $source
if ($signature.Status -ne 'Valid') { throw 'The company server executable requires a trusted release signature.' }
if ($signature.SignerCertificate.Thumbprint -ne $PublisherThumbprint) { throw 'Company server signature publisher does not match the configured certificate.' }
if ([Convert]::FromBase64String($ReleasePublicKey).Length -ne 32) { throw 'Expected a 32-byte project release public key.' }
$componentPins = @{
    'hbbs.exe'='2102e17d32af3ab313a4096d4a4db307984630eccf4f9ed03ef3dfbd4fdb8f83'
    'hbbr.exe'='5b5fe62f5b5f1fa7df521a243cbfadbeca67bd9df80d5017f23cf62aca59257e'
    'caddy.exe'='586d4a4cd74bdfd2951b6b81766a32904ffa69c4f1c3d521da3870a2120f1d31'
}
$fullStack = -not [string]::IsNullOrEmpty($ComponentsDirectory)
if ($fullStack -ne (-not [string]::IsNullOrEmpty($PublicHostname))) { throw 'Specify both prepared components and the public hostname.' }
if ($fullStack) {
    if ($PublicHostname.Length -gt 253 -or $PublicHostname -notmatch '\.' -or @($PublicHostname.Split('.') | Where-Object { $_ -notmatch '^[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?$' }).Count) { throw 'Invalid company public hostname.' }
    if ($Port -in @(80,443,21115,21116,21117)) { throw 'Management port conflicts with a packaged component.' }
    $ComponentsDirectory = (Resolve-Path -LiteralPath $ComponentsDirectory).Path
    foreach ($name in $componentPins.Keys) {
        $component = Join-Path $ComponentsDirectory $name
        if ((Get-Item -LiteralPath $component).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Component must be a regular file.' }
        if ((Get-FileHash -LiteralPath $component -Algorithm SHA256).Hash -ne $componentPins[$name]) { throw 'Packaged server component hash mismatch.' }
    }
    foreach ($name in @('Caddy-LICENSE.txt','RustDesk-LICENSE.txt','THIRD-PARTY.txt','server-components.json')) {
        if (-not (Test-Path -LiteralPath (Join-Path $ComponentsDirectory $name) -PathType Leaf)) { throw 'Prepared component license or source notice missing.' }
    }
}
# Enumerate first: filtering cmdlets by a missing port/name can itself report an
# error. Inspection failures must abort rather than masquerade as a free port.
$ports = if ($fullStack) { @(80,443,21115,21116,21117,$Port) } else { @($Port) }
$listeners = @(Get-NetTCPConnection -ErrorAction Stop | Where-Object { $_.State -eq 'Listen' -and $_.LocalPort -in $ports })
if ($listeners.Count) { throw 'A required company server port is already in use. Existing services will not be changed.' }
if ($fullStack) {
    if (@(Get-NetUDPEndpoint -ErrorAction Stop | Where-Object LocalPort -eq 21116).Count) { throw 'Company rendezvous UDP port is already in use.' }
    if (@(Get-NetFirewallRule -ErrorAction Stop | Where-Object Name -in @('SwanCompanyServer-TCP','SwanCompanyServer-UDP')).Count) { throw 'Company firewall rule already exists.' }
}
$installDirectory = Join-Path $env:ProgramFiles 'Swan Company Server'
$dataDirectory = Join-Path $env:ProgramData 'SwanCompanyServer'
if (@(Get-Service -ErrorAction Stop | Where-Object Name -eq 'SwanCompanyServer').Count) { throw 'Company server already exists. Use the documented backed-up maintenance upgrade procedure.' }
if ((Test-Path -LiteralPath $installDirectory) -or (Test-Path -LiteralPath $dataDirectory)) { throw 'Company install/data directory already exists. Preserve it and use the recovery or maintenance procedure.' }
New-Item -ItemType Directory -Path $installDirectory,$dataDirectory | Out-Null
& icacls.exe $dataDirectory '/inheritance:r' '/grant:r' '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Cannot protect server data.' }
$destination = Join-Path $installDirectory 'swan-management.exe'
Copy-Item -LiteralPath $source -Destination $destination
$copiedSignature = Get-AuthenticodeSignature -LiteralPath $destination
if ($copiedSignature.Status -ne 'Valid' -or $copiedSignature.SignerCertificate.Thumbprint -ne $PublisherThumbprint) { throw 'Copied management executable does not match the trusted publisher.' }
$serviceEnvironment = @("SWAN_DATA_DIR=$dataDirectory","SWAN_LISTEN=127.0.0.1:$Port","SWAN_RELEASE_PUBLIC_KEY=$ReleasePublicKey")
if ($fullStack) {
    $packaged = Join-Path $installDirectory 'components'
    New-Item -ItemType Directory -Path $packaged | Out-Null
    foreach ($name in $componentPins.Keys) { Copy-Item -LiteralPath (Join-Path $ComponentsDirectory $name) -Destination (Join-Path $packaged $name) }
    foreach ($name in @('Caddy-LICENSE.txt','RustDesk-LICENSE.txt','THIRD-PARTY.txt','server-components.json')) { Copy-Item -LiteralPath (Join-Path $ComponentsDirectory $name) -Destination (Join-Path $packaged $name) }
    $configuration = "{`n    admin off`n}`n$PublicHostname {`n    reverse_proxy 127.0.0.1:$Port`n    header Strict-Transport-Security max-age=31536000`n    request_body {`n        max_size 512MB`n    }`n}`n"
    [IO.File]::WriteAllText((Join-Path $dataDirectory 'Caddyfile'),$configuration,[Text.UTF8Encoding]::new($false))
    $serviceEnvironment += @('SWAN_COMPONENTS=1',"SWAN_PUBLIC_HOST=$PublicHostname")
}
$serviceCreated = $false
$createdRules = @()
try {
    New-Service -Name SwanCompanyServer -DisplayName 'Swan Remote Support Company Server' -BinaryPathName ('"'+$destination+'" --service') -StartupType Automatic | Out-Null
    $serviceCreated = $true
    $serviceRegistry = 'HKLM:/SYSTEM/CurrentControlSet/Services/SwanCompanyServer'
    New-ItemProperty -Path $serviceRegistry -Name Environment -PropertyType MultiString -Value $serviceEnvironment -Force | Out-Null
    & sc.exe failure SwanCompanyServer reset= 86400 actions= restart/60000/restart/60000/restart/300000 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot configure company service recovery.' }
    # Component supervision reports Stopped with a nonzero exit code after cleanup.
    & sc.exe failureflag SwanCompanyServer 1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot enable recovery for reported company service failures.' }
    if ($fullStack) {
        New-NetFirewallRule -Name 'SwanCompanyServer-TCP' -Group 'SwanCompanyServer' -DisplayName 'Swan company HTTPS and transport' -Direction Inbound -Action Allow -Protocol TCP -LocalPort 80,443,21115,21116,21117 | Out-Null
        $createdRules += 'SwanCompanyServer-TCP'
        New-NetFirewallRule -Name 'SwanCompanyServer-UDP' -Group 'SwanCompanyServer' -DisplayName 'Swan company rendezvous UDP' -Direction Inbound -Action Allow -Protocol UDP -LocalPort 21116 | Out-Null
        $createdRules += 'SwanCompanyServer-UDP'
    }
    Start-Service SwanCompanyServer
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    $healthy = $false
    while ([DateTime]::UtcNow -lt $deadline) {
        try { $health = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 2; $healthy = $health.status -eq 'ok' -and $health.product -eq 'Swan Remote Support' } catch { $healthy = $false }
        if ($healthy) { break }
        if ((Get-Service -Name SwanCompanyServer).Status -eq 'Stopped') { throw 'Company service stopped during startup. Review protected component logs.' }
        Start-Sleep -Milliseconds 250
    }
    if (-not $healthy) { throw 'Company management health check timed out.' }
    $receipt = [ordered]@{schema=1; service_name='SwanCompanyServer'; install_directory=[IO.Path]::GetFullPath($installDirectory);
        data_directory=[IO.Path]::GetFullPath($dataDirectory); full_stack=$fullStack; firewall_rules=@($createdRules)}
    [IO.File]::WriteAllText((Join-Path $dataDirectory 'installation.json'),($receipt | ConvertTo-Json -Depth 4),[Text.UTF8Encoding]::new($false))
} catch {
    foreach ($rule in $createdRules) { Remove-NetFirewallRule -Name $rule -ErrorAction Continue }
    if ($serviceCreated) {
        Stop-Service -Name SwanCompanyServer -ErrorAction Continue
        & sc.exe delete SwanCompanyServer | Out-Null
        if ($LASTEXITCODE -ne 0) { Write-Warning 'Failed to remove the newly created company service.' }
    }
    Write-Warning 'Installation failed. Newly created files and private data remain for inspection; preserve them before retrying.'
    throw
}
if ($fullStack) {
    Write-Host "Company components installed. Configure company DNS and forward required ports to this server, then open https://$PublicHostname to complete setup."
    Write-Host 'HTTPS issuance requires public DNS and inbound reachability. Management health alone does not prove external HTTPS or transport connectivity.'
} else { Write-Host "Company server installed on localhost:$Port. Put an HTTPS reverse proxy in front of this service." }
Write-Host "Read the one-time setup token from $dataDirectory/setup-token.txt, then complete the web setup wizard."
Write-Host 'Existing RustDesk and Tailscale services were not changed. Public transport setup will fail if its required ports are occupied.'
