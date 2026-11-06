$ErrorActionPreference='Stop'
$script=Join-Path $PSScriptRoot 'Complete-CustomerExeUninstall.ps1'
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile($script,[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Customer uninstall completion failed parsing.' }
$function=$ast.Find({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Remove-CustomerExeAgent'},$true)
if (-not $function) { throw 'Customer uninstall completion function missing.' }
. ([ScriptBlock]::Create($function.Extent.Text))
$root=Join-Path ([IO.Path]::GetTempPath()) ('swan-customer-exe-uninstall-'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory $root|Out-Null
$agent=Join-Path $root 'swan-agent.exe';$marker=Join-Path $root 'pending-uninstall'
$state=Join-Path $root 'managed-state.json';$other=Join-Path $root 'other.exe'
try {
    [IO.File]::WriteAllText($agent,'inert agent fixture')
    [IO.File]::WriteAllText($state,'retained identity and consent')
    [IO.File]::WriteAllText($other,'unrelated retained fixture')
    $rejected=$false
    try { Remove-CustomerExeAgent -Directory $root } catch { $rejected=$true }
    if (-not $rejected -or -not (Test-Path $agent)) { throw 'Agent removed without uninstall cancellation.' }
    New-Item -ItemType Directory $marker|Out-Null
    $rejected=$false
    try { Remove-CustomerExeAgent -Directory $root } catch { $rejected=$true }
    if (-not $rejected -or -not (Test-Path $agent)) { throw 'Invalid marker allowed removal.' }
    Remove-Item -LiteralPath $marker
    [IO.File]::WriteAllText($marker,'')
    Remove-CustomerExeAgent -Directory $root
    Remove-CustomerExeAgent -Directory $root
    if ((Test-Path $agent) -or -not (Test-Path $marker) -or [IO.File]::ReadAllText($state) -cne 'retained identity and consent' -or [IO.File]::ReadAllText($other) -cne 'unrelated retained fixture') { throw 'Removal changed retained data or failed to remove agent.' }
    New-Item -ItemType Directory $agent|Out-Null
    $rejected=$false
    try { Remove-CustomerExeAgent -Directory $root } catch { $rejected=$true }
    if (-not $rejected -or -not (Test-Path $agent)) { throw 'Directory accepted as agent.' }
} finally {
    foreach($path in @($agent,$marker,$state,$other)) { if(Test-Path -LiteralPath $path){Remove-Item -LiteralPath $path} }
    Remove-Item -LiteralPath $root
}
Write-Output 'Customer EXE uninstall completion checks passed; installed applications were not changed.'
