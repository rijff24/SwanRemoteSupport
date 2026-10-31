param([Parameter(Mandatory=$true)][string]$ManagementExe)
$ErrorActionPreference='Stop'
if($env:GITHUB_ACTIONS -cne 'true' -or $env:RUNNER_OS -cne 'Windows'){throw 'This canonical-directory fixture requires a disposable Windows CI runner.'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Administrator fixture required.'}
$exe=(Resolve-Path -LiteralPath $ManagementExe).Path
$root=Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanCompanyServer'
if((Test-Path -LiteralPath $root) -or (Get-Service SwanCompanyServer -ErrorAction SilentlyContinue)){throw 'Existing server state must not be changed.'}
function Invoke-Preparation([string]$Path){
    $start=[Diagnostics.ProcessStartInfo]::new();$start.FileName=$exe
    $start.Arguments='prepare-server-directory "'+$Path+'"';$start.UseShellExecute=$false;$start.CreateNoWindow=$true
    $start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
    $process=[Diagnostics.Process]::Start($start)
    try{
        $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
        if(-not $process.WaitForExit(30000)){$process.Kill();$process.WaitForExit();throw 'Directory preparation exceeded its deadline.'}
        $null=$stdout.GetAwaiter().GetResult();$null=$stderr.GetAwaiter().GetResult()
        return $process.ExitCode
    }finally{$process.Dispose()}
}
$created=$false
try{
    $wrong=Join-Path ([IO.Path]::GetTempPath()) ('swan-wrong-server-'+[Guid]::NewGuid().ToString('N'))
    if((Invoke-Preparation $wrong) -eq 0 -or (Test-Path -LiteralPath $wrong)){throw 'Management command accepted a noncanonical path.'}
    if((Invoke-Preparation $root) -ne 0){throw 'Fresh private server directory creation failed.'};$created=$true
    $acl=Get-Acl -LiteralPath $root;$before=$acl.Sddl
    if(-not $acl.AreAccessRulesProtected -or $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -cne 'S-1-5-32-544'){throw 'Server directory owner or protection differs.'}
    $rules=@($acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier]))
    if($rules.Count -ne 2){throw 'Unexpected server directory rules.'}
    foreach($sid in @('S-1-5-18','S-1-5-32-544')){
        $rule=@($rules|Where-Object {$_.IdentityReference.Value -ceq $sid -and $_.AccessControlType -eq 'Allow' -and $_.FileSystemRights -eq 'FullControl'})
        if($rule.Count -ne 1){throw 'Server directory permissions differ.'}
    }
    if((Invoke-Preparation $root) -eq 0 -or (Get-Acl -LiteralPath $root).Sddl -cne $before){throw 'Fresh server setup adopted or changed existing state.'}
    if(@(Get-ChildItem -LiteralPath $root -Force).Count -or (Get-Service SwanCompanyServer -ErrorAction SilentlyContinue)){throw 'Preparation created data or a service.'}
    Write-Output 'PASS: compiled management creates canonical private state atomically, rejects wrong paths and refuses existing state without changing permissions; no server started.'
}finally{
    if($created -and (Test-Path -LiteralPath $root)){
        $item=Get-Item -LiteralPath $root
        if($item.PSIsContainer -and -not ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -and @(Get-ChildItem -LiteralPath $root -Force).Count -eq 0){[IO.Directory]::Delete($root,$false)}
    }
}
