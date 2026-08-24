# Windows code signing and Defender release process

## Current release gate

The Swan client is not safe to describe as antivirus-friendly merely because it compiles successfully. Windows evaluates the downloaded installer, the executables and DLLs extracted from it, their signatures, and their reputation. Remote-control software also receives additional scrutiny because the same capabilities are abused by attackers.

The current GitHub Actions workflow intentionally produces an **unsigned build artifact**. It is suitable for isolated development testing only. Do not give it to customers as the production installer.

Current primary references: [Microsoft SmartScreen reputation guidance](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation), [Microsoft Artifact Signing eligibility and setup](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart), [SignPath Foundation conditions](https://signpath.org/terms.html), and the [Microsoft Security Intelligence file-submission portal](https://www.microsoft.com/en-us/wdsi/filesubmission).

The repository's release rules are in the public [Swan code-signing policy](CODE_SIGNING_POLICY.md). Free code signing provided by SignPath.io, certificate by SignPath Foundation.

## What signing can and cannot do

A trusted Authenticode signature proves which verified publisher signed a file and detects changes after signing. It materially improves Windows trust decisions, but it does not guarantee that SmartScreen or antivirus software will never warn. New certificates and new file hashes can still have little reputation at first.

Do not use a self-signed certificate for customer distribution. Microsoft documents self-signed applications as having the same initial SmartScreen warning behavior as unsigned applications. Importing a self-signed root certificate on each customer PC would create a private trust arrangement, but it does not establish public publisher reputation and increases certificate-management risk.

## Realistic signing choices for Swan Computing

### 1. SignPath Foundation for an open-source public release

This is the first option to investigate if the Swan fork and every Swan component will remain publicly available under an OSI-approved licence.

SignPath Foundation advertises free signing for eligible open-source projects, but approval is not automatic. Its current conditions include a released and actively maintained public project, documented behavior, no proprietary components, MFA, defined author/reviewer/approver roles, a published code-signing policy, and verifiable builds from the linked source repository. The certificate publisher will be SignPath Foundation rather than Swan Computing.

Required preparation:

1. Publish the exact Swan source in a Swan-controlled public GitHub repository.
2. Add a public download/release page, privacy statement, source link, and code-signing policy.
3. Enable MFA for every repository and signing participant.
4. Define reviewers and signing approvers.
5. Apply to SignPath Foundation and wait for acceptance.
6. Design a two-stage signing pipeline: sign the inner executables/DLLs, create the self-extracting installer, then sign and timestamp the outer installer.
7. Verify that SignPath can support the custom self-extracting format. If it cannot, package the signed client as an MSI or another supported container.

The official Tailscale Windows MSI is not embedded in or signed as part of Swan. The bootstrap downloads it from Tailscale and requires its pinned SHA-256 plus a valid `Tailscale Inc.` Authenticode signature. This boundary avoids representing Tailscale's proprietary Windows GUI as a Swan open-source component; SignPath must still review and accept the complete bootstrap behavior.

### 2. Buy an organization-validated code-signing certificate

Choose this when the Windows publisher should identify the verified Swan legal entity. Obtain an OV code-signing certificate or managed signing service from a certification authority in Microsoft's trusted-root ecosystem. Current industry rules generally require protected private-key storage such as a hardware token or managed HSM; follow the selected provider's exact process.

Do not pay an EV premium solely because of an expectation that EV instantly bypasses SmartScreen. Microsoft's current guidance says OV and EV certificates can both show an initial unrecognized-app warning and that EV no longer receives automatic SmartScreen reputation.

The signing service must be integrated into CI or an isolated release machine so it can sign the inner binaries before packaging and the final installer afterward. Never upload a signing PFX or password into this repository.

### 3. Microsoft Artifact Signing

Microsoft recommends Artifact Signing for non-Store distribution where Public Trust is available. However, Microsoft's current eligibility documentation does not list South African organizations for Public Trust. Private Trust would only work on devices where Swan controls and deploys the private trust policy; it would not solve general customer-download reputation. Recheck Microsoft's official eligibility before each future decision because geographic availability can change.

## Required release pipeline

The production sequence must be:

1. Build from the reviewed Swan source and pinned dependencies.
2. Scan the unpacked build outputs.
3. Sign and timestamp every Swan-authored `.exe` and `.dll` in the client directory.
4. Verify each inner signature before packaging.
5. Build the final self-extracting installer from those signed files.
6. Sign and timestamp the outer installer.
7. Verify the outer signature and calculate its SHA-256.
8. Test installation, launch, service startup, remote connection, reboot, uninstall, and every executable code path on a clean Windows 10/11 test PC.
9. Run Microsoft Defender's custom scan against the final release.
10. If Defender reports a false positive, stop distribution and submit the exact signed file to Microsoft's Security Intelligence submission portal. Wait for the result; do not tell customers to disable antivirus.
11. Publish the signed installer, SHA-256, exact corresponding source, version, privacy information, and rollback instructions over HTTPS.

Basic signature inspection:

```powershell
Get-AuthenticodeSignature -LiteralPath .\SwanRemoteSupport-1.4.9-x64-install.exe | Format-List
Get-FileHash -Algorithm SHA256 -LiteralPath .\SwanRemoteSupport-1.4.9-x64-install.exe
```

The expected signature status is `Valid`, the signer subject must match the approved certificate identity, and the timestamp must remain valid after the leaf certificate expires.

## Reputation expectations

Even a correctly signed first release can show **Windows protected your PC** while reputation is new. Use the same trusted signing identity for every clean release, publish from a stable HTTPS domain, avoid packers/obfuscators beyond the existing documented installer format, do not modify anything after signing, and never ask users to disable SmartScreen or antivirus.

For the few-client in-person deployment described for this project, explain the new-publisher possibility before installation and verify the publisher and SHA-256 on site. A warning is not proof of malware, but it is also not something to bypass without verifying the exact signed release.
