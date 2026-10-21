[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Directory,[ValidateSet('canonical','custom')][string]$Scope='canonical')
$ErrorActionPreference='Stop'
function Assert-CustomerAcl {
    param([Parameter(Mandatory=$true)]$Acl)
    $allowed=@('S-1-5-18','S-1-5-32-544')
    if ($Acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -notin $allowed) { throw 'Customer state has an untrusted owner. Preserve and investigate it before setup.' }
    $rules=$Acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])
    foreach($rule in $rules) {
        if ($rule.AccessControlType -ne [Security.AccessControl.AccessControlType]::Allow -or $rule.IdentityReference.Value -notin $allowed) { throw 'Customer state permissions are not restricted to SYSTEM and Administrators.' }
    }
    foreach($sid in $allowed) {
        if (@($rules|Where-Object {$_.IdentityReference.Value -eq $sid -and ($_.FileSystemRights -band [Security.AccessControl.FileSystemRights]::FullControl) -eq [Security.AccessControl.FileSystemRights]::FullControl}).Count -eq 0) { throw 'Customer state requires full control for SYSTEM and Administrators.' }
    }
}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Customer state preparation requires administrator rights.' }
$Directory=[IO.Path]::GetFullPath($Directory).TrimEnd('\')
if ($Directory -eq [IO.Path]::GetPathRoot($Directory).TrimEnd('\')) { throw 'A volume root is not a customer state directory.' }
$expected=Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanRemoteSupport'
if ($Scope -eq 'canonical' -and $Directory -ine [IO.Path]::GetFullPath($expected)) { throw 'The customer MSI requires the canonical state directory.' }
$descriptor=[Security.AccessControl.DirectorySecurity]::new()
$descriptor.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
$descriptor.SetAccessRuleProtection($true,$false)
foreach($sid in @('S-1-5-18','S-1-5-32-544')) {
    $descriptor.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))
}
if(Test-Path -LiteralPath $Directory){
    $existing=Get-Item -LiteralPath $Directory -Force
    if(-not $existing.PSIsContainer -or ($existing.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Customer state directory is unsafe.'}
    Assert-CustomerAcl (Get-Acl -LiteralPath $Directory)
}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public static class SwanCustomerDirectoryLock {
    [StructLayout(LayoutKind.Sequential)]
    private struct SecurityAttributes { public int Length; public IntPtr Descriptor; [MarshalAs(UnmanagedType.Bool)] public bool Inherit; }
    [DllImport("kernel32.dll", EntryPoint="CreateDirectoryW", CharSet=CharSet.Unicode, SetLastError=true)]
    private static extern bool CreateDirectory(string path,ref SecurityAttributes security);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    private static extern SafeFileHandle CreateFile(string path,uint access,uint share,IntPtr security,uint creation,uint flags,IntPtr template);
    public static int Create(string path,byte[] descriptor) {
        GCHandle pinned=GCHandle.Alloc(descriptor,GCHandleType.Pinned);
        try {
            SecurityAttributes security=new SecurityAttributes { Length=Marshal.SizeOf(typeof(SecurityAttributes)), Descriptor=pinned.AddrOfPinnedObject(), Inherit=false };
            return CreateDirectory(path,ref security) ? 0 : Marshal.GetLastWin32Error();
        } finally { pinned.Free(); }
    }
    public static SafeFileHandle Open(string path) { return CreateFile(path,0x80000000,1,IntPtr.Zero,3,0x02200000,IntPtr.Zero); }
}
'@
# CreateDirectoryW applies security atomically only to a new directory. An
# existing object is inspected, never adopted by changing its owner or ACL.
$created=[SwanCustomerDirectoryLock]::Create($Directory,$descriptor.GetSecurityDescriptorBinaryForm())
if($created -notin @(0,183)){throw ([ComponentModel.Win32Exception]::new($created))}
# Sharing only read access rejects a pre-existing writable directory handle;
# do not race a user who opened the directory before permissions were secured.
$handle=[SwanCustomerDirectoryLock]::Open($Directory)
if ($handle.IsInvalid) { $handle.Dispose();throw 'Customer state directory is in use or cannot be inspected safely.' }
try {
    $pending=[Collections.Generic.Stack[string]]::new();$pending.Push($Directory);$count=0
    while($pending.Count) {
        $path=$pending.Pop();$item=Get-Item -LiteralPath $path -Force
        if($path -eq $Directory -and -not $item.PSIsContainer){throw 'Customer state must be a directory.'}
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Customer state must not contain reparse points.' }
        Assert-CustomerAcl (Get-Acl -LiteralPath $path)
        $count++;if($count -gt 10000){throw 'Customer state inspection limit exceeded.'}
        if($item.PSIsContainer){foreach($child in @(Get-ChildItem -LiteralPath $path -Force)){$pending.Push($child.FullName)}}
    }
} finally { $handle.Dispose() }
