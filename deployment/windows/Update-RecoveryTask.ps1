[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][ValidateSet('register','remove')][string]$Operation,
    [Parameter(Mandatory=$true)][string]$Directory,
    [Parameter(Mandatory=$true)][ValidateSet('customer','technician')][string]$Edition,
    [Parameter(Mandatory=$true)][UInt64]$Sequence,
    [switch]$InspectOnly
)
$ErrorActionPreference='Stop'
function Assert-RecoveryTask($Task,[string]$Helper,[string]$AccountSid) {
    if ($Task.TaskPath -cne '\' -or $Task.Actions.Count -ne 1 -or [IO.Path]::GetFullPath($Task.Actions[0].Execute) -ine $Helper -or $Task.Actions[0].Arguments -cne 'recover-update-task') { throw 'Update recovery task belongs to another installation.' }
    # Task Scheduler can return a short account name after registration by SID.
    # Resolve that identity; never authorize by display-name or suffix matching.
    try {
        $principal=[string]$Task.Principal.UserId
        $actualSid=if ($principal -match '^S-\d-') { ([Security.Principal.SecurityIdentifier]::new($principal)).Value } else { ([Security.Principal.NTAccount]::new($principal)).Translate([Security.Principal.SecurityIdentifier]).Value }
        $expectedSid=([Security.Principal.SecurityIdentifier]::new($AccountSid)).Value
    } catch { throw 'Update recovery task account cannot be resolved.' }
    if ($actualSid -cne $expectedSid) { throw 'Update recovery task belongs to another installation.' }
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
    Assert-RecoveryTask $task $helper $user
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
$settings=New-ScheduledTaskSettingsSet -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Hours 2) -MultipleInstances IgnoreNew -Priority 4
if ($Edition -eq 'customer') {
    $startup=New-ScheduledTaskTrigger -AtStartup
    $principal=New-ScheduledTaskPrincipal -UserId $user -LogonType ServiceAccount -RunLevel Highest
} else {
    $startup=New-ScheduledTaskTrigger -AtLogOn -User $user
    $principal=New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Limited
}
Register-ScheduledTask -TaskName $name -Action $action -Trigger @($startup,$repeat) -Settings $settings -Principal $principal -Force | Out-Null
