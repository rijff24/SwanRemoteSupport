[CmdletBinding(DefaultParameterSetName='Package')]
param(
    [Parameter(Mandatory,ParameterSetName='Package')][string]$TransportPackage,
    [Parameter(Mandatory,ParameterSetName='Directory')][string]$TransportDirectory,
    [Parameter(Mandatory)][string]$HttpsPackage,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$PythonExecutable='python'
)
$ErrorActionPreference='Stop'
$verifier=Join-Path $PSScriptRoot '../transport/prepare_windows_components.py'
$arguments=@($verifier,'--https-package',$HttpsPackage,'--output',$OutputDirectory)
if($PSCmdlet.ParameterSetName -eq 'Package'){$arguments+=@('--transport-package',$TransportPackage)}else{$arguments+=@('--transport-directory',$TransportDirectory)}
$result=& $PythonExecutable @arguments
if($LASTEXITCODE -ne 0){throw 'Maintained transport or HTTPS verification failed; preserve any partial output for inspection.'}
$result|Set-Content -LiteralPath (Join-Path $OutputDirectory 'component-verification.json')
Write-Output ([IO.Path]::GetFullPath($OutputDirectory))
