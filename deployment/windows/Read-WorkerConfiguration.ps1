[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Path,[ValidateSet('installed','input')][string]$Scope='installed',
      [ValidateSet('console','pipeline')][string]$Output='console')
$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
$root=Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanInstallerWorker'
$expected=Join-Path $root 'worker.json'
if($Scope -eq 'installed') {
 if([IO.Path]::GetFullPath($Path) -ine [IO.Path]::GetFullPath($expected)){throw 'Use the canonical private worker configuration.'}
 $allowed=@('S-1-5-18','S-1-5-32-544')
} else {
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent()
 if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Private worker configuration installation requires administrator rights.'}
 $expected=[IO.Path]::GetFullPath($Path)
 if($expected.StartsWith('\\')){throw 'Use a private local worker configuration file.'}
 $root=Split-Path -Parent $expected
 $allowed=@('S-1-5-18','S-1-5-32-544',$identity.User.Value)|Select-Object -Unique
}
foreach($item in @((Split-Path -Parent $root),$root,$expected)){
 if((Get-Item -LiteralPath $item -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Worker configuration paths must not be reparse points.'}
}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public static class SwanWorkerConfigurationHandle {
    [DllImport("kernel32.dll", EntryPoint="CreateFileW", CharSet=CharSet.Unicode, SetLastError=true)]
    private static extern SafeFileHandle CreateFile(string path,uint access,uint share,IntPtr security,uint creation,uint flags,IntPtr template);
    public static SafeFileHandle Open(string path,bool directory) {
        return CreateFile(path,0x80000000,1,IntPtr.Zero,3,0x00200000|(directory?0x02000000u:0u),IntPtr.Zero);
    }
}
'@
$directory=[SwanWorkerConfigurationHandle]::Open($root,$true)
if($directory.IsInvalid){$directory.Dispose();throw 'Private worker directory is unavailable or in use.'}
$fileHandle=$null;$stream=$null;$reader=$null
try {
 $fileHandle=[SwanWorkerConfigurationHandle]::Open($expected,$false)
 if($fileHandle.IsInvalid){throw 'Private worker configuration is unavailable or in use.'}
 foreach($item in @($root,$expected)){
  $info=Get-Item -LiteralPath $item -Force
  if($info.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Worker configuration path changed to a reparse point.'}
  $acl=Get-Acl -LiteralPath $item
  if($Scope -eq 'installed' -and !$acl.AreAccessRulesProtected){throw 'Installed worker configuration must not inherit public permissions.'}
  if($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -notin $allowed){throw 'Private worker configuration has an untrusted owner.'}
  $rules=$acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])
  foreach($rule in $rules){if($rule.AccessControlType -ne 'Allow' -or $rule.IdentityReference.Value -notin $allowed){throw 'Private worker configuration grants unexpected access. Revoke credentials previously stored in a public location.'}}
  foreach($sid in @('S-1-5-18','S-1-5-32-544')){if(@($rules|Where-Object {$_.IdentityReference.Value -eq $sid -and ($_.FileSystemRights -band [Security.AccessControl.FileSystemRights]::FullControl) -eq [Security.AccessControl.FileSystemRights]::FullControl}).Count -eq 0){throw 'Private worker configuration needs SYSTEM and Administrators access.'}}
 }
 if(!(Get-Item -LiteralPath $root).PSIsContainer -or (Get-Item -LiteralPath $expected).PSIsContainer){throw 'Invalid worker configuration object type.'}
 $stream=[IO.FileStream]::new($fileHandle,[IO.FileAccess]::Read)
 if($stream.Length -lt 1 -or $stream.Length -gt 16384){throw 'Invalid private worker configuration size.'}
 $reader=[IO.StreamReader]::new($stream,[Text.UTF8Encoding]::new($false,$true),$true)
 $content=$reader.ReadToEnd()
 if($Output -eq 'pipeline'){Write-Output $content}else{
  [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
  [Console]::Write($content)
 }
} finally {
 if($reader){$reader.Dispose()}elseif($stream){$stream.Dispose()}
 if($fileHandle){$fileHandle.Dispose()}
 $directory.Dispose()
}
