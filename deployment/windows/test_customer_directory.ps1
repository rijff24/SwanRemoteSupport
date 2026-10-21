param([switch]$RequireAdministratorCases)
$ErrorActionPreference='Stop'
$script=Join-Path $PSScriptRoot 'Prepare-CustomerDirectory.ps1'
$tokens=$null;$errors=$null
$ast=[System.Management.Automation.Language.Parser]::ParseFile($script,[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Customer directory helper failed parsing.'}
$guard=$ast.Find({param($node)$node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Assert-CustomerAcl'},$true)
if($null -eq $guard){throw 'Customer ACL guard missing.'}
. ([ScriptBlock]::Create($guard.Extent.Text))
function New-FixtureAcl([string]$owner){
    $acl=[Security.AccessControl.DirectorySecurity]::new()
    $acl.SetOwner([Security.Principal.SecurityIdentifier]::new($owner));$acl.SetAccessRuleProtection($true,$false)
    foreach($sid in @('S-1-5-18','S-1-5-32-544')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))}
    return $acl
}
Assert-CustomerAcl (New-FixtureAcl 'S-1-5-18')
Assert-CustomerAcl (New-FixtureAcl 'S-1-5-32-544')
foreach($sid in @('S-1-5-32-545','S-1-1-0')){
    $acl=New-FixtureAcl 'S-1-5-18'
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'ReadAndExecute,Write','ContainerInherit,ObjectInherit','None','Allow'))
    $rejected=$false;try{Assert-CustomerAcl $acl}catch{$rejected=$true}
    if(-not $rejected){throw 'Unprivileged state permissions accepted.'}
}
$rejected=$false;try{Assert-CustomerAcl (New-FixtureAcl 'S-1-5-32-545')}catch{$rejected=$true}
if(-not $rejected){throw 'Untrusted owner accepted.'}
$root=Join-Path ([IO.Path]::GetTempPath()) ('swan-directory-security-'+[Guid]::NewGuid().ToString('N'))
$native=Join-Path ([Environment]::GetFolderPath('System')) 'WindowsPowerShell/v1.0/powershell.exe'
function Invoke-DirectoryHelper([string]$Directory) {
    # Windows PowerShell converts redirected native stderr into terminating
    # errors under Stop. Observe the child exit code for intentional rejection.
    $start=[Diagnostics.ProcessStartInfo]::new();$start.FileName=$native
    $start.Arguments='-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "'+$script+'" -Directory "'+$Directory+'" -Scope custom'
    $start.UseShellExecute=$false;$start.CreateNoWindow=$true
    $start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
    $process=[Diagnostics.Process]::Start($start)
    try {
        $output=$process.StandardOutput.ReadToEndAsync();$errors=$process.StandardError.ReadToEndAsync()
        if(-not $process.WaitForExit(30000)){$process.Kill();$process.WaitForExit();throw 'Customer directory fixture exceeded its deadline.'}
        $null=$output.GetAwaiter().GetResult();$null=$errors.GetAwaiter().GetResult()
        return $process.ExitCode
    } finally {$process.Dispose()}
}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$admin=([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if($RequireAdministratorCases -and -not $admin){throw 'Administrator filesystem cases are required for this run.'}
if($admin){
    try{
        if((Invoke-DirectoryHelper $root) -ne 0){throw 'Private directory creation failed.'}
        Assert-CustomerAcl (Get-Acl -LiteralPath $root)
        if((Invoke-DirectoryHelper $root) -ne 0){throw 'Trusted existing directory rejected.'}
        $child=Join-Path $root 'untrusted.txt';[IO.File]::WriteAllText($child,'untrusted fixture: preserve')
        $fileAcl=[Security.AccessControl.FileSecurity]::new();$fileAcl.SetOwner($identity.User);$fileAcl.SetAccessRuleProtection($true,$false)
        $fileAcl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($identity.User,'FullControl','Allow'))
        Set-Acl -LiteralPath $child -AclObject $fileAcl
        $before=(Get-Acl -LiteralPath $child).Sddl
        if((Invoke-DirectoryHelper $root) -eq 0 -or (Get-Acl -LiteralPath $child).Sddl -cne $before -or [IO.File]::ReadAllText($child) -cne 'untrusted fixture: preserve'){throw 'Untrusted existing file was adopted or changed.'}
        Remove-Item -LiteralPath $child
        $broad=Get-Acl -LiteralPath $root
        $broad.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-32-545'),'Write','ContainerInherit,ObjectInherit','None','Allow'))
        Set-Acl -LiteralPath $root -AclObject $broad;$before=(Get-Acl -LiteralPath $root).Sddl
        if((Invoke-DirectoryHelper $root) -eq 0 -or (Get-Acl -LiteralPath $root).Sddl -cne $before){throw 'Writable existing state was silently adopted.'}
    }finally{
        $child=Join-Path $root 'untrusted.txt';if(Test-Path -LiteralPath $child){Remove-Item -LiteralPath $child}
        if(Test-Path -LiteralPath $root){Remove-Item -LiteralPath $root}
    }
}
Write-Output ('Customer directory ACL checks passed; administrator filesystem cases executed: '+$admin)
