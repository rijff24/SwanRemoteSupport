[CmdletBinding()]
param([Parameter(Mandatory)][string]$Package,[Parameter(Mandatory)][string]$CustomerExecutable)
$ErrorActionPreference='Stop'
if($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or -not $env:RUNNER_TEMP){throw 'This installation test requires a disposable Windows CI runner.'}
$packagePath=(Resolve-Path -LiteralPath $Package).Path
if($packagePath -notmatch '-unsigned\.msi$'){throw 'Select an explicitly unsigned test MSI.'}
$packageHash=(Get-FileHash -LiteralPath $packagePath).Hash
$executableHash=(Get-FileHash -LiteralPath $CustomerExecutable).Hash
$installation=Join-Path $env:ProgramFiles 'Swan Remote Support'
$state=Join-Path $env:ProgramData 'SwanRemoteSupport'
$registration='HKLM:\Software\SwanRemoteSupport\Customer'
$serviceName='Swan Remote Support'
if((Test-Path -LiteralPath $installation) -or (Test-Path -LiteralPath $state) -or (Test-Path -LiteralPath $registration) -or (Get-Service -Name $serviceName -ErrorAction SilentlyContinue)){throw 'Preserve an existing customer installation or configuration.'}
$root=Join-Path ([IO.Path]::GetFullPath($env:RUNNER_TEMP)) ('swan-customer-service-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root|Out-Null
$script:msiActive=$false
function Invoke-OwnedMsi([string]$Operation,[string]$Name){
 if($Operation -notin @('/i','/x') -or $Name -notin @('install','uninstall')){throw 'Invalid lifecycle operation.'}
 if((Get-FileHash -LiteralPath $packagePath).Hash -cne $packageHash){throw 'MSI bytes changed.'}
 $log=Join-Path $root ($Name+'.log')
 if(@($packagePath,$log)|Where-Object {$_ -match '["\r\n]'}){throw 'Unsupported MSI argument.'}
 $start=[Diagnostics.ProcessStartInfo]::new()
 $start.FileName=Join-Path $env:SystemRoot 'System32\msiexec.exe'
 $start.UseShellExecute=$false
 $start.CreateNoWindow=$true
 $start.Arguments=$Operation+' "'+$packagePath+'" /qn /norestart /l*v "'+$log+'"'
 $process=[Diagnostics.Process]::Start($start)
 $script:msiActive=$true
 try{
  if(-not $process.WaitForExit(180000)){throw 'Original MSI is still running; do not launch a competing cleanup.'}
  $script:msiActive=$false
  if($process.ExitCode -ne 0){throw ('MSI '+$Name+' failed: '+$process.ExitCode)}
 }finally{$process.Dispose()}
}
$installed=$false
$passed=$false
$record=[ordered]@{fixture_revision=$env:GITHUB_SHA;package_sha256=$packageHash.ToLowerInvariant();unsigned_test=$true;passed=$false;scope='Fresh administrative MSI install/start and service module inspection, then uninstall on disposable CI; no cold reboot, enrollment, signature acceptance or remote session'}
try{
 $timer=[Diagnostics.Stopwatch]::StartNew()
 Invoke-OwnedMsi '/i' 'install'
 $installed=$true
 $timer.Stop()
 $record.install_elapsed_seconds=$timer.Elapsed.TotalSeconds
 $target=Join-Path $installation 'Swan Remote Support.exe'
 if((Get-FileHash -LiteralPath $target).Hash -cne $executableHash){throw 'Installed launcher differs from built payload.'}
 $shortcut=Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'Swan Remote Support.lnk'
 if(-not (Test-Path -LiteralPath $shortcut)){throw 'Installed customer shortcut is missing.'}
 $shell=New-Object -ComObject WScript.Shell
 $link=$shell.CreateShortcut($shortcut)
 if([IO.Path]::GetFullPath($link.TargetPath) -ine [IO.Path]::GetFullPath($target)){throw 'Customer shortcut must directly open the installed application rather than invoke MSI advertisement.'}
 Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
using System.Text;
public static class SwanCustomerShortcutCheck {
 [DllImport("msi.dll", CharSet=CharSet.Unicode, EntryPoint="MsiGetShortcutTargetW")]
 public static extern uint GetTarget(string shortcut, StringBuilder product, StringBuilder feature, StringBuilder component);
}
'@
 if([SwanCustomerShortcutCheck]::GetTarget($shortcut,[Text.StringBuilder]::new(39),[Text.StringBuilder]::new(39),[Text.StringBuilder]::new(39)) -eq 0){throw 'Advertised customer shortcut can trigger an uncoordinated MSI repair for standard users.'}
 $record.customer_shortcut_direct_target_verified=$true
 $record.customer_shortcut_not_advertised=$true
 $service=Get-CimInstance Win32_Service -Filter "Name='Swan Remote Support'"
 if($service.State -ne 'Running' -or $service.StartMode -ne 'Auto' -or $service.StartName -ne 'LocalSystem'){throw 'MSI did not start the expected automatic LocalSystem service.'}
 if($service.PathName -cne ('"'+$target+'" --service')){throw 'Unexpected service executable or arguments.'}
 $process=Get-Process -Id $service.ProcessId
 try{$modules=@($process.Modules|ForEach-Object ModuleName)}finally{$process.Dispose()}
 if(-not ($modules -icontains 'librustdesk.dll')){throw 'Expected native service core missing.'}
 if(@($modules|Where-Object {$_ -ieq 'flutter_windows.dll' -or $_ -imatch '_plugin\.dll$'}).Count){throw 'Headless service eagerly loaded desktop DLLs.'}
 $record.service_running_without_manual_start=$true
 $record.no_desktop_engine_or_plugin_modules=$true
 $record.service_modules=$modules
 $passed=$true
}catch{
 $record.failure=$_.Exception.Message
 throw
}finally{
 try{if($installed -and -not $script:msiActive){
  try{
   Invoke-OwnedMsi '/x' 'uninstall'
   if((Get-Service -Name $serviceName -ErrorAction SilentlyContinue) -or (Test-Path -LiteralPath (Join-Path $installation 'Swan Remote Support.exe')) -or (Test-Path -LiteralPath $registration)){throw 'Owned customer installation remains after uninstall.'}
   $record.uninstall_verified=$true
  }catch{$passed=$false;$record.cleanup_failure=$_.Exception.Message;throw}
 }}finally{
 $record.passed=$passed
 $record.completed_utc=[DateTime]::UtcNow.ToString('o')
 $record|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $root 'result.json')
 }
}
Write-Output ('Customer service startup and module test passed; evidence: '+$root)
