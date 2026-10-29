[CmdletBinding()]
param([Parameter(Mandatory)][string]$Executable,
      [Parameter(Mandatory)][string]$ConfigurationPath,
      [Parameter(Mandatory)][ValidatePattern('^[A-Fa-f0-9]{40}$')][string]$PublisherThumbprint)
$ErrorActionPreference='Stop'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Worker installation requires administrator rights.'}
$source=(Resolve-Path -LiteralPath $Executable).Path
if((Get-Item -LiteralPath $source).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Worker executable must be a regular file.'}
$signature=Get-AuthenticodeSignature -LiteralPath $source
if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ine $PublisherThumbprint){throw 'Worker signature does not match the trusted publisher.'}
try {
 $privateInput=& (Join-Path $PSScriptRoot 'Read-WorkerConfiguration.ps1') -Path $ConfigurationPath -Scope input -Output pipeline
 $config=$privateInput|ConvertFrom-Json
} catch {throw 'Worker configuration must be valid JSON in a private local administrator directory. Revoke any credential previously stored publicly.'}
$allowed=@('management_url','worker_token','release_public_key','profile_public_key')
if(@($config.PSObject.Properties.Name).Count -ne $allowed.Count -or @($config.PSObject.Properties.Name|Where-Object {$_ -notin $allowed}).Count){throw 'Worker configuration requires exactly the documented fields.'}
foreach($name in $allowed){if($config.$name -isnot [string] -or [string]::IsNullOrWhiteSpace($config.$name) -or $config.$name.Contains("`n") -or $config.$name.Contains("`r") -or $config.$name.Contains([char]0)){throw 'Invalid worker configuration value.'}}
$url=$null
if(-not [Uri]::TryCreate($config.management_url,[UriKind]::Absolute,[ref]$url) -or $url.Scheme -cne 'https' -or -not $url.Host -or $url.UserInfo -or $url.Fragment -or $url.Query){throw 'Worker requires a company HTTPS origin without credentials, query or fragment.'}
foreach($name in @('release_public_key','profile_public_key')){if([Convert]::FromBase64String($config.$name).Length -ne 32){throw 'Expected a 32-byte public trust key.'}}
if($config.worker_token -cnotmatch '^[\x21-\x7E]{32,256}$'){throw 'Invalid worker credential format.'}
$install=Join-Path $env:ProgramFiles 'Swan Installer Worker'
$data=Join-Path $env:ProgramData 'SwanInstallerWorker'
if((Get-Service -ErrorAction Stop|Where-Object {$_.Name -ceq 'SwanInstallerWorker'}) -or (Test-Path -LiteralPath $install) -or (Test-Path -LiteralPath $data)){throw 'Existing worker installation must be inspected; replacement is not automatic.'}
foreach($parent in @($env:ProgramFiles,$env:ProgramData)){if((Get-Item -LiteralPath $parent -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Worker installation parent cannot be a reparse point.'}}
New-Item -ItemType Directory -Path $install|Out-Null
foreach($directory in @($install)){
 & icacls.exe $directory '/inheritance:r' '/grant:r' '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F'|Out-Null
 if($LASTEXITCODE -ne 0){throw 'Cannot protect worker installation.'}
}
$destination=Join-Path $install 'swan-worker.exe'
Copy-Item -LiteralPath $source -Destination $destination
if((Get-FileHash -LiteralPath $source).Hash -cne (Get-FileHash -LiteralPath $destination).Hash){throw 'Worker executable changed during installation.'}
$installedSignature=Get-AuthenticodeSignature -LiteralPath $destination
if($installedSignature.Status -ne 'Valid' -or $installedSignature.SignerCertificate.Thumbprint -ine $PublisherThumbprint){throw 'Installed worker signature does not match the trusted publisher.'}
function Invoke-WorkerSetupCommand([string]$Mode,[string]$Argument,[string]$PrivateConfiguration) {
 $start=[Diagnostics.ProcessStartInfo]::new()
 $start.FileName=$destination;$start.Arguments=$Mode
 if($Argument){$start.Arguments+=' "'+$Argument+'"'}
 $start.UseShellExecute=$false;$start.CreateNoWindow=$true
 foreach($name in @('SWAN_WORKER_CONFIG','SWAN_WORKER_TOKEN','SWAN_MANAGEMENT_URL','SWAN_RELEASE_PUBLIC_KEY','SWAN_PROFILE_PUBLIC_KEY','SWAN_ARTIFACT_DIR')){[void]$start.EnvironmentVariables.Remove($name)}
 if($PrivateConfiguration){$start.EnvironmentVariables['SWAN_WORKER_CONFIG']=$PrivateConfiguration}
 $process=[Diagnostics.Process]::Start($start)
 try {
  if(!$process.WaitForExit(300000)){$process.Kill();$process.WaitForExit();throw 'Worker preparation timed out. Preserve existing files before retrying.'}
  if($process.ExitCode -ne 0){throw 'Worker preparation failed. Use the matching worker release and preserve existing files.'}
 }finally{$process.Dispose()}
}
# The verified worker creates the state root with its private descriptor in the
# same operation. An existing or concurrently created root is never adopted.
Invoke-WorkerSetupCommand 'prepare-worker-directory' $data ''
$configurationFile=Join-Path $data 'worker.json'
$canonical=[ordered]@{}
foreach($name in $allowed){$canonical[$name]=$config.$name}
$bytes=[Text.UTF8Encoding]::new($false).GetBytes(($canonical|ConvertTo-Json -Compress))
$file=[IO.FileStream]::new($configurationFile,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
try {
 $fileAcl=[Security.AccessControl.FileSecurity]::new()
 $fileAcl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
 $fileAcl.SetAccessRuleProtection($true,$false)
 foreach($sid in @('S-1-5-18','S-1-5-32-544')){$fileAcl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','Allow'))}
 Set-Acl -LiteralPath $configurationFile -AclObject $fileAcl
 $secured=Get-Acl -LiteralPath $configurationFile
 if(!$secured.AreAccessRulesProtected -or $secured.GetOwner([Security.Principal.SecurityIdentifier]).Value -cne 'S-1-5-32-544'){throw 'Worker configuration file is not privately owned.'}
 foreach($rule in $secured.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])){if($rule.AccessControlType -ne 'Allow' -or $rule.IdentityReference.Value -notin @('S-1-5-18','S-1-5-32-544')){throw 'Worker configuration file grants unexpected access; no credential written.'}}
 $file.Write($bytes,0,$bytes.Length);$file.Flush($true)
}finally{$file.Dispose()}
Invoke-WorkerSetupCommand 'check-configuration' '' $configurationFile
$artifacts=Join-Path $data 'artifacts';New-Item -ItemType Directory -Path $artifacts|Out-Null
$created=$false
try{
 New-Service -Name SwanInstallerWorker -DisplayName 'Swan Remote Support Installer Worker' -BinaryPathName ('"'+$destination+'" --service') -StartupType Automatic|Out-Null
 $created=$true
 $registry='HKLM:\SYSTEM\CurrentControlSet\Services\SwanInstallerWorker'
 # Registry permissions may gain app-container read ACEs on Windows. Persist
 # only the public path; the service reads its credential from the private file.
 $acl=[Security.AccessControl.RegistrySecurity]::new()
 $acl.SetAccessRuleProtection($true,$false)
 foreach($sid in @('S-1-5-18','S-1-5-32-544')){
  $principal=[Security.Principal.SecurityIdentifier]::new($sid)
  $rule=[Security.AccessControl.RegistryAccessRule]::new($principal,[Security.AccessControl.RegistryRights]::FullControl,[Security.AccessControl.InheritanceFlags]::ContainerInherit,[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow)
  $acl.AddAccessRule($rule)
 }
 Set-Acl -LiteralPath $registry -AclObject $acl
 $environment=@("SWAN_WORKER_CONFIG=$configurationFile")
 New-ItemProperty -LiteralPath $registry -Name Environment -PropertyType MultiString -Value $environment|Out-Null
 & sc.exe failure SwanInstallerWorker reset= 86400 actions= restart/60000/restart/60000/restart/300000|Out-Null
 if($LASTEXITCODE -ne 0){throw 'Cannot configure worker recovery.'}
 # The worker reports Stopped with a nonzero exit code for fatal runtime errors.
 & sc.exe failureflag SwanInstallerWorker 1|Out-Null
 if($LASTEXITCODE -ne 0){throw 'Cannot enable recovery for reported worker failures.'}
 Start-Service SwanInstallerWorker
 (Get-Service SwanInstallerWorker).WaitForStatus([ServiceProcess.ServiceControllerStatus]::Running,[TimeSpan]::FromSeconds(30))
 @{schema=2;service='SwanInstallerWorker';executable_sha256=(Get-FileHash $destination).Hash;publisher_thumbprint=$PublisherThumbprint;configuration_path=$configurationFile}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $data 'installation.json')
}catch{
 if($created){Stop-Service SwanInstallerWorker -ErrorAction Continue;& sc.exe delete SwanInstallerWorker|Out-Null}
 Write-Warning 'Worker installation failed; protected files remain for inspection. No automatic replacement will occur.'
 throw
}
Write-Host 'Worker service started. Verify an authenticated completed build and startup after reboot before publication.'
