[CmdletBinding(SupportsShouldProcess)]
param([switch]$Confirmed)
$ErrorActionPreference='Stop'
$directory=[IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'SwanRemoteSupport-Technician'))
if([IO.Path]::GetFullPath($PSScriptRoot) -ine $directory){throw 'Run the installed technician uninstaller.'}
foreach($path in @($env:LOCALAPPDATA,$directory)){
    $item=Get-Item -LiteralPath $path -Force
    if(-not $item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Technician installation directory is unsafe.'}
}
$metadata=Get-Content -LiteralPath (Join-Path $directory 'installed-release.json') -Raw|ConvertFrom-Json
$release=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($metadata.payload))|ConvertFrom-Json
if($release.edition -cne 'technician' -or $release.format -cne 'exe'){throw 'Use Windows installed-app removal for MSI installations.'}
$agent=Join-Path $directory 'swan-agent.exe'
if((Get-AuthenticodeSignature -LiteralPath $agent).Status -ne 'Valid'){throw 'Repair the configuration agent before uninstalling.'}
$env:SWAN_STATE_DIR=$directory
& $agent verify-installed
if($LASTEXITCODE -ne 0){throw 'Repair the installed identity before uninstalling.'}
if(-not $Confirmed -and -not $WhatIfPreference){
    Add-Type -AssemblyName System.Windows.Forms
    if([Windows.Forms.MessageBox]::Show('Remove the technician application? Close it first. Company configuration is retained for reinstall.','Remove technician application','YesNo','Question') -ne 'Yes'){return}
}
if(-not $PSCmdlet.ShouldProcess($directory,'Remove portable technician application and retain company state')){return}
$lock=[IO.File]::Open((Join-Path $directory 'activity.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite)
$locked=$false
try{
    $lock.Lock(0,1);$locked=$true
    & $agent cancel-update-recovery
    if($LASTEXITCODE -ne 0){throw 'Update recovery cancellation failed; no application files were removed.'}
    # Fixed regular files only: never recurse into company state, releases or
    # extracted runtimes shared with another installation.
    foreach($name in @('SwanRemoteSupport-Technician.exe','Open-Technician.ps1','swan-agent.exe')){
        $path=Join-Path $directory $name
        if(Test-Path -LiteralPath $path){
            $item=Get-Item -LiteralPath $path -Force
            if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Unexpected application file; removal remains pending.'}
            Remove-Item -LiteralPath $path -Force
        }
    }
    foreach($name in @('Swan Remote Support Technician.lnk','Uninstall Swan Remote Support Technician.lnk')){
        $path=Join-Path ([Environment]::GetFolderPath('Programs')) $name
        if(Test-Path -LiteralPath $path){
            $shell=New-Object -ComObject WScript.Shell
            $shortcut=$shell.CreateShortcut($path)
            if($shortcut.Arguments.Contains('"'+(Join-Path $directory 'Open-Technician.ps1')+'"') -or $shortcut.Arguments.Contains('"'+(Join-Path $directory 'Uninstall-Technician.ps1')+'"')){Remove-Item -LiteralPath $path}
        }
    }
    Write-Output 'Technician application removed. Company state is retained and automatic recovery is cancelled.'
}finally{if($locked){$lock.Unlock(0,1)};$lock.Dispose()}
