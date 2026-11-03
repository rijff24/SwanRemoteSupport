$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$root=Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanRemoteSupport'
$prefix='Swan Explicit UI '
if(Test-Path -LiteralPath $root){throw 'This broker test requires an empty disposable Windows runner; preserve existing company storage.'}
if(@(Get-ScheduledTask|Where-Object {$_.TaskName.StartsWith($prefix,[StringComparison]::Ordinal)}).Count){throw 'Preserve existing setup UI tasks.'}
$id=[guid]::NewGuid().ToString('N');$madeRoot=$false;$ownedNames=@();$source=$null
$scratch=Join-Path ([IO.Path]::GetTempPath()) ('swan-ui-broker-'+$id)
$broker=Join-Path $PSScriptRoot 'Run-SetupUiTask.ps1'
function Require([bool]$Condition,[string]$Message){if(-not $Condition){throw $Message}}
function Rejected([scriptblock]$Action){$rejected=$false;try{& $Action}catch{$rejected=$true};Require $rejected 'Unsafe broker request was accepted.'}
try {
    & (Join-Path $PSScriptRoot 'Prepare-CustomerDirectory.ps1') -Directory $root -Existing reject
    $madeRoot=$true
    [IO.File]::WriteAllText((Join-Path $root 'broker-test.guard'),$id)
    New-Item -ItemType Directory -Path $scratch|Out-Null
    $stage=Join-Path $root ('setup-ui\'+$id)
    New-Item -ItemType Directory -Path $stage|Out-Null
    $source=Join-Path $scratch 'BrokerFixture.cs';$exe=Join-Path $stage 'swan-agent.exe'
    # This inert fixture proves Task Scheduler boundaries and SYSTEM execution,
    # not signing, actual UI recovery, or a successful product installation.
    [IO.File]::WriteAllText($source,@'
using System;
using System.IO;
using System.Reflection;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Threading;
class BrokerFixture {
 static string Hash(string path){using(var h=SHA256.Create())return BitConverter.ToString(h.ComputeHash(File.ReadAllBytes(path))).Replace("-","").ToLowerInvariant();}
 static int Main(string[] args){
  if(!WindowsIdentity.GetCurrent().IsSystem || args.Length!=4 || args[0]!="setup-ui")return 91;
  string stage=Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);
  string counter=Path.Combine(stage,"starts.txt");
  File.WriteAllText(counter,(File.Exists(counter)?int.Parse(File.ReadAllText(counter))+1:1).ToString());
  string mode=File.ReadAllText(Path.Combine(stage,"mode.txt"));
  if(mode=="slow")Thread.Sleep(10000);
  if(mode=="fail")return 7;
  if(mode=="missing")return 0;
  string id=mode=="wrong"?new string('0',32):args[3];
  string json="{\"schema\":1,\"operation_id\":\""+id+"\",\"operation\":\""+args[1]+"\",\"marker_sha256\":\""+Hash(Path.Combine(args[2],"pending-install.json"))+"\",\"agent_sha256\":\""+Hash(Assembly.GetExecutingAssembly().Location)+"\",\"system_context\":true}";
  File.WriteAllText(Path.Combine(stage,args[1]+"-result.json"),json);
  return 0;
 }
}
'@)
    $csc=Join-Path $env:SystemRoot 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
    & $csc /nologo /target:exe (('/out:')+$exe) $source
    if($LASTEXITCODE -ne 0){throw 'SYSTEM fixture compilation failed.'}
    [IO.File]::WriteAllText((Join-Path $root 'pending-install.json'),'isolated broker marker fixture; no device or company')
    $agentHash=(Get-FileHash $exe).Hash.ToLowerInvariant()
    $markerHash=(Get-FileHash (Join-Path $root 'pending-install.json')).Hash.ToLowerInvariant()
    $mode=Join-Path $stage 'mode.txt';$counter=Join-Path $stage 'starts.txt';$receipt=Join-Path $stage 'prepare-result.json'
    [IO.File]::WriteAllText($mode,'normal')
    $name=$prefix+'prepare '+$id;$ownedNames+=$name
    function Run-Broker([int]$Seconds=20){& $broker -Directory $root -Operation prepare -OperationId $id -AgentSha256 $agentHash -MarkerSha256 $markerHash -WaitSeconds $Seconds}
    Rejected {& $broker -Directory $root -Operation prepare -OperationId $id -AgentSha256 ('0'*64) -MarkerSha256 $markerHash}
    Rejected {& $broker -Directory $root -Operation prepare -OperationId $id -AgentSha256 $agentHash -MarkerSha256 ('0'*64)}
    Require (-not (Test-Path $counter) -and -not (Get-ScheduledTask $name -ErrorAction SilentlyContinue)) 'Rejected hashes started a SYSTEM job.'
    $foreign=New-ScheduledTaskAction -Execute (Join-Path $env:SystemRoot 'System32/notepad.exe')
    $principal=New-ScheduledTaskPrincipal -UserId 'S-1-5-18' -LogonType ServiceAccount -RunLevel Highest
    Register-ScheduledTask $name -Action $foreign -Principal $principal|Out-Null
    Rejected {Run-Broker}
    Require ((Get-ScheduledTask $name).Actions[0].Execute -ceq $foreign.Execute -and -not (Test-Path $counter)) 'Foreign action was changed or started.'
    Unregister-ScheduledTask $name -Confirm:$false
    $expectedAction=New-ScheduledTaskAction -Execute $exe -Argument ('setup-ui prepare "'+$root+'" '+$id) -WorkingDirectory $stage
    Register-ScheduledTask $name -Action $expectedAction -Principal $principal|Out-Null
    $scheduler=New-Object -ComObject Schedule.Service;$scheduler.Connect()
    ($scheduler.GetFolder('\').GetTask($name)).SetSecurityDescriptor('O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FRFX;;;BU)',0x10)
    Rejected {Run-Broker}
    Require (-not (Test-Path $counter)) 'A task with untrusted account access was started.'
    Unregister-ScheduledTask $name -Confirm:$false
    Run-Broker
    Require ((Get-Content $counter) -eq '1' -and -not (Get-ScheduledTask $name -ErrorAction SilentlyContinue)) 'Positive SYSTEM task did not execute once and clean up.'
    $completion=Get-Content $receipt -Raw|ConvertFrom-Json
    Require $completion.system_context 'Fixture did not run as SYSTEM.'
    Run-Broker
    Require ((Get-Content $counter) -eq '2') 'Retry reused a stale success receipt.'
    [IO.File]::WriteAllText($mode,'fail');Rejected {Run-Broker}
    Require ((Get-ScheduledTaskInfo $name).LastTaskResult -eq 7 -and (Get-Content $counter) -eq '3' -and -not (Test-Path $receipt)) 'Nonzero result or old receipt was accepted.'
    [IO.File]::WriteAllText($mode,'missing');Rejected {Run-Broker}
    Require ((Get-ScheduledTaskInfo $name).LastTaskResult -eq 0 -and (Get-Content $counter) -eq '4' -and -not (Test-Path $receipt)) 'Exit0 without a completion receipt was accepted.'
    [IO.File]::WriteAllText($mode,'wrong');Rejected {Run-Broker}
    Require ((Get-ScheduledTask $name).State -notin @('Running','Queued') -and (Get-Content $counter) -eq '5' -and (Get-Content $receipt -Raw|ConvertFrom-Json).operation_id -ceq ('0'*32)) 'Wrong-receipt fixture did not actually execute.'
    [IO.File]::WriteAllText($mode,'slow');Rejected {Run-Broker 1}
    Require ((Get-ScheduledTask $name).State -in @('Running','Queued') -and (Test-Path (Join-Path $root 'pending-install.json'))) 'Timeout destroyed the live attempt or recovery marker.'
    $startsBefore=[int](Get-Content $counter)
    Run-Broker 25
    Require ([int](Get-Content $counter) -eq $startsBefore) 'Continuing a live attempt started a duplicate SYSTEM job.'
    [IO.File]::WriteAllText($mode,'fail');Rejected {Run-Broker}
    $foreignName=$prefix+'invalid '+$id;$ownedNames+=$foreignName
    Register-ScheduledTask $foreignName -Action $foreign -Principal $principal|Out-Null
    [IO.File]::WriteAllText((Join-Path $root 'pending-uninstall'),'')
    Rejected {& $broker -Directory $root -Operation cleanup}
    Require ([bool](Get-ScheduledTask $name -ErrorAction SilentlyContinue) -and [bool](Get-ScheduledTask $foreignName -ErrorAction SilentlyContinue)) 'Cleanup partly deleted tasks before rejecting a foreign entry.'
    Unregister-ScheduledTask $foreignName -Confirm:$false
    & $broker -Directory $root -Operation cleanup
    Require (-not (Get-ScheduledTask $name -ErrorAction SilentlyContinue)) 'Uninstall cleanup left an owned broker task.'
    $result=@{actual_system_execution=$true;wrong_hash_and_marker_rejected=$true;foreign_action_preserved=$true;untrusted_task_acl_rejected=$true;nonzero_and_missing_or_wrong_receipt_rejected=$true;new_attempt_does_not_reuse_stale_receipt=$true;timeout_preserved_live_task_and_marker=$true;continuation_did_not_duplicate_task=$true;foreign_cleanup_rejected_before_deletion=$true;owned_uninstall_task_cleanup=$true;signed_agent_or_actual_installer_qualified=$false}
} finally {
    if($madeRoot){
        foreach($taskName in $ownedNames){
            $task=Get-ScheduledTask $taskName -ErrorAction SilentlyContinue
            if($task){
                if($task.State -in @('Running','Queued')){
                    $task|Stop-ScheduledTask;$deadline=[DateTime]::UtcNow.AddSeconds(10)
                    while((Get-ScheduledTask $taskName).State -in @('Running','Queued')){if([DateTime]::UtcNow -ge $deadline){throw 'Preserve the fixture while its SYSTEM job is still live.'};Start-Sleep -Milliseconds 100}
                }
                Unregister-ScheduledTask $taskName -Confirm:$false
            }
        }
        $expected=[IO.Path]::GetFullPath((Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanRemoteSupport'))
        $rootItem=Get-Item -LiteralPath $root
        if($rootItem.FullName -ine $expected -or ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -or [IO.File]::ReadAllText((Join-Path $root 'broker-test.guard')) -cne $id){throw 'Refusing cleanup of an unverified fixture path.'}
        if(@(Get-ChildItem -LiteralPath $root -Force -Recurse|Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'Refusing fixture cleanup through a reparse point.'}
        Remove-Item -LiteralPath $root -Recurse
    }
    if($source -and (Test-Path -LiteralPath $source)){Remove-Item -LiteralPath $source}
    if(Test-Path -LiteralPath $scratch){Remove-Item -LiteralPath $scratch}
}
$result|ConvertTo-Json -Compress
