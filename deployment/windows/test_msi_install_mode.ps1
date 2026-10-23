$ErrorActionPreference='Stop'
$fixture=Join-Path ([IO.Path]::GetTempPath()) ('swan-msi-mode-'+[guid]::NewGuid().ToString('N')+'.msi')
[IO.File]::WriteAllText($fixture,'Inert database stand-in; never executed.')
$script:code='{D32FA420-4F84-50A6-BAB2-CBFDA73F87C9}'
$script:other='{A6857EBE-9FD4-52DE-B899-454C2F066E52}'
$script:properties=@{ProductCode=$script:code;UpgradeCode='{A4374699-436F-4917-9A4E-223F62A9E634}';ProductVersion='1.5.1'}
$global:SwanMsiModeFixture=@{properties=$script:properties}
$database=[pscustomobject]@{}
$database|Add-Member ScriptMethod OpenView {param($sql) $name=$sql.Split("'")[-2];$view=[pscustomobject]@{PropertyName=$name};$view|Add-Member ScriptMethod Execute {};$view|Add-Member ScriptMethod Close {};$view|Add-Member ScriptMethod Fetch {$row=[pscustomobject]@{PropertyName=$this.PropertyName};$row|Add-Member ScriptMethod StringData {param($index) $global:SwanMsiModeFixture.properties[$this.PropertyName]};return $row};return $view}
$script:database=$database
$script:installer=[pscustomobject]@{}
$global:SwanMsiModeFixture.database=$database
$global:SwanMsiModeFixture.installer=$script:installer
$script:installer|Add-Member ScriptMethod OpenDatabase {param($path,$mode) if($mode -ne 0){throw 'Database must be opened read-only'};return $global:SwanMsiModeFixture.database}
$script:installer|Add-Member ScriptMethod RelatedProducts {param($upgrade) return @($global:SwanMsiModeFixture.related.Keys)}
$script:installer|Add-Member ScriptMethod ProductInfo {param($code,$name) if($name -cne 'VersionString'){throw 'Unexpected catalog query'};return $global:SwanMsiModeFixture.related[$code]}
$script:installer|Add-Member ScriptMethod ProductState {param($code) return $global:SwanMsiModeFixture.state}
function New-Object {param($ComObject) if($ComObject -cne 'WindowsInstaller.Installer'){throw 'Unexpected factory'};return $global:SwanMsiModeFixture.installer}
try {
    $cases=@(
        @{state=-1;related=@{};version='1.5.1';mode='/i'},
        @{state=1;related=@{};version='1.5.1';mode='/i'},
        @{state=5;related=@{$script:code='1.5.1'};version='1.5.1';mode='/fvamus'},
        @{state=-1;related=@{$script:other='1.5.0'};version='1.5.1';mode='/i'},
        @{state=-1;related=@{$script:other='1.5.7'};version='1.5.1';reject=$true;reason='This package cannot replace registered product version*'},
        @{state=-1;related=@{$script:other='1.5.1'};version='1.5.1';reject=$true;reason='This package cannot replace registered product version*'},
        @{state=-1;related=@{$script:other='1.5.1.999'};version='1.5.1';reject=$true;reason='This package cannot replace registered product version*'},
        @{state=-1;related=@{$script:other='1.10.0'};version='1.9.9';reject=$true;reason='This package cannot replace registered product version*'},
        @{state=-1;related=@{$script:other='broken'};version='1.5.1';reject=$true;reason='MSI version cannot be compared safely.'},
        @{state=2;related=@{};version='1.5.1';reject=$true;reason='MSI product registration cannot be safely recovered*'}
    )
    $caseIndex=0
    foreach($case in $cases){
        $script:state=$case.state;$script:related=$case.related;$script:properties.ProductVersion=$case.version
        $global:SwanMsiModeFixture.state=$case.state;$global:SwanMsiModeFixture.related=$case.related
        $failed=$false;$mode=$null
        $reason=$null
        try {$mode=& (Join-Path $PSScriptRoot 'Get-MsiInstallMode.ps1') -Package $fixture}catch {$failed=$true;$reason=$_.Exception.Message}
        if([bool]$case.reject -ne $failed -or (-not $failed -and $mode -cne $case.mode)){throw "Incorrect installation/repair/downgrade decision in case ${caseIndex}: failed=$failed, mode=$mode, reason=$reason"}
        if($failed -and $reason -notlike $case.reason){throw "Unexpected rejection reason in case ${caseIndex}: $reason"}
        $caseIndex++
    }
    @{passed=$true;cases=$cases.Count;scope='Read-only catalog decision tests; inert fixture and COM substitute; no MSI actions or registration changes'}|ConvertTo-Json -Compress
} finally {Remove-Item -LiteralPath $fixture -Force;Remove-Variable -Name SwanMsiModeFixture -Scope Global}
