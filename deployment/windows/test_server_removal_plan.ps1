$ErrorActionPreference = 'Stop'
$root = Join-Path ([IO.Path]::GetTempPath()) ('swan-server-removal-'+[guid]::NewGuid().ToString('N'))
$install = Join-Path $root 'Swan Company Server'
$data = Join-Path $root 'SwanCompanyServer'
New-Item -ItemType Directory -Path $root,$install,$data | Out-Null
$receiptPath = Join-Path $data 'installation.json'
$database = Join-Path $data 'management.sqlite3'
$binary = '"'+(Join-Path $install 'swan-management.exe')+'" --service'
$receipt = @{schema=1;service_name='SwanCompanyServer';install_directory=$install;data_directory=$data;full_stack=$true;firewall_rules=@('SwanCompanyServer-TCP','SwanCompanyServer-UDP')}
function Save-Receipt { [IO.File]::WriteAllText($receiptPath,($receipt | ConvertTo-Json -Depth 4)) }
function Plan { & (Join-Path $PSScriptRoot 'Get-ServerRemovalPlan.ps1') -InstallDirectory $install -DataDirectory $data -ServiceBinary $binary }
function Reject-Plan { $rejected=$false; try { Plan | Out-Null } catch {$rejected=$true}; if (-not $rejected) {throw 'Unsafe removal plan accepted'} }
try {
    [IO.File]::WriteAllText($database,'identity preservation fixture')
    Save-Receipt
    $plan = Plan
    if ($plan.Files.Count -ne 8 -or @($plan.Files | Where-Object {$_ -like ($data+'*')}).Count) {throw 'Removal includes company data or omits expected binaries'}
    $saved=$binary; $binary='"C:\Windows\System32\svchost.exe" --service'; Reject-Plan; $binary=$saved
    $binary=$saved+' --unexpected'; Reject-Plan; $binary=$saved
    $receipt.firewall_rules=@('Unrelated-Firewall-Rule'); Save-Receipt; Reject-Plan
    $receipt.firewall_rules=@('SwanCompanyServer-TCP','SwanCompanyServer-TCP'); Save-Receipt; Reject-Plan
    $receipt.firewall_rules=@('SwanCompanyServer-TCP','SwanCompanyServer-UDP'); $receipt.data_directory=Join-Path $root 'other-company'; Save-Receipt; Reject-Plan
    $receipt.data_directory=$data; $receipt.full_stack='false'; Save-Receipt; Reject-Plan
    $receipt.full_stack=$false; $receipt.firewall_rules=@(); Save-Receipt; $plan=Plan
    if ($plan.Files.Count -ne 1 -or $plan.FirewallRules.Count) {throw 'Management-only removal owns component resources'}
    $executableDirectory=Join-Path $install 'swan-management.exe'
    New-Item -ItemType Directory -Path $executableDirectory | Out-Null
    try { Reject-Plan } finally { [IO.Directory]::Delete($executableDirectory) }
    if ($env:OS -eq 'Windows_NT') {
        $target=Join-Path $root 'unrelated-directory'
        $junction=Join-Path $install 'components'
        New-Item -ItemType Directory -Path $target | Out-Null
        try {
            New-Item -ItemType Junction -Path $junction -Target $target | Out-Null
            try { Reject-Plan } finally { [IO.Directory]::Delete($junction) }
            if (-not (Test-Path -LiteralPath $target -PathType Container)) {throw 'Planning or junction cleanup removed its target'}
        } finally { [IO.Directory]::Delete($target) }
    }
    if ([IO.File]::ReadAllText($database) -ne 'identity preservation fixture') {throw 'Planning changed company identity'}
    Write-Output 'PASS: removal ownership checks reject wrong executable/arguments, company, rule names, duplicates, policy types, directories and junctions; company data is excluded.'
} finally {
    foreach ($file in @($receiptPath,$database)) {if (Test-Path -LiteralPath $file) {Remove-Item -LiteralPath $file}}
    foreach ($directory in @($install,$data,$root)) {Remove-Item -LiteralPath $directory}
}
