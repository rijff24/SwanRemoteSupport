# Technician MSI packaging

`msi/Technician.wxs` defines the generic per-user x64 technician MSI. Its fixed
upgrade and component identifiers do not depend on a company display name. It
installs the packed technician executable, agent, graphical-launcher script and
license under `%LOCALAPPDATA%/SwanRemoteSupport-Technician`, with a Start menu
shortcut. Upgrade removal is scheduled inside the MSI transaction. Company
state is not an MSI-owned file and is preserved through major upgrades.

Build with pinned WiX 5.0.2:

```powershell
./deployment/windows/Build-TechnicianMsi.ps1 `
  -WixExe C:/BuildTools/WiX/bin/wix.exe `
  -TechnicianExe C:/Release/SwanRemoteSupport-Technician.exe `
  -AgentExe C:/Release/swan-agent.exe `
  -Version 1.5.0 -Output C:/Release/SwanRemoteSupport-Technician-unsigned.msi
```

Production inputs require valid signatures from the same release publisher.
The builder itself produces an unsigned MSI; sign that container separately,
then compute release hashes and publish signed release metadata. `-UnsignedTest`
allows clearly marked local/CI test payloads and does not bypass installation,
worker or updater trust checks. No project signing key is provided to a company.

The official WiX package is pinned to SHA-256
`097383b2773bdd3d76a6a2cd3f4de6357bae59172e1b6fa5ad87ce062074e8d0`.
`Expand-WixBuildTools.ps1` reads its MSI tables and extracts tools without running
the supplier installer. `Build-TechnicianMsi.ps1` checks the extracted executable
publisher and exact tool version before execution. Native CI includes this
recipe and uploads the unsigned MSI with its build-input hashes.

A company worker consumes the approved unchanged MSI release and bundles it
with public company bootstrap data and canonical setup scripts. Technician
bundle downloads require company authentication. Run `Install-Company.ps1` to
validate, configure and record installation, rather than launching the bare MSI
and assuming it has enrolled or configured the app.

For a managed deployment, an explicit public bootstrap file can be provided:

```powershell
./Install-Company.ps1 -BootstrapPath C:/Deployment/company-bootstrap.json `
  -ConfirmedCompanyDomain support.example.com
```

This confirms the exact HTTPS hostname and retains signed endpoint/profile,
company ownership, hash, publisher and installed-identity checks. The bootstrap
contains no technician credentials. Without this explicit hostname, setup asks
for confirmation interactively. Unattended customer consent remains a separate
explicit action; this parameter cannot grant it.

The technician updater supports both portable EXEs and MSI containers. MSI
artifact and installed executable hashes are distinct signed fields. MSI
installation uses the canonical per-user folder and requires full installed
application/agent verification before the pending receipt is cleared.

Failed MSI updates can now enter durable technician rollback when the previous
signed release and agent both support rollback protocol 1. Recovery requires the
retained original MSI and complete verified snapshot. It checks the current
user's edition-specific upgrade family and cached product identities before
removing only the exact failed ProductCode, then installs or repairs the verified
previous package. Unexpected registrations stop recovery. Interrupted removal
or restoration leaves the signed receipt pending for retry; no company identity
or consent is restored from software snapshots. Hash, publisher, installed-file
and agent checks must pass before quarantining the failed release and clearing
the receipt. Customer MSI/service rollback remains separate unfinished work.

`Restore-TechnicianMsi.ps1 -InspectOnly` performs read-only identity and
registration planning and exits before invoking Windows Installer. It is a
diagnostic, not evidence that native rollback succeeds. Production recovery
uses the signed agent's prior hash/publisher checks and does not select this mode.

## Evidence and remaining work

An actual unsigned MSI was built locally with WiX `5.0.2+aa65968c` from the
verified native `31b13aa` artifact. The MSI tables contain the four expected files,
stable upgrade code, per-user location, major-upgrade behavior and Windows
launch conditions. A second build rehearsed the extraction/build scripts.
Packages and their hashes remain outside Git. No supplier installer or product
installer ran on this host during these builds. Rust product tests and
PowerShell parsing pass.

Clean-machine installation, company setup, repair, versioned major upgrades,
uninstall cleanup, signed automatic MSI updates and interruption rollback remain
unverified. Customer MSI authoring is available separately; end-to-end company
publication remains unverified. The bare MSI launch conditions do not replace the signed agent's
Windows compatibility preflight. Advertised Windows compatibility requires the
full acceptance matrix.

Sources: [WiX 5.0.2](https://github.com/wixtoolset/wix/releases/tag/v5.0.2),
[package scope](https://docs.firegiant.com/wix/schema/wxs/package/) and
[major-upgrade transaction scheduling](https://docs.firegiant.com/wix/schema/wxs/majorupgrade/).
