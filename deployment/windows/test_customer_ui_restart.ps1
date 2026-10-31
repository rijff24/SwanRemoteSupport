[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Directory,
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$ResultPath
)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
if (-not [Security.Principal.WindowsIdentity]::GetCurrent().IsSystem) { throw 'Use SYSTEM in a disposable Windows lab with an installed customer and a logged-in standard user.' }
if (Test-Path -LiteralPath $Directory) { throw 'Preserve the previous test directory.' }
$exe=[IO.Path]::GetFullPath($Executable)
$initial=@()
foreach ($process in @(Get-CimInstance Win32_Process -Filter "Name='Swan Remote Support.exe'")) {
    if ($process.SessionId -eq 0 -or $process.ExecutablePath -ine $exe) { continue }
    $owner=Invoke-CimMethod -InputObject $process -MethodName GetOwnerSid
    if ($owner.ReturnValue -ne 0) { throw 'Cannot identify disposable customer UI.' }
    if ($owner.Sid -ne 'S-1-5-18') { $initial+=@{Pid=[int]$process.ProcessId;Sid=$owner.Sid;Created=$process.CreationDate.ToUniversalTime()} }
}
$accounts=@($initial | ForEach-Object Sid | Sort-Object -Unique)
if ($accounts.Count -ne 1 -or $initial.Count -lt 1) { throw 'One genuine standard-user customer desktop is required.' }
$sid=$accounts[0]
$administrators=Get-LocalGroup -SID 'S-1-5-32-544'
if (@(Get-LocalGroupMember -Group $administrators.Name | Where-Object { $_.SID.Value -eq $sid }).Count) { throw 'Administrator UI does not qualify this regression.' }
$serviceBefore=(Get-Service -Name 'Swan Remote Support').Status
if ($serviceBefore -ne 'Running') { throw 'The disposable customer service must already be healthy.' }
$payloadBefore=(Get-FileHash -LiteralPath $exe).Hash
New-Item -ItemType Directory -Path $Directory | Out-Null
& icacls.exe $Directory /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Private fixture directory failed.' }
# This is a separate helper fixture, not a forged installation or authorization.
# Real automatic-update acceptance must also exercise the signed MSI/agent path.
[IO.File]::WriteAllText((Join-Path $Directory 'installed-release.json'),'{}')
. (Join-Path $PSScriptRoot 'Restart-CustomerUi.ps1') -Directory $Directory -Executable $exe
function Require([bool]$Condition,[string]$Message) { if (-not $Condition) { throw $Message } }
function Require-OriginalProcesses {
    foreach ($process in $initial) {
        $current=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Pid)
        Require ($null -ne $current -and $current.CreationDate.ToUniversalTime() -eq $process.Created) 'A safety rejection stopped an original GUI.'
    }
}
function New-OwnedFixtureTask([string]$Name,$Action) {
    $principal=New-ScheduledTaskPrincipal -UserId $sid -LogonType Interactive -RunLevel Limited
    $trigger=New-ScheduledTaskTrigger -AtLogOn -User $sid
    $settings=New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 2) -MultipleInstances IgnoreNew -Priority 4 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    $task=Register-ScheduledTask -TaskName $Name -Action $Action -Principal $principal -Trigger $trigger -Settings $settings
    $scheduler=New-Object -ComObject Schedule.Service;$scheduler.Connect();$registered=$scheduler.GetFolder('\').GetTask($Name)
    $registered.SetSecurityDescriptor(('O:SYG:SYD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FRFXSD;;;'+$sid+')'),0x10)
    Assert-CustomerUiTask $task $Action $sid $registered
    return $task
}
Stop-OwnedCustomerUi @() $exe
$probe=$initial[0]
foreach ($case in @('creation','path','sid')) {
    $created=$probe.Created.ToFileTimeUtc();$path=$exe;$account=$sid
    if ($case -eq 'creation') { $created+=10 }
    elseif ($case -eq 'path') { $path=Join-Path $Directory 'Swan Remote Support.exe' }
    else { $account='S-1-5-18' }
    $rejected=$false
    try { [void][SwanCustomerUiProcess]::Stop($probe.Pid,$created,$path,$account) } catch { $rejected=$true }
    Require $rejected ('Native process guard accepted wrong '+$case)
    Require-OriginalProcesses
}
$context=Read-CustomerUiRecoveryTasks $Directory $exe
$name=Get-CustomerUiTaskName $context.Prefix $sid
Require (-not (Get-ScheduledTask -TaskName $name -ErrorAction SilentlyContinue)) 'Preserve an existing task.'
$foreignAction=New-ScheduledTaskAction -Execute ($env:SystemRoot+'/System32/notepad.exe')
$foreignPrincipal=New-ScheduledTaskPrincipal -UserId 'S-1-5-18' -LogonType ServiceAccount -RunLevel Highest
Register-ScheduledTask -TaskName $name -Action $foreignAction -Principal $foreignPrincipal | Out-Null
$rejected=$false
try { Restart-VerifiedCustomerUi $Directory $exe | Out-Null } catch { $rejected=$true }
Require $rejected 'A foreign task was accepted.'
$foreign=Get-ScheduledTask -TaskName $name
Require ($foreign.Actions[0].Execute -ceq $foreignAction.Execute -and $foreign.Principal.UserId -in @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM')) 'Foreign task was changed.'
Require-OriginalProcesses
$foreign | Unregister-ScheduledTask -Confirm:$false

$originalStop=(Get-Item Function:\Stop-OwnedCustomerUi).ScriptBlock
try {
    function Stop-OwnedCustomerUi { throw 'Simulated interruption before stopping old GUI.' }
    $interrupted=$false
    try { Restart-VerifiedCustomerUi $Directory $exe | Out-Null } catch { $interrupted=$true }
    Require $interrupted 'The pre-stop interruption was not exercised.'
    Require-OriginalProcesses
    $context=Read-CustomerUiRecoveryTasks $Directory $exe
    Require ($context.Tasks.Count -eq 1 -and $context.Tasks.ContainsKey($sid)) 'Restart intent was not durable before process stop.'
} finally { Set-Item Function:\Stop-OwnedCustomerUi -Value $originalStop }

try {
    function Start-ScheduledTask { throw 'Simulated interruption after stopping old GUI, before user launch.' }
    $deferred=Restart-VerifiedCustomerUi $Directory $exe | ConvertFrom-Json
    Require ($deferred.old_processes_closed -eq $initial.Count -and $sid -in $deferred.user_restarts_deferred_to_logon) 'Post-stop restart intent was lost.'
    $context=Read-CustomerUiRecoveryTasks $Directory $exe
    Require ($context.Tasks.Count -eq 1) 'Post-stop interruption lost its durable task.'
} finally { Remove-Item Function:\Start-ScheduledTask }

$restored=Restart-VerifiedCustomerUi $Directory $exe | ConvertFrom-Json
Require ($sid -in $restored.user_restarts_requested) 'Recovery did not request the original standard-user GUI.'
$deadline=[DateTime]::UtcNow.AddSeconds(90);$fresh=@()
do {
    $fresh=@()
    foreach ($process in @(Get-CimInstance Win32_Process -Filter "Name='Swan Remote Support.exe'")) {
        if ($process.SessionId -eq 0 -or $process.ExecutablePath -ine $exe -or $process.CreationDate.ToUniversalTime() -lt (Get-Item (Join-Path $Directory 'installed-release.json')).LastWriteTimeUtc) { continue }
        $owner=Invoke-CimMethod -InputObject $process -MethodName GetOwnerSid
        if ($owner.ReturnValue -eq 0 -and $owner.Sid -eq $sid) { $fresh+=@{Pid=$process.ProcessId;Created=$process.CreationDate.ToUniversalTime()} }
    }
    $queued=Get-ScheduledTask -TaskName $name -ErrorAction SilentlyContinue
    if ($fresh.Count -gt 0 -and -not $queued) { break }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Standard-user relaunch or task self-cleanup did not complete.' }
    Start-Sleep -Seconds 2
} while ($true)
$idempotent=Restart-VerifiedCustomerUi $Directory $exe | ConvertFrom-Json
Require ($idempotent.old_processes_closed -eq 0 -and @($idempotent.user_restarts_requested).Count -eq 0) 'Retry restarted an already fresh UI.'
foreach ($process in $fresh) {
    $current=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Pid)
    Require ($null -ne $current -and $current.CreationDate.ToUniversalTime() -eq $process.Created) 'Fresh UI was stopped on retry.'
}
$action=Get-CustomerUiAction $exe $sid $name
New-OwnedFixtureTask $name $action | Out-Null
$sentinel='SwanLab UI sentinel '+($name -replace '^Swan Customer UI ','')
Register-ScheduledTask -TaskName $sentinel -Action $foreignAction -Principal $foreignPrincipal | Out-Null
[IO.File]::WriteAllText((Join-Path $Directory 'pending-uninstall'),'')
Remove-CustomerUiRecoveryTasks $Directory $exe
Require (-not (Get-ScheduledTask -TaskName $name -ErrorAction SilentlyContinue)) 'Uninstall left its owned UI task.'
$retained=Get-ScheduledTask -TaskName $sentinel
Require ($retained.Actions[0].Execute -ceq $foreignAction.Execute -and $retained.Principal.UserId -in @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM')) 'Uninstall changed an unrelated task.'
$retained | Unregister-ScheduledTask -Confirm:$false
Require ((Get-Service -Name 'Swan Remote Support').Status -eq $serviceBefore -and (Get-FileHash -LiteralPath $exe).Hash -ceq $payloadBefore) 'The helper changed the service or application payload.'
@{
    native_wrong_creation_path_and_sid_rejected=$true
    foreign_task_unchanged=$true
    restart_intent_durable_before_stop=$true
    restart_intent_survives_stop_before_launch=$true
    real_standard_user_gui_relaunched=$true
    temporary_task_self_deleted=$true
    fresh_gui_preserved_on_retry=$true
    owned_task_removed_on_uninstall=$true
    unrelated_task_preserved=$true
    service_and_payload_unchanged=$true
    passwords_or_privileged_user_tokens_used=$false
    standard_user_sid=$sid
    new_gui_processes=$fresh
    automatic_update_qualified=$false
    utc=[DateTime]::UtcNow.ToString('o')
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $ResultPath
