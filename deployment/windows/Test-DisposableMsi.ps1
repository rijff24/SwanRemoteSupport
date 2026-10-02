[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('customer','technician')][string]$Edition,
    [Parameter(Mandatory)][ValidateSet('install','repair','uninstall')][string]$Phase,
    [Parameter(Mandatory)][string]$Package,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$PackageSha256,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$AgentSha256,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{40}$')][string]$SourceRevision,
    [Parameter(Mandatory)][string]$ExpectedComputerName,
    [Parameter(Mandatory)][string]$ResultDirectory,
    [string]$Manifest,
    [string]$ManifestSha256,
    [string]$TechnicianSha256
)
$ErrorActionPreference='Stop'
# This deliberately mutates a disposable guest. A private, independently
# provisioned marker and exact machine name prevent accidental host execution.
if ($env:COMPUTERNAME -cne $ExpectedComputerName) { throw 'Wrong test computer.' }
$marker = Get-Content -LiteralPath 'C:/SwanLab/disposable-lab.json' -Raw | ConvertFrom-Json
if ($marker.purpose -cne 'swan-disposable-windows-acceptance' -or $marker.computer_name -cne $ExpectedComputerName) { throw 'Disposable guest marker missing or mismatched.' }
if (-not [Environment]::Is64BitProcess) { throw 'Run the x64 test shell.' }
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Disposable MSI tests require an administrator.' }
$packagePath=(Resolve-Path -LiteralPath $Package).Path
if ([IO.Path]::GetExtension($packagePath) -ine '.msi' -or (Get-FileHash -LiteralPath $packagePath -Algorithm SHA256).Hash -ine $PackageSha256) { throw 'MSI hash or format mismatch.' }
$msi=& (Join-Path $PSScriptRoot 'Get-MsiIdentity.ps1') -Package $packagePath | ConvertFrom-Json
$upgrade=if($Edition -eq 'customer'){'{32A585D7-9A78-4AD2-AF72-D0266EFC709D}'}else{'{A4374699-436F-4917-9A4E-223F62A9E634}'}
if ($msi.upgrade_code -ine $upgrade -or $msi.template -notmatch '^x64;' -or $msi.product_code -notmatch '^\{[A-Fa-f0-9-]{36}\}$') { throw 'Unexpected MSI edition or architecture.' }
$compatibility=(& (Join-Path ([Environment]::GetFolderPath('System')) 'WindowsPowerShell/v1.0/powershell.exe') -NoProfile -NonInteractive -File (Join-Path $PSScriptRoot 'Get-WindowsCompatibility.ps1') | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $compatibility -notmatch '^(windows_10|windows_11|server_2016|server_2019|server_2022|server_2025)$') { throw 'Unsupported guest operating system.' }
$install=if($Edition -eq 'customer'){Join-Path $env:ProgramFiles 'Swan Remote Support'}else{Join-Path $env:LOCALAPPDATA 'SwanRemoteSupport-Technician'}
$main=Join-Path $install $(if($Edition -eq 'customer'){'Swan Remote Support.exe'}else{'SwanRemoteSupport-Technician.exe'})
$agent=if($Edition -eq 'customer'){Join-Path $env:ProgramData 'SwanRemoteSupport/swan-agent.exe'}else{Join-Path $install 'swan-agent.exe'}
$files=@()
if ($Edition -eq 'technician' -and $TechnicianSha256 -notmatch '^[a-fA-F0-9]{64}$') { throw 'Pinned technician executable hash required.' }
if ($Edition -eq 'customer') {
    if ($ManifestSha256 -notmatch '^[a-fA-F0-9]{64}$' -or (Get-FileHash -LiteralPath $Manifest -Algorithm SHA256).Hash -ine $ManifestSha256) { throw 'Pinned customer manifest required.' }
    $files=@(Get-Content -LiteralPath $Manifest -Raw | ConvertFrom-Json)
    $seen=@{}
    foreach($file in $files){
        if ($file.path -notmatch '^[A-Za-z0-9 _\-.@+/]+$' -or @($file.path.Split('/') | Where-Object {$_ -in @('','.', '..') -or $_.EndsWith('.') -or $_.EndsWith(' ')}).Count -or $seen.ContainsKey($file.path) -or $file.sha256 -notmatch '^[a-fA-F0-9]{64}$') { throw 'Unsafe manifest entry.' }
        $seen[$file.path]=$true
    }
    if (@($files | Where-Object path -eq 'LICENSE.txt').Count -ne 1) { throw 'Expected a single packaged license.' }
}
function Assert-Payload {
    foreach($path in @($main,$agent)){if(-not(Test-Path -LiteralPath $path -PathType Leaf)){throw 'Installed application or agent missing.'}}
    if ((Get-FileHash -LiteralPath $agent -Algorithm SHA256).Hash -ine $AgentSha256) { throw 'Installed agent hash mismatch.' }
    if ($Edition -eq 'technician' -and (Get-FileHash -LiteralPath $main -Algorithm SHA256).Hash -ine $TechnicianSha256) { throw 'Installed technician hash mismatch.' }
    foreach($file in $files){
        $path=Join-Path $install $file.path
        if ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ine $file.sha256) { throw "Installed file differs: $($file.path)" }
    }
    if($Edition -eq 'customer'){
        $service=Get-CimInstance Win32_Service -Filter "Name='Swan Remote Support'"
        if(-not $service -or $service.PathName -ine ('"'+$main+'" --service') -or $service.StartName -ne 'LocalSystem' -or $service.State -ne 'Running'){throw 'Customer service ownership, startup or executable mismatch.'}
    }
}
if($Phase -eq 'install'){
    if((Test-Path -LiteralPath $install) -or (Test-Path -LiteralPath $agent)){throw 'Fresh installation requires an unused product directory.'}
    if($Edition -eq 'customer' -and (Get-CimInstance Win32_Service -Filter "Name='Swan Remote Support'")){throw 'Customer service already present.'}
}else{
    Assert-Payload
}
$resultRoot=[IO.Path]::GetFullPath($ResultDirectory)
if(Test-Path -LiteralPath $resultRoot){throw 'Use a new result directory for each phase.'}
New-Item -ItemType Directory -Path $resultRoot | Out-Null
if($Phase -eq 'repair' -and $Edition -eq 'customer'){
    # Modify only a previously hash-verified, non-executable package key path.
    [IO.File]::WriteAllText((Join-Path $install 'LICENSE.txt'),'Disposable repair test: replace this damaged packaged file.')
}
$operation=if($Phase -eq 'install'){'/i'}elseif($Phase -eq 'repair'){'/fvamus'}else{'/x'}
$log=Join-Path $resultRoot 'msi.log'
$process=Start-Process -FilePath (Join-Path ([Environment]::GetFolderPath('System')) 'msiexec.exe') -ArgumentList @($operation,('"'+$packagePath+'"'),'/qn','/norestart','/l*v',('"'+$log+'"')) -WindowStyle Hidden -Wait -PassThru
$report=[ordered]@{edition=$Edition;phase=$Phase;source=$SourceRevision;runner_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant();package_sha256=$PackageSha256.ToLowerInvariant();agent_sha256=$AgentSha256.ToLowerInvariant();msi=$msi;platform=$compatibility;os=(Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber,OSArchitecture);exit_code=$process.ExitCode;reboot_required=($process.ExitCode -eq 3010);passed=$false;scope='Bare MSI lifecycle only; no company enrollment, publisher, update or session acceptance'}
try{
    if($process.ExitCode -notin @(0,3010)){throw "MSI phase failed ($($process.ExitCode)); inspect private log."}
    if($Phase -eq 'uninstall'){
        foreach($path in @($main,$agent)){if(Test-Path -LiteralPath $path){throw 'Uninstall retained the application or agent executable.'}}
        if($Edition -eq 'customer' -and (Get-CimInstance Win32_Service -Filter "Name='Swan Remote Support'")){throw 'Uninstall retained the customer service.'}
    }else{Assert-Payload}
    $report.passed=$true
}finally{
    $report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $resultRoot 'result.json')
}
$report | ConvertTo-Json -Depth 6
