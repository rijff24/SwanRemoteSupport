[CmdletBinding(SupportsShouldProcess=$true)]
param([Parameter(Mandatory=$true)][string]$Executable,
      [Parameter(Mandatory=$true)][string]$PreviousConfigurationPath,
      [Parameter(Mandatory=$true)][string]$ConfigurationPath,
      [Parameter(Mandatory=$true)][ValidatePattern('^[A-Fa-f0-9]{40}$')][string]$PublisherThumbprint)
$ErrorActionPreference='Stop'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Worker migration requires administrator rights.'}
$install=[IO.Path]::GetFullPath((Join-Path $env:ProgramFiles 'Swan Installer Worker'))
$data=[IO.Path]::GetFullPath((Join-Path $env:ProgramData 'SwanInstallerWorker'))
$destination=Join-Path $install 'swan-worker.exe';$receiptPath=Join-Path $data 'installation.json'
$configurationFile=Join-Path $data 'worker.json';$journal=Join-Path $data 'credential-migration.json'
$source=(Resolve-Path -LiteralPath $Executable).Path
foreach($path in @($env:ProgramFiles,$env:ProgramData,$install,$data,$destination,$receiptPath,$source)){
 if((Get-Item -LiteralPath $path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Worker migration paths must not be reparse points.'}
}
if((Test-Path -LiteralPath $journal) -or (Test-Path -LiteralPath $configurationFile)){throw 'Migration state already exists; preserve it for explicit recovery.'}
$service=Get-CimInstance Win32_Service -Filter "Name='SwanInstallerWorker'"
if(!$service -or $service.PathName -cne ('"'+$destination+'" --service') -or $service.StartName -notin @('LocalSystem','NT AUTHORITY\SYSTEM') -or $service.State -ne 'Stopped' -or $service.StartMode -ne 'Manual'){throw 'Inspect and stop the owned legacy worker, with manual startup, before migration.'}
if(@(Get-CimInstance Win32_Process|Where-Object {$_.ExecutablePath -ieq $destination}).Count){throw 'An existing worker process remains alive.'}
$installAcl=Get-Acl -LiteralPath $install
if(!$installAcl.AreAccessRulesProtected -or $installAcl.GetOwner([Security.Principal.SecurityIdentifier]).Value -notin @('S-1-5-18','S-1-5-32-544')){throw 'Existing worker executable directory is not private.'}
foreach($rule in $installAcl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])){if($rule.AccessControlType -ne 'Allow' -or $rule.IdentityReference.Value -notin @('S-1-5-18','S-1-5-32-544')){throw 'Existing worker executable directory grants unexpected access.'}}
$receipt=Get-Content -LiteralPath $receiptPath -Raw|ConvertFrom-Json
if($receipt.schema -ne 1 -or $receipt.service -cne 'SwanInstallerWorker' -or $receipt.executable_sha256 -ine (Get-FileHash -LiteralPath $destination).Hash){throw 'Legacy worker receipt or executable differs.'}
foreach($item in @(@{path=$destination;pin=$receipt.publisher_thumbprint},@{path=$source;pin=$PublisherThumbprint})){
 $signature=Get-AuthenticodeSignature -LiteralPath $item.path
 if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ine $item.pin){throw 'Worker executable publisher verification failed.'}
}
$read=Join-Path $PSScriptRoot 'Read-WorkerConfiguration.ps1'
try{
 $previous=(& $read -Path $PreviousConfigurationPath -Scope input -Output pipeline)|ConvertFrom-Json
 $next=(& $read -Path $ConfigurationPath -Scope input -Output pipeline)|ConvertFrom-Json
}catch{throw 'Both worker configurations must be valid private local files.'}
$fields=@('management_url','worker_token','release_public_key','profile_public_key')
foreach($configuration in @($previous,$next)){
 if(@($configuration.PSObject.Properties.Name).Count -ne 4 -or @($configuration.PSObject.Properties.Name|Where-Object {$_ -notin $fields}).Count){throw 'Worker configurations must contain exactly the documented fields.'}
 foreach($field in $fields){if($configuration.$field -isnot [string] -or [string]::IsNullOrWhiteSpace($configuration.$field)){throw 'Worker configuration fields must be strings.'}}
 if($configuration.worker_token -cnotmatch '^[\x21-\x7E]{32,256}$'){throw 'Invalid worker credential format.'}
}
foreach($field in @('management_url','release_public_key','profile_public_key')){if($previous.$field -cne $next.$field){throw 'Credential migration must retain the existing company and trust keys.'}}
if($previous.worker_token -ceq $next.worker_token){throw 'Revoke the legacy registration and obtain a new credential.'}
$uri=$null
if(![Uri]::TryCreate($next.management_url,[UriKind]::Absolute,[ref]$uri) -or $uri.Scheme -cne 'https' -or !$uri.Host -or $uri.UserInfo -or $uri.Query -or $uri.Fragment){throw 'Worker migration requires the existing company HTTPS origin.'}
# A receipt or operator assertion is insufficient: confirm that the real HTTPS
# server rejects the actual old credential. Never disable TLS validation.
[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12
$request=[Net.HttpWebRequest]::Create($next.management_url.TrimEnd('/')+'/api/v1/worker/recipe')
$request.Method='GET';$request.AllowAutoRedirect=$false;$request.Timeout=15000;$request.ReadWriteTimeout=15000
$request.Headers['Authorization']='Bearer '+$previous.worker_token
$rejected=$false
try{$response=$request.GetResponse();$response.Close()}catch [Net.WebException]{
 if($_.Exception.Response){try{$rejected=[int]$_.Exception.Response.StatusCode -eq 401}finally{$_.Exception.Response.Close()}}
}
if(!$rejected){throw 'Company HTTPS endpoint did not confirm rejection of the legacy credential; no files changed.'}
$request=[Net.HttpWebRequest]::Create($next.management_url.TrimEnd('/')+'/api/v1/worker/recipe')
$request.Method='GET';$request.AllowAutoRedirect=$false;$request.Timeout=15000;$request.ReadWriteTimeout=15000
$request.Headers['Authorization']='Bearer '+$next.worker_token
$accepted=$false
try{$response=$request.GetResponse();try{$accepted=[int]$response.StatusCode -eq 200}finally{$response.Close()}}catch [Net.WebException]{if($_.Exception.Response){$_.Exception.Response.Close()}}
if(!$accepted){throw 'Company HTTPS endpoint did not accept the replacement worker credential; no files changed.'}
if(!$PSCmdlet.ShouldProcess('SwanInstallerWorker','Preserve the stopped legacy installation and migrate to a new private-file credential')){return}
# Inspect every retained object without changing its permissions. Only an owned
# private installation may be migrated; public or linked state needs investigation.
& (Join-Path $PSScriptRoot 'Prepare-CustomerDirectory.ps1') -Directory $data -Scope worker -Existing allow
$backup=Join-Path $install ('legacy-'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $backup|Out-Null
Copy-Item -LiteralPath $destination -Destination (Join-Path $backup 'swan-worker.exe')
Copy-Item -LiteralPath $receiptPath -Destination (Join-Path $backup 'installation.json')
if((Get-FileHash (Join-Path $backup 'swan-worker.exe')).Hash -ine $receipt.executable_sha256){throw 'Legacy preservation hash differs; keep the service stopped.'}
$record=[ordered]@{schema=1;utc=[DateTime]::UtcNow.ToString('o');phase='preserved';legacy_directory=$backup;legacy_credential_rejected=$true;replacement_sha256=(Get-FileHash $source).Hash;artifacts_retained=$true}
$record|ConvertTo-Json|Set-Content -LiteralPath $journal
try{
 $key=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Services\SwanInstallerWorker',$true)
 if(!$key){throw 'Worker service registry disappeared.'}
 try{$key.SetValue('Environment',[string[]]@("SWAN_WORKER_CONFIG=$configurationFile"),[Microsoft.Win32.RegistryValueKind]::MultiString)}finally{$key.Close()}
 # The retained root is private. Seal the empty file before writing secret bytes.
 $stream=[IO.FileStream]::new($configurationFile,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
 try{
  $acl=[Security.AccessControl.FileSecurity]::new();$acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'));$acl.SetAccessRuleProtection($true,$false)
  foreach($sid in @('S-1-5-18','S-1-5-32-544')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','Allow'))}
  Set-Acl -LiteralPath $configurationFile -AclObject $acl
  $actual=Get-Acl -LiteralPath $configurationFile
  if(!$actual.AreAccessRulesProtected -or $actual.GetOwner([Security.Principal.SecurityIdentifier]).Value -cne 'S-1-5-32-544'){throw 'Replacement configuration is not private.'}
  foreach($rule in $actual.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])){if($rule.AccessControlType -ne 'Allow' -or $rule.IdentityReference.Value -notin @('S-1-5-18','S-1-5-32-544')){throw 'Unexpected configuration access; no credential written.'}}
  $canonical=[ordered]@{};foreach($field in $fields){$canonical[$field]=$next.$field}
  $bytes=[Text.UTF8Encoding]::new($false).GetBytes(($canonical|ConvertTo-Json -Compress));$stream.Write($bytes,0,$bytes.Length);$stream.Flush($true)
 }finally{$stream.Dispose()}
 Copy-Item -LiteralPath $source -Destination $destination -Force
 if((Get-FileHash $destination).Hash -cne $record.replacement_sha256){throw 'Replacement executable changed.'}
 $signature=Get-AuthenticodeSignature $destination
 if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ine $PublisherThumbprint){throw 'Replacement publisher changed.'}
 $start=[Diagnostics.ProcessStartInfo]::new();$start.FileName=$destination;$start.Arguments='check-configuration';$start.UseShellExecute=$false;$start.CreateNoWindow=$true
 foreach($name in @('SWAN_WORKER_TOKEN','SWAN_MANAGEMENT_URL','SWAN_RELEASE_PUBLIC_KEY','SWAN_PROFILE_PUBLIC_KEY','SWAN_ARTIFACT_DIR')){[void]$start.EnvironmentVariables.Remove($name)}
 $start.EnvironmentVariables['SWAN_WORKER_CONFIG']=$configurationFile
 $process=[Diagnostics.Process]::Start($start)
 try{if(!$process.WaitForExit(300000)){$process.Kill();$process.WaitForExit();throw 'Configuration verification timed out.'};if($process.ExitCode -ne 0){throw 'Compiled worker rejected replacement configuration.'}}finally{$process.Dispose()}
 $record.phase='validated';$record|ConvertTo-Json|Set-Content -LiteralPath $journal
 @{schema=2;service='SwanInstallerWorker';executable_sha256=$record.replacement_sha256;publisher_thumbprint=$PublisherThumbprint;configuration_path=$configurationFile}|ConvertTo-Json|Set-Content -LiteralPath $receiptPath
 Start-Service SwanInstallerWorker;(Get-Service SwanInstallerWorker).WaitForStatus('Running',[TimeSpan]::FromSeconds(30))
 Set-Service SwanInstallerWorker -StartupType Automatic
 $record.phase='started';$record|ConvertTo-Json|Set-Content -LiteralPath $journal
}catch{
 Stop-Service SwanInstallerWorker -ErrorAction Continue;Set-Service SwanInstallerWorker -StartupType Manual
 Write-Warning 'Migration failed. Preserve files and journal; the legacy credential remains revoked and the old worker will not be restarted.'
 throw
}
Write-Host 'Private-file worker migration started. Verify authenticated build completion and reboot before qualification. Legacy files and artifacts remain retained.'
