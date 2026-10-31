[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$BuildDirectory,
      [Parameter(Mandatory=$true)][string]$OutputDirectory,
      [switch]$OwnedDisposableLab)
$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$admin=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if(!$admin.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Disposable service tests require administrator access.'}
if(!$OwnedDisposableLab -and ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows')){throw 'Use a disposable Windows CI runner or explicitly select an owned lab VM.'}
$build=(Resolve-Path -LiteralPath $BuildDirectory).Path
$output=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $output){throw 'Preserve existing test output.'}
New-Item -ItemType Directory -Path $output|Out-Null
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
icacls.exe $output /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' ('*'+$sid+':(OI)(CI)F')|Out-Null
if($LASTEXITCODE){throw 'Fixture directory must be private.'}
$records=@()
foreach($case in @(
 @{name='normal';exe='bootstrap-check.exe';core='core-fast.dll';service='SwanLabBootstrapFixture';expected=0},
 @{name='slow-loader';exe='bootstrap-check.exe';core='core-slow.dll';service='SwanLabBootstrapFixture';expected=0},
 @{name='missing-core';exe='bootstrap-check.exe';core=$null;service='SwanLabBootstrapFixture';expected=126},
 @{name='missing-export';exe='bootstrap-check.exe';core='core-missing-export.dll';service='SwanLabBootstrapFixture';expected=127},
 @{name='bounded-loader-timeout';exe='bootstrap-timeout-check.exe';core='core-timeout.dll';service='SwanLabBootstrapTimeoutFixture';expected=1460}
)){
 $name=$case.service
 if(Get-Service -Name $name -ErrorAction SilentlyContinue){throw 'Preserve existing fixture service.'}
 $directory=Join-Path $output $case.name;New-Item -ItemType Directory $directory|Out-Null
 $exe=Join-Path $directory $case.exe;Copy-Item -LiteralPath (Join-Path $build $case.exe) -Destination $exe
 if($case.core){Copy-Item -LiteralPath (Join-Path $build $case.core) -Destination (Join-Path $directory 'librustdesk.dll')}
 $binary='"'+$exe+'" --service';$created=$false;$passed=$false;$snapshots=@();$timer=[Diagnostics.Stopwatch]::StartNew()
 try{
  New-Service -Name $name -BinaryPathName $binary -StartupType Manual|Out-Null;$created=$true
  $null=& sc.exe start $name
  do{
   $service=Get-CimInstance Win32_Service -Filter ("Name='"+$name+"'")
   $snapshots+=@{elapsed=$timer.Elapsed.TotalSeconds;state=$service.State;checkpoint=$service.CheckPoint;wait_hint=$service.WaitHint;pid=$service.ProcessId;exit_code=$service.ExitCode}
   if($service.State -in @('Running','Stopped')){break}
   if($timer.Elapsed.TotalSeconds -gt 100){throw 'Original service initialization exceeded the fixture deadline.'}
   Start-Sleep -Milliseconds 500
  }while($true)
  if($case.expected -eq 0){
   if($service.State -ne 'Running'){throw 'Core did not enter Running.'}
   if($case.name -eq 'slow-loader'){
    $pending=@($snapshots|Where-Object state -eq 'Start Pending')
    if($timer.Elapsed.TotalSeconds -lt 30 -or @($pending.checkpoint|Select-Object -Unique).Count -lt 3){throw 'Long DLL initialization did not show genuine SCM pending checkpoints.'}
   }
   $null=& sc.exe interrogate $name;if($LASTEXITCODE){throw 'Interrogate forwarding failed.'}
   Stop-Service -Name $name
   $controller=Get-Service $name;try{$controller.WaitForStatus('Stopped',[TimeSpan]::FromSeconds(20))}finally{$controller.Dispose()}
  }elseif($service.State -ne 'Stopped' -or $service.ExitCode -ne $case.expected){throw 'Failed initialization did not report the expected stopped/error state.'}
  $deadline=[DateTime]::UtcNow.AddSeconds(10)
  do{
   $remaining=@(Get-CimInstance Win32_Process -Filter ("Name='"+$case.exe+"'")|Where-Object ExecutablePath -ieq $exe)
   if(!$remaining.Count){break};if([DateTime]::UtcNow -gt $deadline){throw 'Owned fixture process survived service shutdown.'};Start-Sleep -Milliseconds 200
  }while($true)
  $passed=$true
 }finally{
  $record=@{case=$case.name;passed=$passed;expected_error=$case.expected;elapsed_seconds=$timer.Elapsed.TotalSeconds;snapshots=$snapshots;real_windows_service=$true;simulated_core_only=$true}
  $record|ConvertTo-Json -Depth 6|Set-Content (Join-Path $directory 'result.json');$records+=$record
  if($created){
   $owned=Get-CimInstance Win32_Service -Filter ("Name='"+$name+"'")
   if($owned -and $owned.PathName -cne $binary){throw 'Fixture ownership changed; no cleanup performed.'}
   if($owned -and $owned.State -ne 'Stopped'){
    foreach($process in @(Get-CimInstance Win32_Process -Filter ("Name='"+$case.exe+"'")|Where-Object ExecutablePath -ieq $exe)){Stop-Process -Id $process.ProcessId -Force}
   }
   $null=& sc.exe delete $name;if($LASTEXITCODE){throw 'Owned fixture service removal failed.'}
   $deadline=[DateTime]::UtcNow.AddSeconds(10)
   while(Get-Service -Name $name -ErrorAction SilentlyContinue){if([DateTime]::UtcNow -gt $deadline){throw 'Fixture service deletion remains pending.'};Start-Sleep -Milliseconds 250}
  }
 }
}
# Exact service-argument handling must not divert desktop or ambiguous commands.
foreach($test in @(@{args='--not-service';exit=7},@{args='--service unexpected';exit=87})){
 $start=[Diagnostics.ProcessStartInfo]::new();$start.FileName=Join-Path $build 'bootstrap-check.exe';$start.Arguments=$test.args;$start.UseShellExecute=$false;$start.CreateNoWindow=$true
 $process=[Diagnostics.Process]::Start($start)
 try{if(!$process.WaitForExit(10000)){$process.Kill();$process.WaitForExit();throw 'Argument fixture did not exit.'};if($process.ExitCode -ne $test.exit){throw 'Service argument handling differs.'}}finally{$process.Dispose()}
}
@{utc=[DateTime]::UtcNow.ToString('o');cases=$records;argument_checks_passed=$true;scope='Real SCM/loader boundary with a simulated DLL; no native Rust core, signing, enrollment, session, cold reboot or complete product acceptance.'}|ConvertTo-Json -Depth 8|Set-Content (Join-Path $output 'verified.json')
Write-Host 'Actual Windows SCM bootstrap cases passed; simulated DLL scope only. No native product or cold-boot acceptance inferred.'
