[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Directory,[Parameter(Mandatory=$true)][ValidateSet('customer','technician')][string]$Edition)
$ErrorActionPreference='Stop'
function Complete-Uninstall {
    param([Parameter(Mandatory=$true)][string]$Directory)
    $folder=Get-Item -LiteralPath $Directory
    if (-not $folder.PSIsContainer -or ($folder.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe uninstall state directory.' }
    if (Test-Path -LiteralPath (Join-Path $Directory 'swan-agent.exe')) { throw 'Uninstall commit requires the installed agent to be removed.' }
    $marker=Get-Item -LiteralPath (Join-Path $Directory 'pending-uninstall')
    if ($marker.PSIsContainer -or ($marker.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Invalid uninstall cancellation marker.' }
    $records=@()
    foreach ($name in @('uninstall-cancellation-rollback.json','configuration-task-rollback.json')) {
        $path=Join-Path $Directory $name
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $file=Get-Item -LiteralPath $path
        if ($file.PSIsContainer -or ($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $file.Length -gt 4096) { throw 'Invalid uninstall rollback record.' }
        $record=Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
        if ($name -eq 'uninstall-cancellation-rollback.json') {
            if ($record.marker_existed -isnot [bool]) { throw 'Invalid cancellation rollback state.' }
        } elseif ($record.enabled -isnot [bool] -or $record.running -isnot [bool]) { throw 'Invalid task rollback state.' }
        $records+=$path
    }
    # All validation precedes deletion. Keep the cancellation marker so staged
    # update recovery cannot reinstall software the customer just uninstalled.
    foreach ($path in $records) { Remove-Item -LiteralPath $path }
}
$expected=if($Edition -eq 'customer'){Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) 'SwanRemoteSupport'}else{Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'SwanRemoteSupport-Technician'}
if ([IO.Path]::GetFullPath($Directory).TrimEnd('\') -ine [IO.Path]::GetFullPath($expected).TrimEnd('\')) { throw 'Uninstall commit requires the canonical edition state directory.' }
Complete-Uninstall -Directory $Directory
