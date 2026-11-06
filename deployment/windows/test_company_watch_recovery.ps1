$ErrorActionPreference='Stop'
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'Install-Company.ps1'),[ref]$tokens,[ref]$errors)
if($errors){throw 'Setup source does not parse'}
$capture=@($ast.FindAll({param($node) $node -is [Management.Automation.Language.AssignmentStatementAst] -and $node.Left.Extent.Text -eq '$configurationTaskRestore' -and $node.Right.Extent.Text.StartsWith('@{')},$true))
$handler=@($ast.FindAll({param($node) $node -is [Management.Automation.Language.CatchClauseAst] -and $node.Body.Extent.Text -match '^\{\s*\$setupError = \$_'},$true))
if($capture.Count -ne 1 -or $handler.Count -ne 1){throw 'Expected actual setup snapshot and failure handler'}
$snapshot=[scriptblock]::Create($capture[0].Extent.Text)
$text=$handler[0].Body.Extent.Text
$recover=[scriptblock]::Create($text.Substring(1,$text.Length-2))
$editionDirectory=Join-Path ([IO.Path]::GetTempPath()) 'swan-watch-unit-state'
$agent=[IO.Path]::GetFullPath((Join-Path $editionDirectory 'swan-agent.exe'))
function Get-ScheduledTask {param($TaskName,$TaskPath) if($TaskName -cne 'Swan Company Configuration' -or $TaskPath -cne '\'){throw 'Unexpected task target'};if(-not $script:missing){$script:task}}
function Enable-ScheduledTask {param([Parameter(ValueFromPipeline)]$InputObject) process {$script:enabled++;$script:task.State='Ready'}}
function Disable-ScheduledTask {param([Parameter(ValueFromPipeline)]$InputObject) process {$script:disabled++;$script:task.State='Disabled'}}
function Start-ScheduledTask {param([Parameter(ValueFromPipeline)]$InputObject) process {if($script:failStart){throw 'Simulated scheduler start failure'};$script:started++;$script:task.State='Running'}}
$cases=@(
    @{before='Running';after='Running';enable=1;start=1;disable=0},
    @{before='Queued';after='Running';enable=1;start=1;disable=0},
    @{before='Ready';after='Ready';enable=1;start=0;disable=0},
    @{before='Disabled';after='Disabled';enable=0;start=0;disable=1}
)
$count=0
foreach($case in $cases){
    $script:task=[pscustomobject]@{TaskName='Swan Company Configuration';TaskPath='\';State=$case.before;Actions=@([pscustomobject]@{Execute=$agent;Arguments='watch'});Principal=[pscustomobject]@{UserId='SYSTEM'}}
    $configurationTask=@($script:task);. $snapshot
    $script:task.State='Disabled';$script:enabled=0;$script:disabled=0;$script:started=0;$script:missing=$false;$script:failStart=$false
    $caught=$false
    try {try {throw 'Simulated MSI failure (1603).'}catch {& $recover}}catch {if($_.Exception.Message -cne 'Simulated MSI failure (1603).'){throw};$caught=$true}
    if(-not $caught -or $script:task.State -cne $case.after -or $script:enabled -ne $case.enable -or $script:started -ne $case.start -or $script:disabled -ne $case.disable){throw 'Setup failure did not preserve prior watcher state and original error'}
    $count++
}
foreach($unsafe in @('changed-action','missing-task','restart-failure')){
    $script:task=[pscustomobject]@{TaskName='Swan Company Configuration';TaskPath='\';State='Running';Actions=@([pscustomobject]@{Execute=$agent;Arguments='watch'});Principal=[pscustomobject]@{UserId='SYSTEM'}}
    $configurationTask=@($script:task);. $snapshot
    $script:task.State='Disabled';$script:enabled=0;$script:disabled=0;$script:started=0;$script:missing=($unsafe -eq 'missing-task');$script:failStart=($unsafe -eq 'restart-failure')
    if($unsafe -eq 'changed-action'){$script:task.Actions[0].Execute=Join-Path $editionDirectory 'unexpected.exe'}
    $caught=$false
    try {try {throw 'Original setup rejection.'}catch {& $recover}}catch {if($_.Exception.Message -notlike 'Customer setup failed and its configuration task could not be restored:*Original setup error: Original setup rejection.'){throw};$caught=$true}
    if(-not $caught -or $script:started -ne 0){throw 'Unsafe or failed task restoration was silently accepted'}
    if($unsafe -ne 'restart-failure' -and $script:enabled -ne 0){throw 'Changed or missing task was modified'}
    $count++
}
@{passed=$true;cases=$count;scope='Actual setup snapshot/failure-handler AST with isolated scheduler substitutes; no installed software, task or enrollment was changed'}|ConvertTo-Json -Compress
