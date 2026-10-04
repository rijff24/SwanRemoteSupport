# Optional company signing

Company signing is a local operator-controlled integration. The project does
not provide certificates, approval, shared signing infrastructure or private
keys. Default distribution consumes unchanged project-signed artifacts.

`Sign-CompanyArtifact.ps1` signs a new copy of one locally prepared EXE or MSI.
Supply its independently verified input SHA-256, a new output path, the exact
company publisher, a certificate thumbprint in the local Personal store, a
Microsoft-signed Windows SDK SignTool path and the company's chosen HTTP or HTTPS
RFC3161 timestamp endpoint. Select `LocalMachine` only if that is where the
worker account's certificate is configured. The script does not import or
export keys, install trust roots, accept remote scripts or modify its input.

The certificate must have a private-key provider available to that account,
permit code signing and be current. Hardware providers may require their own
operator interaction. Provider-specific cloud signing is not implemented.
SignTool uses SHA-256 file and timestamp digests; subsequent verification pins
the trusted Authenticode publisher and certificate SHA-256 and requires a
timestamp. A mismatched input is rejected before signing.

SignTool verifies every signature under Authenticode policy with `/pa /all /tw`;
any error or warning prevents publication, including a missing timestamp. The
SDK check is authoritative for RFC3161 timestamps because the tested Server
2016 PowerShell reports no timestamp certificate even after SDK verification
confirms a valid RFC3161 timestamp.

The timestamp URL is explicit; the tool does not retry with a different scheme.
Credentials, query parameters and fragments are rejected so the public receipt
cannot carry timestamp-service credentials. RFC3161 responses remain signed
and trusted timestamp verification is required. The tested SDK rejected the
HTTPS DigiCert URL; DigiCert documents
[its HTTP RFC3161 endpoint](https://knowledge.digicert.com/general-information/rfc3161-compliant-time-stamp-authority-server).

The output `.signing.json` records the original and signed hashes, publisher,
certificate fingerprint and timestamp endpoint. It is evidence for preparing
release metadata, not a signed release manifest or administrator approval.
Sign installed endpoint/agent binaries before regenerating installed-file
manifests and building packages, then sign the final package. Regenerate every
affected hash after signing. Embedded company icons or metadata require a
company build and corresponding source; signing alone does not customize them.

For a new company using company-authored release metadata, explicitly provision
its chosen release-verification public key in the server/worker and endpoint
bootstrap. Set the signed release's publisher and certificate fingerprint to
the company values and include exact corresponding source. Import and approve
that release through management before the worker or updater consumes it.
Private release-signing keys remain on the company's release-authoring system.
Changing an existing deployment's pinned release issuer requires a separate
trusted migration; changing an environment variable does not authorize clients
to accept another issuer. Automated issuer migration remains incomplete.

A private Server 2016 VM now exercises actual software-provider signing with a
non-exportable lab key and a self-signed certificate trusted only in that VM.
The integration signed a copy of the native agent, obtained a DigiCert RFC3161
timestamp, passed SDK verification and matched the pinned publisher/certificate
through `Verify-Package.ps1`, including from a standard account. The unsigned
input stayed unchanged. An otherwise trusted signature without a timestamp
produced SDK warning exit `2` and was rejected. Tampered bytes, a different
publisher, a different certificate, and timestamp URLs containing credentials,
queries or fragments were rejected. Earlier wrong-input-hash rejection also
left the input unchanged and created no output.

This is isolated signing integration evidence, not production provider approval.
MSI signing, hardware/cloud providers, worker consumption and complete signed
install/update acceptance remain unverified. No lab signing key was exported or
added to the host's trust stores.

See [Microsoft SignTool documentation](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)
for certificate-store selection and RFC3161 signing options.
