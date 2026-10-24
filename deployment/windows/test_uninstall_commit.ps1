$ErrorActionPreference='Stop'
$script=Join-Path $PSScriptRoot 'Commit-Uninstall.ps1'
$tokens=$null;$errors=$null
$ast=[System.Management.Automation.Language.Parser]::ParseFile($script,[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Uninstall commit script failed parsing.' }
$function=$ast.Find({param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Complete-Uninstall'},$true)
if ($null -eq $function) { throw 'Commit cleanup function missing.' }
. ([ScriptBlock]::Create($function.Extent.Text))
$root=Join-Path ([IO.Path]::GetTempPath()) ('swan-uninstall-commit-'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root|Out-Null
$marker=Join-Path $root 'pending-uninstall';$cancellation=Join-Path $root 'uninstall-cancellation-rollback.json'
$task=Join-Path $root 'configuration-task-rollback.json';$state=Join-Path $root 'managed-state.json';$agent=Join-Path $root 'swan-agent.exe'
try {
    [IO.File]::WriteAllText($state,'identity and consent fixture: preserve exactly')
    & (Join-Path $PSScriptRoot 'Update-UninstallCancellation.ps1') -Directory $root -Operation cancel
    $rejected=$false
    try { & (Join-Path $PSScriptRoot 'Update-UninstallCancellation.ps1') -Directory $root -Operation cancel } catch { $rejected=$true }
    if (-not $rejected) { throw 'Active cancellation rollback was overwritten.' }
    [IO.File]::WriteAllText($task,'{"enabled":true,"running":false}')
    [IO.File]::WriteAllText($agent,'inert installed agent fixture')
    $rejected=$false
    try { Complete-Uninstall -Directory $root } catch { $rejected=$true }
    if (-not $rejected -or -not (Test-Path $cancellation)) { throw 'Commit ran before agent removal.' }
    Remove-Item -LiteralPath $agent
    # The real entrypoint must reject arbitrary directories even though the
    # pure cleanup function is exercised here against private temporary files.
    foreach($edition in @('customer','technician')) {
        $rejected=$false
        try { & $script -Directory $root -Edition $edition } catch { $rejected=$true }
        if (-not $rejected -or -not (Test-Path $cancellation)) { throw 'Canonical edition state-directory guard failed.' }
    }
    [IO.File]::WriteAllText($task,'{"enabled":"invalid","running":false}')
    $rejected=$false
    try { Complete-Uninstall -Directory $root } catch { $rejected=$true }
    if (-not $rejected -or -not (Test-Path $cancellation) -or -not (Test-Path $marker)) { throw 'Invalid task record partially cleared cancellation.' }
    [IO.File]::WriteAllText($task,'{"enabled":true,"running":false}')
    Complete-Uninstall -Directory $root
    Complete-Uninstall -Directory $root
    if ((Test-Path $cancellation) -or (Test-Path $task) -or -not (Test-Path $marker)) { throw 'Commit did not preserve cancellation while clearing rollback records.' }
    # A later removal can now create its own rollback; rollback must retain
    # the earlier completed uninstall's cancellation rather than enabling support.
    & (Join-Path $PSScriptRoot 'Update-UninstallCancellation.ps1') -Directory $root -Operation cancel
    & (Join-Path $PSScriptRoot 'Update-UninstallCancellation.ps1') -Directory $root -Operation restore
    if (-not (Test-Path $marker)) { throw 'Later uninstall rollback lost earlier cancellation.' }
    if ([IO.File]::ReadAllText($state) -cne 'identity and consent fixture: preserve exactly') { throw 'Commit changed identity/consent state.' }
} finally {
    foreach ($path in @($agent,$marker,$cancellation,$task,$state)) { if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path } }
    Remove-Item -LiteralPath $root
}
Write-Output 'Uninstall commit regression checks passed for both editions; no installed application or task changed.'
