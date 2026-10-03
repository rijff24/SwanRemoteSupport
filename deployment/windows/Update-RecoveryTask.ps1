[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][ValidateSet('register','remove')][string]$Operation,
    [Parameter(Mandatory=$true)][string]$Directory,
    [Parameter(Mandatory=$true)][ValidateSet('customer','technician')][string]$Edition,
    [Parameter(Mandatory=$true)][UInt64]$Sequence,
    [switch]$InspectOnly
)
$ErrorActionPreference='Stop'
function Assert-RecoveryTask($Task,[string]$Helper,[string[]]$Accounts) {
    if ($Task.TaskPath -cne '\' -or $Task.Actions.Count -ne 1 -or [IO.Path]::GetFullPath($Task.Actions[0].Execute) -ine $Helper -or $Task.Actions[0].Arguments -cne 'recover-update-task' -or $Task.Principal.UserId -notin $Accounts) { throw 'Update recovery task belongs to another installation.' }
}
$directoryPath=[IO.Path]::GetFullPath($Directory).TrimEnd('\')
if ($Sequence -eq 0) { throw 'A positive recovery sequence is required.' }
$helper=Join-Path $directoryPath ("updates\$Sequence\swan-agent.exe")
$hash=[Security.Cryptography.SHA256]::Create()
try { $identifier=([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($directoryPath.ToLowerInvariant())))).Replace('-','').Substring(0,16) } finally { $hash.Dispose() }
$name="Swan Update Recovery $Edition $identifier $Sequence"
$user=if ($Edition -eq 'customer') { 'S-1-5-18' } else { [Security.Principal.WindowsIdentity]::GetCurrent().User.Value }
$tasks=@(Get-ScheduledTask | Where-Object { $_.TaskName -ceq $name })
if ($tasks.Count -gt 1) { throw 'Ambiguous update recovery task.' }
if ($tasks.Count -eq 1) {
    $task=$tasks[0]
    $account=if ($Edition -eq 'customer') { @('SYSTEM','S-1-5-18','NT AUTHORITY\SYSTEM') } else { @($user,[Security.Principal.WindowsIdentity]::GetCurrent().Name) }
    Assert-RecoveryTask $task $helper $account
}
if ($InspectOnly) {
    @{task_name=$name;helper=$helper;principal=$user;operation=$Operation;task_mutated=$false} | ConvertTo-Json -Compress
    return
}
if ($Operation -eq 'remove') {
    if ($tasks.Count -eq 1) { $tasks[0] | Unregister-ScheduledTask -Confirm:$false }
    return
}
if ((Get-AuthenticodeSignature -LiteralPath $helper).Status -ne 'Valid') { throw 'Recovery helper signature is invalid.' }
$action=New-ScheduledTaskAction -Execute $helper -Argument 'recover-update-task'
$repeat=New-ScheduledTaskTrigger -Once -At ([DateTime]::Now.AddMinutes(5)) -RepetitionInterval (New-TimeSpan -Minutes 5)
$settings=New-ScheduledTaskSettingsSet -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Hours 2) -MultipleInstances IgnoreNew
if ($Edition -eq 'customer') {
    $startup=New-ScheduledTaskTrigger -AtStartup
    $principal=New-ScheduledTaskPrincipal -UserId $user -LogonType ServiceAccount -RunLevel Highest
} else {
    $startup=New-ScheduledTaskTrigger -AtLogOn -User $user
    $principal=New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Limited
}
Register-ScheduledTask -TaskName $name -Action $action -Trigger @($startup,$repeat) -Settings $settings -Principal $principal -Force | Out-Null
