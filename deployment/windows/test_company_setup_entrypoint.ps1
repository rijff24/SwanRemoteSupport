$ErrorActionPreference='Stop'
if (-not [Environment]::Is64BitProcess) { throw 'Run the x64 setup-entry tests.' }
$root=Join-Path ([IO.Path]::GetTempPath()) ('swan-setup-entry-'+[guid]::NewGuid().ToString('N'))
$bundle=Join-Path $root "Company & setup's folder!"
$oldLocal=$env:LOCALAPPDATA
$process=$null
New-Item -ItemType Directory -Path $bundle|Out-Null
try {
    foreach($name in @('Install-Company.ps1','Install-SwanRemoteSupport.cmd')) {
        $text=[IO.File]::ReadAllText((Join-Path $PSScriptRoot $name)).Replace("`r`n","`n")
        [IO.File]::WriteAllText((Join-Path $bundle $name),$text,[Text.UTF8Encoding]::new($false))
    }
    $setup=Join-Path $bundle 'Install-Company.ps1'
    $bootstrap=Join-Path $bundle 'bootstrap.json'
    $env:LOCALAPPDATA=Join-Path $root 'profile'
    [IO.File]::WriteAllText((Join-Path $bundle 'swan-agent.exe'),'This fixture must never execute.')
    function Write-Bootstrap([string]$Edition,[string]$Url) {
        @{edition=$Edition;management_url=$Url}|ConvertTo-Json|Set-Content -LiteralPath $bootstrap -Encoding UTF8
    }
    $setupTestState=[pscustomobject]@{Calls=@();ExitCode=0;Cancel=$false}
    function Start-Process {
        param($FilePath,$ArgumentList,$Verb,[switch]$PassThru,[switch]$Wait)
        $setupTestState.Calls+=@{file=$FilePath;arguments=$ArgumentList;verb=$Verb;wait=[bool]$Wait}
        if($setupTestState.Cancel){throw 'Simulated elevation cancellation'}
        [pscustomobject]@{ExitCode=$setupTestState.ExitCode}
    }
    function Assert-Rejected([string]$Edition,[string]$Url,[string]$Expected) {
        Write-Bootstrap $Edition $Url
        $rejected=$false
        try { & $setup -RequestElevation -ConfirmedCompanyDomain 'company.test' }
        catch { if($_.Exception.Message -cne $Expected){throw};$rejected=$true }
        if(-not $rejected -or $setupTestState.Calls.Count){throw 'Preflight unexpectedly started an elevated process'}
    }
    Assert-Rejected 'other' 'https://company.test' 'Unknown package edition.'
    Assert-Rejected 'customer' 'http://company.test' 'A company HTTPS endpoint is required.'
    Assert-Rejected 'technician' 'https://company.test' 'Configuration agent is not trusted.'
    if(Test-Path $env:LOCALAPPDATA){throw 'Rejected technician setup wrote installation state'}

    $administrator=([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if(-not $administrator) {
        Write-Bootstrap 'customer' 'https://company.test'
        & $setup -RequestElevation -ConfirmedCompanyDomain 'company.test' -Repair -ReplaceFailedSetup -UnattendedConsent
        if($setupTestState.Calls.Count -ne 1 -or $setupTestState.Calls[0].verb -cne 'RunAs' -or -not $setupTestState.Calls[0].wait){throw 'Customer setup did not request one awaited elevation'}
        $a=$setupTestState.Calls[0].arguments
        foreach($value in @(('"'+$setup+'"'),('"'+$bootstrap+'"'),'-Repair','-ReplaceFailedSetup','-UnattendedConsent','"company.test"')){if($value -cnotin $a){throw 'Elevation lost a quoted path or explicit setup option'}}
        if('-RequestElevation' -in $a){throw 'Elevation could loop'}
        $setupTestState.Calls=@();$setupTestState.ExitCode=32
        $rejected=$false
        try { & $setup -RequestElevation -ConfirmedCompanyDomain 'company.test' }
        catch { if($_.Exception.Message -notlike 'Customer setup did not complete (32).*'){throw};$rejected=$true }
        if(-not $rejected){throw 'Elevated failure was reported as success'}
        $setupTestState.Calls=@();$setupTestState.Cancel=$true;$rejected=$false
        try { & $setup -RequestElevation -ConfirmedCompanyDomain 'company.test' }
        catch { if($_.Exception.Message -cne 'Customer setup requires administrator approval. Setup was not started.'){throw};$rejected=$true }
        if(-not $rejected){throw 'Cancelled elevation did not fail closed'}
    }
    Remove-Item Function:\Start-Process

    # Exercise the real CMD/Windows PowerShell path with LF bundle text, spaces,
    # apostrophe and ampersand in its folder. The invalid fixture agent must fail
    # signature verification before any installation or enrollment can occur.
    Write-Bootstrap 'technician' 'https://company.test'
    $start=[Diagnostics.ProcessStartInfo]::new()
    $start.FileName=Join-Path ([Environment]::GetFolderPath('System')) 'cmd.exe'
    $start.Arguments='/d /s /c ""'+(Join-Path $bundle 'Install-SwanRemoteSupport.cmd')+'""'
    $start.UseShellExecute=$false;$start.CreateNoWindow=$true
    $start.RedirectStandardInput=$true;$start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
    $process=[Diagnostics.Process]::Start($start)
    $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
    # Windows PowerShell may select a BOM-emitting encoding for redirected
    # input on hosted runners. Supply the same plain hostname a user types.
    $inputWriter=[IO.StreamWriter]::new($process.StandardInput.BaseStream,[Text.UTF8Encoding]::new($false))
    try { $inputWriter.WriteLine('company.test') } finally { $inputWriter.Dispose() }
    if(-not $process.WaitForExit(60000)){throw 'Setup fixture did not exit; inspect the owned test process'}
    $output=$stdout.GetAwaiter().GetResult()+$stderr.GetAwaiter().GetResult()
    if($process.ExitCode -ne 1 -or $output -notmatch 'Configuration agent is not trusted.' -or $output -notmatch 'Setup did not complete.'){
        # This subprocess has only inert fixture inputs, no credentials or company state.
        Write-Output (@{fixture_exit_code=$process.ExitCode;fixture_output=$output.Substring(0,[Math]::Min($output.Length,4096))}|ConvertTo-Json -Compress)
        throw 'Real entry point lost its path, trust rejection or failure exit code'
    }
    $process.Dispose()
    $process=$null
    if(Test-Path $env:LOCALAPPDATA){throw 'Rejected CMD setup wrote installation state'}
    @{real_cmd_trust_rejection=$true;technician_never_elevates=$true;unsafe_bootstrap_rejected=$true;elevation_routing_fixture_tested=(-not $administrator);actual_uac_tested=$false;scope='Entry-point routing and real unsigned-agent rejection; no successful product installation'}|ConvertTo-Json -Compress
} finally {
    if($null -ne $process) {
        try {
            if(-not $process.HasExited) {
                # This handle identifies only the CMD process created above and
                # its fixture children; never stop a user's existing console.
                & (Join-Path ([Environment]::GetFolderPath('System')) 'taskkill.exe') /PID $process.Id /T /F|Out-Null
            }
        } finally { $process.Dispose() }
    }
    Remove-Item Function:\Start-Process -ErrorAction SilentlyContinue
    $env:LOCALAPPDATA=$oldLocal
    $resolved=[IO.Path]::GetFullPath($root)
    if([IO.Path]::GetDirectoryName($resolved) -ine [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') -or [IO.Path]::GetFileName($resolved) -notmatch '^swan-setup-entry-[a-f0-9]{32}$'){throw 'Fixture cleanup path changed'}
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
