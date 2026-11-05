param([Parameter(Mandatory=$true)][string]$ManagementExecutable,[Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$executable=(Resolve-Path -LiteralPath $ManagementExecutable).Path
$directory=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $directory){throw 'Use a new network-probe test directory.'}
New-Item -ItemType Directory -Path $directory | Out-Null
$authority=$null;$certificate=$null
try {
    # Disposable issuer only; never install it into a trusted root store.
    $authority=New-SelfSignedCertificate -Type Custom -Subject 'CN=Swan disposable probe CA' -CertStoreLocation Cert:\CurrentUser\My -KeyExportPolicy Exportable -KeyAlgorithm RSA -KeyLength 2048 -HashAlgorithm SHA256 -NotAfter (Get-Date).AddDays(2) -KeyUsage CertSign,CRLSign,DigitalSignature -TextExtension @('2.5.29.19={critical}{text}ca=1')
    $certificate=New-SelfSignedCertificate -Type Custom -Subject 'CN=localhost' -DnsName localhost -Signer $authority -CertStoreLocation Cert:\CurrentUser\My -KeyExportPolicy Exportable -KeyAlgorithm RSA -KeyLength 2048 -HashAlgorithm SHA256 -NotAfter (Get-Date).AddDays(1) -KeyUsage DigitalSignature,KeyEncipherment -TextExtension @('2.5.29.19={critical}{text}ca=0','2.5.29.37={text}1.3.6.1.5.5.7.3.1')
    $password=[Guid]::NewGuid().ToString('N')
    $pfx=Join-Path $directory 'localhost.pfx';$ca=Join-Path $directory 'root.cer'
    Export-PfxCertificate -Cert $certificate -FilePath $pfx -Password (ConvertTo-SecureString $password -AsPlainText -Force) | Out-Null
    Export-Certificate -Cert $authority -FilePath $ca | Out-Null
    $environment=Join-Path $directory 'test.private.env'
    [IO.File]::WriteAllText($environment,"SWAN_TEST_TLS_PFX=$pfx`nSWAN_TEST_TLS_PASSWORD=$password`nSWAN_TEST_CA_FILE=$ca`n")
    & node (Join-Path $PSScriptRoot 'test-network-probe.js') $environment $executable (Join-Path $directory 'verified.json')
    if($LASTEXITCODE -ne 0){throw 'Compiled connection checker regression failed.'}
} finally {
    if($certificate){Remove-Item -LiteralPath ('Cert:\CurrentUser\My\'+$certificate.Thumbprint)}
    if($authority){Remove-Item -LiteralPath ('Cert:\CurrentUser\My\'+$authority.Thumbprint)}
    foreach($name in @('localhost.pfx','test.private.env')){
        $file=Join-Path $directory $name
        if(Test-Path -LiteralPath $file){Remove-Item -LiteralPath $file}
    }
}
