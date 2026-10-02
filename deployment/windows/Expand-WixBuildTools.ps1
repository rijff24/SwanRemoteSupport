[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Package,[Parameter(Mandatory=$true)][string]$OutputDirectory,[string]$SevenZip='7z.exe')
$ErrorActionPreference = 'Stop'
$Package = (Resolve-Path -LiteralPath $Package).Path
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Tool output directory already exists.' }
if ((Get-FileHash -LiteralPath $Package -Algorithm SHA256).Hash -ne '097383B2773BDD3D76A6A2CD3F4DE6357BAE59172E1B6FA5AD87CE062074E8D0') { throw 'Pinned WiX 5.0.2 package hash mismatch.' }
$raw = $OutputDirectory + '.raw'
if (Test-Path -LiteralPath $raw) { throw 'Raw extraction directory already exists.' }
& $SevenZip x $Package ('-o'+$raw) -y | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'WiX extraction failed.' }
# Read MSI tables; never execute the supplier installer or its custom actions.
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.OpenDatabase($Package,0)
$directories = @{}
$components = @{}
$view = $database.OpenView('SELECT `Directory`,`Directory_Parent`,`DefaultDir` FROM `Directory`')
$view.Execute()
while ($row = $view.Fetch()) { $directories[$row.StringData(1)] = @($row.StringData(2),$row.StringData(3)) }
$view.Close()
$view = $database.OpenView('SELECT `Component`,`Directory_` FROM `Component`')
$view.Execute()
while ($row = $view.Fetch()) { $components[$row.StringData(1)] = $row.StringData(2) }
$view.Close()
$view = $database.OpenView('SELECT `File`,`Component_`,`FileName` FROM `File`')
$view.Execute()
while ($row = $view.Fetch()) {
    $parts = [Collections.Generic.List[string]]::new()
    $directory = $components[$row.StringData(2)]
    $depth = 0
    while ($directory -ne 'INSTALLFOLDER' -and $directory -and $directories.ContainsKey($directory)) {
        if (++$depth -gt 16) { throw 'Unexpected supplier directory depth.' }
        $name = ($directories[$directory][1] -split '\|')[-1]
        if ($name -ne '.') { $parts.Insert(0,$name) }
        $directory = $directories[$directory][0]
    }
    # Extension packages installed to separate user storage are not needed here.
    if ($directory -ne 'INSTALLFOLDER') { continue }
    $parts.Add(($row.StringData(3) -split '\|')[-1])
    $destination = [IO.Path]::GetFullPath((Join-Path $OutputDirectory ($parts -join '\')))
    if (-not $destination.StartsWith($OutputDirectory+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Supplier path escaped the tool directory.' }
    New-Item -ItemType Directory -Force ([IO.Path]::GetDirectoryName($destination)) | Out-Null
    Copy-Item -LiteralPath (Join-Path $raw $row.StringData(1)) -Destination $destination
}
$view.Close()
$tool = Join-Path $OutputDirectory 'bin/wix.exe'
$signature = Get-AuthenticodeSignature -LiteralPath $tool
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'CN=WiX Toolset \(\.NET Foundation\)') { throw 'Extracted WiX publisher verification failed.' }
Write-Output $tool
