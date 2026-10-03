[CmdletBinding()]
param([Parameter(Mandatory)][string]$Package,[Parameter(Mandatory)][string]$TechnicianExecutable,[Parameter(Mandatory)][string]$AgentExecutable)
$ErrorActionPreference='Stop'
if($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or -not $env:RUNNER_TEMP){throw 'This destructive repair fixture is restricted to disposable Windows CI runners.'}
$packagePath=(Resolve-Path -LiteralPath $Package).Path
if($packagePath -notmatch '-unsigned\.msi$'){throw 'Select an explicitly unsigned test MSI.'}
$expected=@{
 'SwanRemoteSupport-Technician.exe'=(Get-FileHash -LiteralPath $TechnicianExecutable -Algorithm SHA256).Hash
 'swan-agent.exe'=(Get-FileHash -LiteralPath $AgentExecutable -Algorithm SHA256).Hash
}
$registry='HKCU:\Software\SwanRemoteSupport\Technician'
if(Test-Path -LiteralPath $registry){throw 'Existing technician registration must be preserved.'}
$root=Join-Path ([IO.Path]::GetFullPath($env:RUNNER_TEMP)) ('swan-msi-lifecycle-'+[guid]::NewGuid().ToString('N'))
$installation=Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'SwanRemoteSupport-Technician'
if(Test-Path -LiteralPath $installation){throw 'Existing technician installation must be preserved.'}
New-Item -ItemType Directory -Path $root | Out-Null
function Invoke-Msi([string[]]$Arguments,[string]$LogName){
 if($Arguments.Count -ne 2 -or $Arguments[0] -notin @('/i','/fa','/x') -or $Arguments[1] -cne $packagePath -or $LogName -notmatch '^(install|repair|uninstall)\.log$'){throw 'Invalid MSI lifecycle command binding.'}
 $log=Join-Path $root $LogName
 $all=@($Arguments)+@('/qn','/norestart','/l*v',$log)
 if(@($all|Where-Object {$_ -match '["\r\n]'}).Count){throw 'Unsupported MSI argument'}
 $start=[Diagnostics.ProcessStartInfo]::new()
 $start.FileName=Join-Path $env:SystemRoot 'System32\msiexec.exe'
 $start.UseShellExecute=$false
 $start.Arguments=($all|ForEach-Object {'"'+$_+'"'}) -join ' '
 $process=[Diagnostics.Process]::Start($start)
 try{
  if(-not $process.WaitForExit(120000)){throw 'MSI still running; inspect its process and retained log before retrying.'}
  if($process.ExitCode -ne 0){throw ('MSI failed with '+$process.ExitCode+'; retained log '+$log)}
 }finally{$process.Dispose()}
}
function Verify-Payload {
 foreach($name in $expected.Keys){
  if((Get-FileHash -LiteralPath (Join-Path $installation $name) -Algorithm SHA256).Hash -ne $expected[$name]){throw 'Installed MSI payload hash mismatch.'}
 }
 if(-not(Test-Path -LiteralPath $registry)){throw 'Technician MSI registration missing.'}
}
Invoke-Msi -Arguments @('/i',$packagePath) -LogName 'install.log'
Verify-Payload
$repairTarget=[IO.Path]::GetFullPath((Join-Path $installation 'SwanRemoteSupport-Technician.exe'))
if(-not $repairTarget.StartsWith([IO.Path]::GetFullPath($installation)+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Repair target escaped owned installation.'}
Remove-Item -LiteralPath $repairTarget
Invoke-Msi -Arguments @('/fa',$packagePath) -LogName 'repair.log'
Verify-Payload
Invoke-Msi -Arguments @('/x',$packagePath) -LogName 'uninstall.log'
foreach($name in $expected.Keys){if(Test-Path -LiteralPath (Join-Path $installation $name)){throw 'MSI payload remains after uninstall.'}}
if(Test-Path -LiteralPath $registry){throw 'MSI registration remains after uninstall.'}
$version=Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
[ordered]@{passed=$true;windows=$version.ProductName;build=$version.CurrentBuildNumber;installation_type=$version.InstallationType;unsigned_test=$true;install=$true;repair=$true;uninstall=$true;scope='MSI payload lifecycle under the CI account; no enrollment, standard-user GUI, signature acceptance or remote session'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $root 'result.json')
Write-Output ('Technician MSI lifecycle passed; evidence: '+$root)
