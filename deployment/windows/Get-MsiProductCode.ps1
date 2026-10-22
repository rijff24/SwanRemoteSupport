[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][ValidateSet('customer','technician')][string]$Edition,
    [Parameter(Mandatory=$true)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version
)
$ErrorActionPreference='Stop'
# RFC 4122 UUIDv5: the stable edition UpgradeCode is the namespace. The product
# changes with its Windows x64 software version, never with company branding.
$namespace=if($Edition -eq 'customer'){'32a585d7-9a78-4ad2-af72-d0266efc709d'}else{'a4374699-436f-4917-9a4e-223f62a9e634'}
$bytes=([Guid]$namespace).ToByteArray()
foreach($segment in @(@(0,4),@(4,2),@(6,2))){[Array]::Reverse($bytes,$segment[0],$segment[1])}
$name=[Text.Encoding]::UTF8.GetBytes('windows-x64|'+$Version)
$sha=[Security.Cryptography.SHA1]::Create()
try{$digest=$sha.ComputeHash([byte[]]($bytes+$name))}finally{$sha.Dispose()}
[byte[]]$result=$digest[0..15]
$result[6]=($result[6] -band 15) -bor 80
$result[8]=($result[8] -band 63) -bor 128
foreach($segment in @(@(0,4),@(4,2),@(6,2))){[Array]::Reverse($result,$segment[0],$segment[1])}
([Guid]::new($result)).ToString('D').ToUpperInvariant()
