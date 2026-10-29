param([Parameter(Mandatory=$true)][string]$WorkerExecutable,[Parameter(Mandatory=$true)][string]$ResultPath)
$ErrorActionPreference='Stop'
$worker=(Resolve-Path -LiteralPath $WorkerExecutable).Path
$root=[IO.Path]::GetFullPath((Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanInstallerWorker'))
if((Test-Path -LiteralPath $root) -or (Get-Service SwanInstallerWorker -ErrorAction SilentlyContinue)){throw 'Only a fresh isolated worker fixture is allowed.'}
if(Test-Path -LiteralPath $ResultPath){throw 'Preserve prior test evidence.'}
$fixtureToken='fixture-worker-credential-not-a-real-company-secret-000000'
$key='11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo='
$nonce=[Guid]::NewGuid().ToString('N');$created=$false;$checks=[Collections.Generic.List[string]]::new()
function Invoke-Fixture([string]$Mode,[string]$Argument,[bool]$Configured,[bool]$Success) {
 $start=[Diagnostics.ProcessStartInfo]::new();$start.FileName=$worker;$start.Arguments=$Mode
 if($Argument){$start.Arguments+=' "'+$Argument+'"'}
 $start.UseShellExecute=$false;$start.CreateNoWindow=$true;$start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
 foreach($name in @('SWAN_WORKER_CONFIG','SWAN_WORKER_TOKEN','SWAN_MANAGEMENT_URL','SWAN_RELEASE_PUBLIC_KEY','SWAN_PROFILE_PUBLIC_KEY','SWAN_ARTIFACT_DIR')){[void]$start.EnvironmentVariables.Remove($name)}
 if($Configured){$start.EnvironmentVariables['SWAN_WORKER_CONFIG']=(Join-Path $root 'worker.json')}else{
  $start.EnvironmentVariables['SWAN_WORKER_TOKEN']=$fixtureToken;$start.EnvironmentVariables['SWAN_MANAGEMENT_URL']='https://example.invalid'
  $start.EnvironmentVariables['SWAN_RELEASE_PUBLIC_KEY']=$key;$start.EnvironmentVariables['SWAN_PROFILE_PUBLIC_KEY']=$key;$start.EnvironmentVariables['SWAN_ARTIFACT_DIR']=$root
 }
 $process=[Diagnostics.Process]::Start($start)
 try {
  $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
  if(!$process.WaitForExit(300000)){$process.Kill();$process.WaitForExit();throw 'Owned configuration verifier exceeded its deadline.'}
  $text=$stdout.GetAwaiter().GetResult()+$stderr.GetAwaiter().GetResult()
  if($text.Contains($fixtureToken)){throw 'Worker fixture credential was reflected into diagnostics.'}
  if(($process.ExitCode -eq 0) -ne $Success){throw 'Compiled worker configuration result differs from the expected access policy.'}
  return $text
 }finally{$process.Dispose()}
}
function Set-PrivateFileAcl([string]$Path) {
 $acl=[Security.AccessControl.FileSecurity]::new();$acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'));$acl.SetAccessRuleProtection($true,$false)
 foreach($sid in @('S-1-5-18','S-1-5-32-544')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','Allow'))}
 Set-Acl -LiteralPath $Path -AclObject $acl
}
try {
 Invoke-Fixture 'prepare-worker-directory' $root $false $true|Out-Null;$created=$true
 [IO.File]::WriteAllText((Join-Path $root 'test-owner-marker'),$nonce)
 $configuration=Join-Path $root 'worker.json'
 $json=@{management_url='https://example.invalid';worker_token=$fixtureToken;release_public_key=$key;profile_public_key=$key}|ConvertTo-Json -Compress
 [IO.File]::WriteAllText($configuration,($json+(' '*8192)),[Text.UTF8Encoding]::new($false));Set-PrivateFileAcl $configuration
 Invoke-Fixture 'check-configuration' '' $true $true|Out-Null;$checks.Add('compiled worker accepts protected configuration larger than the Windows pipe buffer without revealing credentials')
 $acl=Get-Acl -LiteralPath $configuration
 $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-15-2-1'),'Read','Allow'))
 Set-Acl -LiteralPath $configuration -AclObject $acl
 Invoke-Fixture 'check-configuration' '' $true $false|Out-Null;$checks.Add('app-container read permission rejects configuration before any job is claimed')
 Set-PrivateFileAcl $configuration
 $owner=[Security.Principal.WindowsIdentity]::GetCurrent().User
 if($owner.Value -notin @('S-1-5-18','S-1-5-32-544')){
  $acl=Get-Acl -LiteralPath $configuration;$acl.SetOwner($owner);Set-Acl -LiteralPath $configuration -AclObject $acl
  Invoke-Fixture 'check-configuration' '' $true $false|Out-Null;$checks.Add('unexpected file owner rejected');Set-PrivateFileAcl $configuration
 }
 [IO.File]::WriteAllText($configuration,('x'*16385))
 Invoke-Fixture 'check-configuration' '' $true $false|Out-Null;$checks.Add('oversized configuration rejected')
 [IO.File]::WriteAllText($configuration,('{"worker_token":"'+$fixtureToken+'"'))
 Invoke-Fixture 'check-configuration' '' $true $false|Out-Null;$checks.Add('malformed private JSON rejected without echoing credentials')
 $text=Invoke-Fixture 'check-configuration' '' $false $false
 if($text -notmatch 'require protected file configuration'){throw 'Legacy environment was not explicitly refused.'}
 $checks.Add('complete legacy environment cannot substitute for the service private file')
 @{passed=$true;utc=[DateTime]::UtcNow.ToString('o');worker_sha256=(Get-FileHash $worker).Hash.ToLowerInvariant();checks=@($checks);only_fixture_credentials=$true;service_installed=$false;company_request_or_build_claimed=$false}|ConvertTo-Json -Depth 4|Set-Content -LiteralPath $ResultPath
}finally{
 # This explicitly named canonical root was absent before this test and contains
 # only our marker and fake settings. Never clean an installed worker's state.
 if($created -and [IO.Path]::GetFullPath($root) -ceq [IO.Path]::GetFullPath((Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanInstallerWorker')) -and !(Get-Service SwanInstallerWorker -ErrorAction SilentlyContinue)){
  if([IO.File]::ReadAllText((Join-Path $root 'test-owner-marker')) -cne $nonce){throw 'Fixture ownership changed; preserve all files.'}
  Remove-Item -LiteralPath $root -Recurse -Force
 }
}
Write-Host 'Compiled private worker configuration checks passed; no worker service or job was started.'
