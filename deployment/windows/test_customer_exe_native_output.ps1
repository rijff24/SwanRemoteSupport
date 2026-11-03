param(
    [Parameter(Mandatory=$true)][string]$PythonExe,
    [string]$ProducerPath
)
$ErrorActionPreference='Stop'
if(!$ProducerPath){$ProducerPath=Join-Path $PSScriptRoot 'Build-CustomerExe.ps1'}
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile($ProducerPath,[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Producer syntax invalid'}
$functions=@($ast.FindAll({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Invoke-BuildTool'},$true))
if($functions.Count -ne 1){throw 'Expected one real producer native-command helper'}
. ([ScriptBlock]::Create($functions[0].Extent.Text))
Invoke-BuildTool $PythonExe @('-c',"import sys; print('normal build output'); print('normal compiler progress', file=sys.stderr)")
if($ErrorActionPreference -cne 'Stop'){throw 'Native progress weakened PowerShell error handling'}
$rejected=$false
try{Invoke-BuildTool $PythonExe @('-c',"import sys; print('deliberate build rejection', file=sys.stderr); sys.exit(7)")}catch{if($_.Exception.Message -notmatch 'Build tool failed \(7\)'){throw};$rejected=$true}
if(!$rejected -or $ErrorActionPreference -cne 'Stop'){throw 'Native failure was accepted or preference was not restored'}
Invoke-BuildTool $PythonExe @('-c',"print('successful command after rejection')")
$missing=Join-Path $PSScriptRoot 'nonexistent-native-build-tool.exe'
if(Test-Path $missing){throw 'Missing-tool fixture already exists'}
$rejected=$false
try{Invoke-BuildTool $missing @()}catch{$rejected=$true}
if(!$rejected -or $ErrorActionPreference -cne 'Stop'){throw 'Missing tool inherited a stale successful exit code'}
Write-Host 'Actual producer helper accepts native progress, rejects exit7 and a missing tool, restores Stop and supports a subsequent successful command.'
