[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Package)
$ErrorActionPreference = 'Stop'
# Read only: no installer actions or custom actions are executed.
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.OpenDatabase((Resolve-Path -LiteralPath $Package).Path, 0)
$view = $database.OpenView('SELECT `Value` FROM `Property` WHERE `Property` = ''ProductCode''')
$null = $view.Execute()
$row = $view.Fetch()
if ($null -eq $row) { throw 'MSI has no product identity.' }
$code = $row.StringData(1)
$null = $view.Close()
if ($code -notmatch '^\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\}$') { throw 'Invalid MSI product identity.' }
$state = $installer.ProductState($code)
if ($state -eq 5) { Write-Output '/fvamus' }
elseif ($state -in @(-1, 1)) { Write-Output '/i' }
else { throw "MSI product registration cannot be safely recovered (state $state)." }
