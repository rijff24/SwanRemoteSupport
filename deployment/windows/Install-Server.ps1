[CmdletBinding()]
param([Parameter(Mandatory)][string]$Executable,[Parameter(Mandatory)][string]$ReleasePublicKey,[int]$Port=8080)
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Server installation requires administrator rights.' }
if ($Port -lt 1024 -or $Port -gt 65535) { throw 'Invalid management port.' }
$source = (Resolve-Path -LiteralPath $Executable).Path
if ((Get-AuthenticodeSignature -LiteralPath $source).Status -ne 'Valid') { throw 'The company server executable requires a trusted release signature.' }
if ([Convert]::FromBase64String($ReleasePublicKey).Length -ne 32) { throw 'Expected a 32-byte project release public key.' }
$installDirectory = Join-Path $env:ProgramFiles 'Swan Company Server'
$dataDirectory = Join-Path $env:ProgramData 'SwanCompanyServer'
if (Get-Service -Name SwanCompanyServer -ErrorAction SilentlyContinue) { throw 'Company server already exists. Use the documented backed-up maintenance upgrade procedure.' }
New-Item -ItemType Directory -Force -Path $installDirectory,$dataDirectory | Out-Null
& icacls.exe $dataDirectory '/inheritance:r' '/grant:r' '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Cannot protect server data.' }
$destination = Join-Path $installDirectory 'swan-management.exe'
Copy-Item -LiteralPath $source -Destination $destination
New-Service -Name SwanCompanyServer -DisplayName 'Swan Remote Support Company Server' -BinaryPathName ('"'+$destination+'" --service') -StartupType Automatic | Out-Null
$serviceRegistry = 'HKLM:/SYSTEM/CurrentControlSet/Services/SwanCompanyServer'
New-ItemProperty -Path $serviceRegistry -Name Environment -PropertyType MultiString -Value @("SWAN_DATA_DIR=$dataDirectory","SWAN_LISTEN=127.0.0.1:$Port","SWAN_RELEASE_PUBLIC_KEY=$ReleasePublicKey") -Force | Out-Null
& sc.exe failure SwanCompanyServer reset= 86400 actions= restart/60000/restart/60000/restart/300000 | Out-Null
Start-Service SwanCompanyServer
Write-Host "Company server installed on localhost:$Port. Put an HTTPS reverse proxy in front of this service."
Write-Host "Read the one-time setup token from $dataDirectory/setup-token.txt, then complete the web setup wizard."
Write-Host 'Existing RustDesk and Tailscale services were not changed. Install the parallel public transport using the pinned release in deployment/transport-source.json.'
