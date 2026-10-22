$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
# Provisioning is confined to the disposable GitHub Windows build runner.
# Local source builds require the selected toolset to be installed beforehand.
if($env:GITHUB_ACTIONS -cne 'true' -or $env:RUNNER_OS -cne 'Windows'){throw 'Toolset provisioning is only supported on the disposable GitHub Windows runner'}
$recipe=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'flutter-engine.lock.json') -Raw | ConvertFrom-Json
$tools=$recipe.windows_toolchain
if($tools.msvc_component -cne 'Microsoft.VisualStudio.Component.VC.14.38.17.8.x86.x64' -or $tools.msvc_tools_version -cne '14.38.33130'){throw 'Unexpected engine toolset component'}
$studio='C:\Program Files\Microsoft Visual Studio\2022\Enterprise'
$compiler=Join-Path $studio ('VC\Tools\MSVC\'+$tools.msvc_tools_version+'\bin\Hostx64\x64\cl.exe')
if(-not(Test-Path -LiteralPath $compiler)){
 $installer='C:\Program Files (x86)\Microsoft Visual Studio\Installer\setup.exe'
 $signature=Get-AuthenticodeSignature -LiteralPath $installer
 if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'CN=Microsoft Corporation(?:,|$)'){throw 'Visual Studio installer publisher verification failed'}
 $process=Start-Process -FilePath $installer -ArgumentList @('modify','--installPath',('"'+$studio+'"'),'--add',$tools.msvc_component,'--quiet','--norestart') -WindowStyle Hidden -Wait -PassThru
 if($process.ExitCode -ne 0){throw ('Pinned Microsoft toolset installation did not complete without restart: '+$process.ExitCode)}
}
foreach($relative in @('bin\Hostx64\x64\cl.exe','include\yvals_core.h','lib\x64\libcmt.lib','lib\x64\libcpmt.lib')){
 if(-not(Test-Path -LiteralPath (Join-Path $studio ('VC\Tools\MSVC\'+$tools.msvc_tools_version+'\'+$relative)))){throw 'Pinned Microsoft toolset is incomplete'}
}
Write-Output ('Pinned Microsoft C++ toolset available: '+$tools.msvc_tools_version)
