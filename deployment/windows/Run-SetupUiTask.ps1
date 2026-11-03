[CmdletBinding()]
# Internal broker: the signed agent verifies release metadata and publisher
# before staging. Independently enforce private storage, exact bytes, task
# ownership and a transaction-bound completion receipt here.
param(
    [Parameter(Mandatory=$true)][string]$Directory,
    [Parameter(Mandatory=$true)][ValidateSet('prepare','restart','cleanup')][string]$Operation,
    [string]$OperationId='',
    [string]$AgentSha256='',
    [string]$MarkerSha256='',
    [ValidateRange(1,900)][int]$WaitSeconds=900
)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Customer setup UI recovery requires administrator rights.'}
$Directory=[IO.Path]::GetFullPath($Directory).TrimEnd('\')
$canonical=Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanRemoteSupport'
if($Directory -ine [IO.Path]::GetFullPath($canonical)){throw 'Setup UI recovery requires canonical customer storage.'}
function Assert-PrivateSetupPath([string]$Path) {
    $item=Get-Item -LiteralPath $Path -Force
    if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Setup UI storage must not contain reparse points.'}
    $acl=Get-Acl -LiteralPath $Path
    $allowed=@('S-1-5-18','S-1-5-32-544')
    if($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -notin $allowed){throw 'Setup UI storage has an untrusted owner.'}
    $rules=$acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])
    foreach($rule in $rules){if($rule.AccessControlType -ne 'Allow' -or $rule.IdentityReference.Value -notin $allowed){throw 'Setup UI storage permits an untrusted account.'}}
    foreach($sid in $allowed){if(-not @($rules|Where-Object {$_.IdentityReference.Value -eq $sid -and ($_.FileSystemRights -band [Security.AccessControl.FileSystemRights]::FullControl) -eq [Security.AccessControl.FileSystemRights]::FullControl}).Count){throw 'Setup UI storage lacks required privileged access.'}}
}
Assert-PrivateSetupPath $Directory
$prefix='Swan Explicit UI '
$scheduler=New-Object -ComObject Schedule.Service
$scheduler.Connect();$folder=$scheduler.GetFolder('\')
function Get-SetupUiAction([string]$Action,[string]$Id) {
    $executable=Join-Path $Directory ('setup-ui\'+$Id+'\swan-agent.exe')
    New-ScheduledTaskAction -Execute $executable -Argument ('setup-ui '+$Action+' "'+$Directory+'" '+$Id) -WorkingDirectory (Split-Path $executable)
}
function Assert-SetupUiTask($Task,[string]$Action,[string]$Id) {
    $expected=Get-SetupUiAction $Action $Id
    $principal=[string]$Task.Principal.UserId
    $sid=if($principal -match '^S-\d-'){$principal}else{([Security.Principal.NTAccount]::new($principal)).Translate([Security.Principal.SecurityIdentifier]).Value}
    if($Task.TaskPath -cne '\' -or $Task.TaskName -cne ($prefix+$Action+' '+$Id) -or $Task.Actions.Count -ne 1 -or $sid -cne 'S-1-5-18' -or $Task.Principal.LogonType -ne 'ServiceAccount' -or $Task.Principal.RunLevel -ne 'Highest' -or @($Task.Triggers|Where-Object {$null -ne $_}).Count -or [IO.Path]::GetFullPath($Task.Actions[0].Execute) -ine [IO.Path]::GetFullPath($expected.Execute) -or $Task.Actions[0].Arguments -cne $expected.Arguments -or $Task.Actions[0].WorkingDirectory -ine $expected.WorkingDirectory){throw 'Existing setup UI task does not match the fixed operation.'}
    $security=[Security.AccessControl.RawSecurityDescriptor]::new(($folder.GetTask($Task.TaskName)).GetSecurityDescriptor(7))
    if($security.Owner.Value -cne 'S-1-5-32-544' -or $security.DiscretionaryAcl.Count -ne 2){throw 'Setup UI task ownership or permissions differ.'}
    $seen=@{}
    foreach($ace in $security.DiscretionaryAcl){
        $account=$ace.SecurityIdentifier.Value
        if($ace.AceType -ne [Security.AccessControl.AceType]::AccessAllowed -or $ace.AceFlags -ne [Security.AccessControl.AceFlags]::None -or $account -notin @('S-1-5-18','S-1-5-32-544') -or $ace.AccessMask -ne 0x1F01FF -or $seen.ContainsKey($account)){throw 'Setup UI task permits unexpected access.'}
        $seen[$account]=$true
    }
}
if($Operation -eq 'cleanup'){
    $marker=Get-Item -LiteralPath (Join-Path $Directory 'pending-uninstall')
    if($marker.PSIsContainer -or ($marker.Attributes -band [IO.FileAttributes]::ReparsePoint) -or (Test-Path -LiteralPath (Join-Path $Directory 'swan-agent.exe'))){throw 'Setup UI cleanup requires committed explicit uninstall.'}
    $tasks=@(Get-ScheduledTask|Where-Object {$_.TaskName.StartsWith($prefix,[StringComparison]::Ordinal)})
    # Validate every candidate before stopping or deleting any task.
    foreach($task in $tasks){
        if($task.TaskName -cnotmatch '^Swan Explicit UI (prepare|restart) ([0-9a-f]{32})$'){throw 'Unexpected setup UI task name.'}
        Assert-SetupUiTask $task $Matches[1] $Matches[2]
    }
    foreach($task in $tasks){
        if($task.State -in @('Running','Queued')){$task|Stop-ScheduledTask}
        $deadline=[DateTime]::UtcNow.AddSeconds(10)
        while((Get-ScheduledTask -TaskName $task.TaskName).State -in @('Running','Queued')){if([DateTime]::UtcNow -ge $deadline){throw 'Setup UI task cleanup remains pending.'};Start-Sleep -Milliseconds 100}
        $folder.DeleteTask($task.TaskName,0)
    }
    return
}
if($OperationId -cnotmatch '^[0-9a-f]{32}$' -or $AgentSha256 -cnotmatch '^[0-9a-f]{64}$' -or $MarkerSha256 -cnotmatch '^[0-9a-f]{64}$'){throw 'Invalid fixed setup UI operation identity.'}
foreach($path in @((Join-Path $Directory 'setup-ui'),(Join-Path $Directory ('setup-ui\'+$OperationId)),(Join-Path $Directory ('setup-ui\'+$OperationId+'\swan-agent.exe')))){Assert-PrivateSetupPath $path}
$action=Get-SetupUiAction $Operation $OperationId
if((Get-FileHash -LiteralPath $action.Execute -Algorithm SHA256).Hash.ToLowerInvariant() -cne $AgentSha256){throw 'Staged setup UI agent hash differs.'}
$marker=Join-Path $Directory 'pending-install.json'
if((Get-FileHash -LiteralPath $marker -Algorithm SHA256).Hash.ToLowerInvariant() -cne $MarkerSha256){throw 'Signed setup marker changed.'}
$name=$prefix+$Operation+' '+$OperationId
if(@(Get-ScheduledTask|Where-Object {$_.TaskName.StartsWith($prefix,[StringComparison]::Ordinal) -and $_.TaskName -cne $name -and $_.State -in @('Running','Queued')}).Count){throw 'Another setup UI operation is still running; preserve and recover it first.'}
$task=Get-ScheduledTask -TaskName $name -ErrorAction SilentlyContinue
if($task){Assert-SetupUiTask $task $Operation $OperationId}else{
    $principal=New-ScheduledTaskPrincipal -UserId 'S-1-5-18' -LogonType ServiceAccount -RunLevel Highest
    $settings=New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 15) -MultipleInstances IgnoreNew -Priority 4 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    $task=Register-ScheduledTask -TaskName $name -InputObject (New-ScheduledTask -Action $action -Principal $principal -Settings $settings)
    ($folder.GetTask($name)).SetSecurityDescriptor('O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)',0x10)
    Assert-SetupUiTask $task $Operation $OperationId
}
$receipt=Join-Path (Split-Path $action.Execute) ($Operation+'-result.json')
$startedNew=$false
$previousRun=(Get-ScheduledTaskInfo -TaskName $name).LastRunTime
if($task.State -notin @('Running','Queued')){
    # A verified terminal task may be retried, closing any UI reopened since a
    # failed setup. Never start a duplicate of a still-running attempt.
    if(Test-Path -LiteralPath $receipt){Assert-PrivateSetupPath $receipt;Remove-Item -LiteralPath $receipt}
    # Task Scheduler can round LastRunTime to a second. A quick retry must not
    # mistake the old terminal result for completion of this new attempt.
    while([Math]::Abs(([DateTime]::UtcNow-$previousRun.ToUniversalTime()).TotalSeconds) -lt 1.1){Start-Sleep -Milliseconds 50}
    $task|Start-ScheduledTask
    $startedNew=$true
}
$deadline=[DateTime]::UtcNow.AddSeconds($WaitSeconds)
$observedActive=$false
do{
    $task=Get-ScheduledTask -TaskName $name
    Assert-SetupUiTask $task $Operation $OperationId
    $info=Get-ScheduledTaskInfo -TaskName $name
    if($task.State -in @('Running','Queued')){$observedActive=$true}
    if($task.State -notin @('Running','Queued') -and $info.LastRunTime.Year -gt 2000 -and (-not $startedNew -or $observedActive -or $info.LastRunTime -ne $previousRun)){break}
    if([DateTime]::UtcNow -ge $deadline){throw 'Setup UI recovery timed out; task and signed setup marker remain available for recovery.'}
    Start-Sleep -Milliseconds 200
}while($true)
if($info.LastTaskResult -ne 0){throw 'Setup UI recovery failed; preserve its task and signed setup marker.'}
Assert-PrivateSetupPath $receipt
$file=Get-Item -LiteralPath $receipt
if($file.PSIsContainer -or $file.Length -lt 1 -or $file.Length -gt 4096){throw 'Invalid setup UI completion receipt.'}
$completed=Get-Content -LiteralPath $receipt -Raw|ConvertFrom-Json
$keys=@($completed.PSObject.Properties.Name)
$expectedKeys=@('agent_sha256','marker_sha256','operation','operation_id','schema','system_context')
if($keys.Count -ne 6 -or @($keys|Where-Object {$expectedKeys -cnotcontains $_}).Count -or $completed.schema -ne 1 -or $completed.operation -cne $Operation -or $completed.operation_id -cne $OperationId -or $completed.marker_sha256 -cne $MarkerSha256 -or $completed.agent_sha256 -cne $AgentSha256 -or $completed.system_context -isnot [bool] -or -not $completed.system_context){throw 'Setup UI completion does not match the verified operation.'}
$folder.DeleteTask($name,0)
