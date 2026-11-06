[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$ProductCode, [Parameter(Mandatory=$true)][string]$Version)
$ErrorActionPreference = 'Stop'
$installer = New-Object -ComObject WindowsInstaller.Installer
if ($installer.ProductState($ProductCode) -ne 5 -or $installer.ProductInfo($ProductCode, 'VersionString') -cne $Version) {
    throw 'The expected MSI product and version are not installed.'
}
