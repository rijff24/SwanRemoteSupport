[CmdletBinding(SupportsShouldProcess=$true)]
param()
$ErrorActionPreference='Stop'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(-not ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Worker removal requires administrator rights.'}
$install=[IO.Path]::GetFullPath((Join-Path $env:ProgramFiles 'Swan Installer Worker'))
$data=[IO.Path]::GetFullPath((Join-Path $env:ProgramData 'SwanInstallerWorker'))
$executable=Join-Path $install 'swan-worker.exe'
foreach($path in @($env:ProgramFiles,$env:ProgramData,$install,$data,$executable,(Join-Path $data 'installation.json'))){
 if((Get-Item -LiteralPath $path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Worker removal paths cannot be reparse points.'}
}
$service=Get-CimInstance Win32_Service -Filter "Name='SwanInstallerWorker'"
if(-not $service -or $service.PathName -cne ('"'+$executable+'" --service') -or $service.StartName -notin @('LocalSystem','NT AUTHORITY\SYSTEM')){throw 'Worker service ownership mismatch.'}
$receipt=Get-Content -LiteralPath (Join-Path $data 'installation.json') -Raw|ConvertFrom-Json
if($receipt.schema -ne 1 -or $receipt.service -cne 'SwanInstallerWorker' -or $receipt.executable_sha256 -ine (Get-FileHash -LiteralPath $executable).Hash){throw 'Worker installation receipt or executable changed.'}
$signature=Get-AuthenticodeSignature -LiteralPath $executable
if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ine $receipt.publisher_thumbprint){throw 'Worker publisher no longer matches the installation receipt.'}
if($PSCmdlet.ShouldProcess('SwanInstallerWorker','Stop and remove the owned service and executable; retain private artifacts and receipt')){
 $controller=Get-Service SwanInstallerWorker
 try{if($controller.Status -ne [ServiceProcess.ServiceControllerStatus]::Stopped){Stop-Service SwanInstallerWorker;$controller.WaitForStatus([ServiceProcess.ServiceControllerStatus]::Stopped,[TimeSpan]::FromSeconds(30))}}finally{$controller.Dispose()}
 if($service.ProcessId -ne 0 -and (Get-CimInstance Win32_Process -Filter "ProcessId=$($service.ProcessId)")){throw 'Worker process remains alive; retain all files.'}
 & sc.exe delete SwanInstallerWorker|Out-Null
 if($LASTEXITCODE -ne 0){throw 'Worker service deletion failed; retain all files.'}
 Remove-Item -LiteralPath $executable
 if(-not @(Get-ChildItem -LiteralPath $install -Force).Count){Remove-Item -LiteralPath $install}
 Write-Host 'Worker service and executable removed. Protected artifacts and receipt remain for inspection.'
 Write-Host 'The original private configuration file still contains the worker credential. Worker credential revocation/replacement is a separate administrative operation.'
}
