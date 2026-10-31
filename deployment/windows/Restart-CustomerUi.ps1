[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Directory,
    [Parameter(Mandatory=$true)][string]$Executable,
    [ValidateSet('restart','remove')][string]$Operation='restart'
)
$ErrorActionPreference='Stop'

function Get-CustomerUiTaskName([string]$Prefix,[string]$Sid) {
    $hash=[Security.Cryptography.SHA256]::Create()
    try { $suffix=([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Sid)))).Replace('-','').Substring(0,16) } finally { $hash.Dispose() }
    return $Prefix+$suffix
}

function Get-CustomerUiAction([string]$Path,[string]$Sid,[string]$TaskName) {
    # These values come from the verified installation and OS token, never branding.
    $wrapper=@'
$ErrorActionPreference='Stop'
if ([Security.Principal.WindowsIdentity]::GetCurrent().User.Value -cne '__SID__') { throw 'Customer UI account differs.' }
foreach ($name in @('SWAN_STATE_DIR','SWAN_TECHNICIAN_TOKEN','SWAN_SESSION_GRANT','SWAN_SESSION_PROOF_KEY')) { [Environment]::SetEnvironmentVariable($name,$null,'Process') }
Start-Process -FilePath '__EXE__' -WorkingDirectory '__PARENT__' | Out-Null
$scheduler=New-Object -ComObject Schedule.Service
$scheduler.Connect()
$scheduler.GetFolder('\').DeleteTask('__TASK__',0)
'@
    foreach ($entry in @(@('__SID__',$Sid),@('__EXE__',$Path),@('__PARENT__',[IO.Path]::GetDirectoryName($Path)),@('__TASK__',$TaskName))) {
        $wrapper=$wrapper.Replace($entry[0],$entry[1].Replace("'","''"))
    }
    $encoded=[Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($wrapper))
    return New-ScheduledTaskAction -Execute ($env:SystemRoot+'/System32/WindowsPowerShell/v1.0/powershell.exe') -Argument ('-NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -EncodedCommand '+$encoded)
}

function Assert-CustomerUiTask($Task,$Action,[string]$Sid,$Registered) {
    $principal=[string]$Task.Principal.UserId
    $actual=if ($principal -match '^S-\d-') { ([Security.Principal.SecurityIdentifier]::new($principal)).Value } else { ([Security.Principal.NTAccount]::new($principal)).Translate([Security.Principal.SecurityIdentifier]).Value }
    if ($Task.TaskPath -cne '\' -or $Task.Actions.Count -ne 1 -or $actual -cne $Sid -or $Task.Principal.LogonType -ne 'Interactive' -or $Task.Principal.RunLevel -ne 'Limited' -or [IO.Path]::GetFullPath($Task.Actions[0].Execute) -ine [IO.Path]::GetFullPath($Action.Execute) -or $Task.Actions[0].Arguments -cne $Action.Arguments) { throw 'Customer UI task belongs to another installation.' }
    $security=[Security.AccessControl.RawSecurityDescriptor]::new($Registered.GetSecurityDescriptor(7))
    if ($security.Owner.Value -cne 'S-1-5-18' -or $security.DiscretionaryAcl.Count -ne 3) { throw 'Customer UI task ownership differs.' }
    $seen=@{}
    foreach ($ace in $security.DiscretionaryAcl) {
        if ($ace.AceType -ne [Security.AccessControl.AceType]::AccessAllowed -or $ace.AceFlags -ne [Security.AccessControl.AceFlags]::None) { throw 'Unexpected customer UI task permission.' }
        $account=$ace.SecurityIdentifier.Value
        if ($seen.ContainsKey($account)) { throw 'Duplicate customer UI task permission.' }
        $seen[$account]=$true
        if ($account -eq $Sid) {
            # File generic read/execute and delete; no action write or ACL control.
            if ($ace.AccessMask -ne 0x1300A9) { throw 'Customer UI task permits unexpected user changes.' }
        } elseif ($account -in @('S-1-5-18','S-1-5-32-544')) {
            if ($ace.AccessMask -ne 0x1F01FF) { throw 'Customer UI task system permissions differ.' }
        } else { throw 'Unexpected customer UI task account.' }
    }
    if (!$seen.ContainsKey($Sid) -or !$seen.ContainsKey('S-1-5-18') -or !$seen.ContainsKey('S-1-5-32-544')) { throw 'Customer UI task permissions are incomplete.' }
}

function Stop-OwnedCustomerUi($Processes,[string]$Path) {
    if (-not ('SwanCustomerUiProcess' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.Text;
public static class SwanCustomerUiProcess {
    [DllImport("kernel32.dll", SetLastError=true)] static extern IntPtr OpenProcess(uint access, bool inherit, int pid);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool GetProcessTimes(IntPtr process, out long created, out long exited, out long kernel, out long user);
    [DllImport("kernel32.dll", EntryPoint="QueryFullProcessImageNameW", CharSet=CharSet.Unicode, SetLastError=true)] static extern bool QueryImage(IntPtr process, uint flags, StringBuilder path, ref int length);
    [DllImport("advapi32.dll", SetLastError=true)] static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool TerminateProcess(IntPtr process, uint code);
    [DllImport("kernel32.dll", SetLastError=true)] static extern uint WaitForSingleObject(IntPtr process, uint milliseconds);
    public static bool Stop(int pid, long expectedCreated, string expectedPath, string expectedSid) {
        IntPtr process=OpenProcess(0x1000 | 0x0001 | 0x100000, false, pid);
        if (process==IntPtr.Zero) {
            int error=Marshal.GetLastWin32Error();
            if (error==87 || error==1168) return false;
            throw new Win32Exception(error);
        }
        try {
            long created, exited, kernel, user;
            if (!GetProcessTimes(process,out created,out exited,out kernel,out user)) throw new Win32Exception(Marshal.GetLastWin32Error());
            // CIM creation times have microsecond precision; kernel FILETIME is finer.
            if (created/10 != expectedCreated/10) throw new InvalidOperationException("Customer UI process creation differs.");
            int length=32768; var path=new StringBuilder(length);
            if (!QueryImage(process,0,path,ref length)) throw new Win32Exception(Marshal.GetLastWin32Error());
            if (!String.Equals(path.ToString(),expectedPath,StringComparison.OrdinalIgnoreCase)) throw new InvalidOperationException("Customer UI executable differs.");
            IntPtr token;
            if (!OpenProcessToken(process,8,out token)) throw new Win32Exception(Marshal.GetLastWin32Error());
            try { using(var identity=new WindowsIdentity(token)) {
                if (identity.User==null || identity.User.Value!=expectedSid || identity.IsSystem) throw new InvalidOperationException("Customer UI account differs.");
            }} finally { CloseHandle(token); }
            if (WaitForSingleObject(process,0)==0) return false;
            if (!TerminateProcess(process,0)) throw new Win32Exception(Marshal.GetLastWin32Error());
            if (WaitForSingleObject(process,5000)!=0) throw new InvalidOperationException("Previous customer UI has not exited.");
            return true;
        } finally { CloseHandle(process); }
    }
}
'@
    }
    foreach ($process in $Processes) {
        [void][SwanCustomerUiProcess]::Stop($process.Pid,$process.Created.ToFileTimeUtc(),$Path,$process.Sid)
    }
}

function Read-CustomerUiRecoveryTasks([string]$StateDirectory,[string]$Path) {
    if (-not [Security.Principal.WindowsIdentity]::GetCurrent().IsSystem) { throw 'Customer UI recovery requires SYSTEM.' }
    $directoryPath=[IO.Path]::GetFullPath($StateDirectory).TrimEnd('\')
    $exe=[IO.Path]::GetFullPath($Path)
    if ([IO.Path]::GetFileName($exe) -cne 'Swan Remote Support.exe') { throw 'Unexpected installed customer executable.' }
    $hash=[Security.Cryptography.SHA256]::Create()
    try { $identifier=([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($directoryPath.ToLowerInvariant())))).Replace('-','').Substring(0,16) } finally { $hash.Dispose() }
    $prefix='Swan Customer UI '+$identifier+' '
    $scheduler=New-Object -ComObject Schedule.Service;$scheduler.Connect();$folder=$scheduler.GetFolder('\')
    $pending=@{}
    foreach ($task in @(Get-ScheduledTask | Where-Object { $_.TaskName.StartsWith($prefix,[StringComparison]::Ordinal) })) {
        $principal=[string]$task.Principal.UserId
        $sid=if ($principal -match '^S-\d-') { ([Security.Principal.SecurityIdentifier]::new($principal)).Value } else { ([Security.Principal.NTAccount]::new($principal)).Translate([Security.Principal.SecurityIdentifier]).Value }
        if ($sid -eq 'S-1-5-18' -or $task.TaskName -cne (Get-CustomerUiTaskName $prefix $sid)) { throw 'Unexpected customer UI recovery task.' }
        $action=Get-CustomerUiAction $exe $sid $task.TaskName
        Assert-CustomerUiTask $task $action $sid ($folder.GetTask($task.TaskName))
        if ($pending.ContainsKey($sid)) { throw 'Ambiguous customer UI recovery task.' }
        $pending[$sid]=$task
    }
    return @{Prefix=$prefix;Folder=$folder;Tasks=$pending}
}

function Remove-CustomerUiRecoveryTasks([string]$StateDirectory,[string]$Path) {
    if (-not (Test-Path -LiteralPath (Join-Path $StateDirectory 'pending-uninstall'))) { throw 'Customer UI cleanup requires explicit uninstall cancellation.' }
    $context=Read-CustomerUiRecoveryTasks $StateDirectory $Path
    foreach ($task in $context.Tasks.Values) { $context.Folder.DeleteTask($task.TaskName,0) }
}

function Restart-VerifiedCustomerUi([string]$StateDirectory,[string]$Path) {
    if (-not [Security.Principal.WindowsIdentity]::GetCurrent().IsSystem) { throw 'Customer UI update restart requires SYSTEM.' }
    $directoryPath=[IO.Path]::GetFullPath($StateDirectory).TrimEnd('\')
    $exe=[IO.Path]::GetFullPath($Path)
    if ([IO.Path]::GetFileName($exe) -cne 'Swan Remote Support.exe' -or (Get-Item -LiteralPath $exe).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unexpected installed customer executable.' }
    if (Test-Path -LiteralPath (Join-Path $directoryPath 'pending-uninstall')) { throw 'Explicit uninstall cancels customer UI restoration.' }
    # The caller verified this exact signed release while holding session
    # exclusion. Do not restart a newly opened post-commit UI on recovery.
    $committed=(Get-Item -LiteralPath (Join-Path $directoryPath 'installed-release.json')).LastWriteTimeUtc
    $old=@();$fresh=@{}
    foreach ($process in @(Get-CimInstance Win32_Process -Filter "Name='Swan Remote Support.exe'" -OperationTimeoutSec 30)) {
        if ($process.SessionId -eq 0 -or $process.ExecutablePath -ine $exe) { continue }
        $owner=Invoke-CimMethod -InputObject $process -MethodName GetOwnerSid -OperationTimeoutSec 30
        if ($owner.ReturnValue -ne 0) { throw 'Cannot determine customer UI account.' }
        $sid=([Security.Principal.SecurityIdentifier]::new($owner.Sid)).Value
        if ($sid -eq 'S-1-5-18') { continue }
        $created=$process.CreationDate.ToUniversalTime()
        if ($created -ge $committed) { $fresh[$sid]=$true;continue }
        $old+=@{Pid=[int]$process.ProcessId;Sid=$sid;Created=$created}
    }
    $context=Read-CustomerUiRecoveryTasks $directoryPath $exe
    $prefix=$context.Prefix;$folder=$context.Folder;$pending=$context.Tasks
    foreach ($sid in @($old | ForEach-Object Sid | Sort-Object -Unique)) {
        if ($fresh.ContainsKey($sid) -or $pending.ContainsKey($sid)) { continue }
        $name=Get-CustomerUiTaskName $prefix $sid
        $action=Get-CustomerUiAction $exe $sid $name
        $principal=New-ScheduledTaskPrincipal -UserId $sid -LogonType Interactive -RunLevel Limited
        $trigger=New-ScheduledTaskTrigger -AtLogOn -User $sid
        $settings=New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 2) -MultipleInstances IgnoreNew -Priority 4 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
        $task=Register-ScheduledTask -TaskName $name -Action $action -Principal $principal -Trigger $trigger -Settings $settings
        # The UI can remove its completed task, but cannot replace its action or
        # change its ACL. Keep SYSTEM/admin control and do not add a principal ACE.
        $registered=$folder.GetTask($name)
        $registered.SetSecurityDescriptor(('O:SYG:SYD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FRFXSD;;;'+$sid+')'),0x10)
        Assert-CustomerUiTask $task $action $sid $registered
        $pending[$sid]=$task
    }
    # Recovery tasks are durable before any old process is stopped. Their action
    # runs only as that existing user, with no password or elevated token.
    if (Test-Path -LiteralPath (Join-Path $directoryPath 'pending-uninstall')) { throw 'Explicit uninstall cancels customer UI restoration.' }
    Stop-OwnedCustomerUi $old $exe
    $requested=@();$deferred=@()
    foreach ($sid in @($pending.Keys)) {
        $task=$pending[$sid]
        if ($fresh.ContainsKey($sid)) { $folder.DeleteTask($task.TaskName,0);continue }
        try { $task | Start-ScheduledTask;$requested+=$sid }
        catch { Write-Warning ('Customer UI restart deferred to this user next logon: '+$_.Exception.Message);$deferred+=$sid }
    }
    @{old_processes_closed=@($old).Count;user_restarts_requested=$requested;user_restarts_deferred_to_logon=$deferred;credentials_used=$false;user_actions_elevated=$false} | ConvertTo-Json -Compress
}

if ($MyInvocation.InvocationName -ne '.') {
    if ($Operation -eq 'remove') { Remove-CustomerUiRecoveryTasks $Directory $Executable }
    else { Restart-VerifiedCustomerUi $Directory $Executable }
}
