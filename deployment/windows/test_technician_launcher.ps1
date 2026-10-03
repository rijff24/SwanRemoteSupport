$ErrorActionPreference='Stop'
$root=Join-Path ([IO.Path]::GetTempPath()) ('swan-launcher-test-'+[guid]::NewGuid().ToString('N'))
$installation=Join-Path $root 'SwanRemoteSupport-Technician'
New-Item -ItemType Directory -Path $installation|Out-Null
$oldLocal=$env:LOCALAPPDATA;$oldState=$env:SWAN_STATE_DIR
$oldTrace=$env:SWAN_TEST_TRACE;$oldDeny=$env:SWAN_TEST_DENY
try {
    $code=@'
using System;
using System.IO;
public class LauncherFixture {
 public static int Main(string[] args) {
  var mode=args.Length==0?"application":args[0];
  File.AppendAllText(Environment.GetEnvironmentVariable("SWAN_TEST_TRACE"),mode+"\n");
  if(mode=="sync") { Console.Error.WriteLine("fixture network unavailable"); return 7; }
  if(mode=="verify-installed" && Environment.GetEnvironmentVariable("SWAN_TEST_DENY")=="1") { return 9; }
  return 0;
 }
}
'@
    $agent=Join-Path $installation 'swan-agent.exe'
    Add-Type -TypeDefinition $code -OutputAssembly $agent -OutputType ConsoleApplication
    Copy-Item -LiteralPath $agent -Destination (Join-Path $installation 'SwanRemoteSupport-Technician.exe')
    $env:LOCALAPPDATA=$root
    $env:SWAN_TEST_TRACE=Join-Path $root 'trace.txt'
    # This isolated fixture tests launcher control flow, not Authenticode trust.
    function Get-AuthenticodeSignature { param([string]$LiteralPath) [pscustomobject]@{Status='Valid'} }
    $launcher=Join-Path $PSScriptRoot 'Open-Technician.ps1'
    $env:SWAN_TEST_DENY='0'
    & $launcher
    if((Get-Content $env:SWAN_TEST_TRACE -Raw) -cne "sync`nverify-installed`napplication`n"){throw 'Offline launcher did not verify before starting application'}
    [IO.File]::WriteAllText($env:SWAN_TEST_TRACE,'')
    $env:SWAN_TEST_DENY='1'
    $rejected=$false
    try { & $launcher } catch { if($_.Exception.Message -cne 'Technician installed identity or pinned publisher verification failed.'){throw};$rejected=$true }
    if(-not $rejected -or (Get-Content $env:SWAN_TEST_TRACE -Raw) -cne "sync`nverify-installed`n"){throw 'Invalid installed package was launched'}
    Write-Output 'Offline launch and installed-verification rejection passed (fixture scope).'
} finally {
    Remove-Item Function:\Get-AuthenticodeSignature -ErrorAction SilentlyContinue
    $env:LOCALAPPDATA=$oldLocal;$env:SWAN_STATE_DIR=$oldState
    $env:SWAN_TEST_TRACE=$oldTrace;$env:SWAN_TEST_DENY=$oldDeny
    # Delete only the unique fixture directory after checking its absolute parent.
    $resolved=[IO.Path]::GetFullPath($root)
    if([IO.Path]::GetDirectoryName($resolved) -ine [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') -or [IO.Path]::GetFileName($resolved) -notmatch '^swan-launcher-test-[a-f0-9]{32}$'){throw 'Fixture cleanup path changed'}
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
