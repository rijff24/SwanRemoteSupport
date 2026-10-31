param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$directory=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $directory){throw 'Use a new private HTTPS test directory.'}
New-Item -ItemType Directory -Path $directory | Out-Null
$certificate=$null;$authority=$null
try {
    # Disposable CI identity only. Never add it to a trusted root store.
    $authority=New-SelfSignedCertificate -Type Custom -Subject 'CN=Swan disposable download test CA' -CertStoreLocation Cert:\CurrentUser\My -KeyExportPolicy Exportable -KeyAlgorithm RSA -KeyLength 2048 -HashAlgorithm SHA256 -NotAfter (Get-Date).AddDays(2) -KeyUsage CertSign,CRLSign,DigitalSignature -TextExtension @('2.5.29.19={critical}{text}ca=1')
    $certificate=New-SelfSignedCertificate -Type Custom -Subject 'CN=localhost' -DnsName localhost -Signer $authority -CertStoreLocation Cert:\CurrentUser\My -KeyExportPolicy Exportable -KeyAlgorithm RSA -KeyLength 2048 -HashAlgorithm SHA256 -NotAfter (Get-Date).AddDays(1) -KeyUsage DigitalSignature,KeyEncipherment -TextExtension @('2.5.29.19={critical}{text}ca=0','2.5.29.37={text}1.3.6.1.5.5.7.3.1')
    $password=[Guid]::NewGuid().ToString('N')
    $pfx=Join-Path $directory 'localhost.pfx';$ca=Join-Path $directory 'localhost.cer'
    Export-PfxCertificate -Cert $certificate -FilePath $pfx -Password (ConvertTo-SecureString $password -AsPlainText -Force) | Out-Null
    Export-Certificate -Cert $authority -FilePath $ca | Out-Null
    $environment=Join-Path $directory 'test.private.env'
    [IO.File]::WriteAllText($environment,"SWAN_TEST_TLS_PFX=$pfx`nSWAN_TEST_TLS_PASSWORD=$password`nSWAN_TEST_CA_FILE=$ca`n")
    $lines=& cargo +1.90.0 test --manifest-path product/Cargo.toml --locked -p swan-agent --lib --no-run --message-format=json
    if($LASTEXITCODE -ne 0){throw 'Agent test compilation failed.'}
    $executables=@($lines | ForEach-Object { $item=$_ | ConvertFrom-Json; if($item.reason -eq 'compiler-artifact' -and $item.target.name -eq 'swan_agent' -and $item.profile.test -and $item.executable){$item.executable} })
    if($executables.Count -ne 1){throw 'Expected exactly one compiled agent library test executable.'}
    & node (Join-Path $PSScriptRoot 'test-update-download.js') $environment $executables[0] (Join-Path $directory 'verified.json')
    if($LASTEXITCODE -ne 0){throw 'Real HTTPS download verification failed.'}
} finally {
    if($certificate){Remove-Item -LiteralPath ('Cert:\CurrentUser\My\'+$certificate.Thumbprint)}
    if($authority){Remove-Item -LiteralPath ('Cert:\CurrentUser\My\'+$authority.Thumbprint)}
    foreach($name in @('localhost.pfx','test.private.env')){
        $file=Join-Path $directory $name
        if(Test-Path -LiteralPath $file){Remove-Item -LiteralPath $file}
    }
}
