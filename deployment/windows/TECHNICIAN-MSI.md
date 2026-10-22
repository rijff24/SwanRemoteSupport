# Technician MSI packaging

`msi/Technician.wxs` defines the generic per-user x64 technician MSI. Its fixed
upgrade and component identifiers do not depend on a company display name. It
installs the packed technician executable, agent, graphical-launcher script and
license under `%LOCALAPPDATA%/SwanRemoteSupport-Technician`, with a Start menu
shortcut. Upgrade removal is scheduled inside the MSI transaction. Company
state is not an MSI-owned file and is preserved through major upgrades.
Versions must use canonical ASCII `major.minor.build` fields without leading
zeroes (except `0`), within `255.255.65535`. Both builders reject aliases and
overflow before authoring product identities. These limits follow Microsoft's
[ProductVersion specification](https://learn.microsoft.com/windows/win32/msi/productversion).

ProductCode is a UUIDv5 derived from the stable edition UpgradeCode namespace
and `windows-x64|<software-version>`. Identical-version rebuilds therefore keep
the same product identity; a new software version gets a new identity. Company
names and branding are not identity inputs. The customer builder uses the same
rule with its own existing UpgradeCode. Stable ProductCode alone does not make
MSI bytes reproducible or qualify upgrade/recovery behavior.

`Set-ReproducibleMsiMetadata.ps1 -InputMsi <original> -OutputMsi <new-unsigned.msi>
-SourceDateEpoch <release-commit-epoch>` prepares a separate unsigned copy before
signing. It fixes summary timestamps, clears the permitted root-storage modified
time and derives a UUIDv5 PackageCode from the resulting canonical package hash.
Changes to package content or the epoch change the PackageCode. Existing outputs
and signed inputs are rejected. Originals remain intact. Both MSI builders enable
this step with `-SourceDateEpoch <release-commit-epoch>`, retaining the raw WiX
output and recording the metadata receipt. The Windows build workflow passes
its pinned source-commit epoch and runs the staging and metadata guard tests;
execution of that updated workflow remains unverified. The company worker does
not invoke this build step by default.

Two actual WiX 5.0.2 technician builds using the same `e6b87b9` payload produced
identical prepared MSI bytes at epoch `1792648409`: SHA-256
`68ea4ffe612d521a8354910381fb964b18624d413f73c207181bb068ae627855`.
Two subsequent full technician builder invocations with this option produced
identical final unsigned bytes, SHA-256
`bc349fff46cf371579e2f99d78ec11713768896531013f074358daf01cd76efb`.
The native metadata test passes under Windows PowerShell 5.1 and checks
idempotence, changed-epoch and changed-table package identities, overwrite
rejection, signed-input rejection and preservation of original files. This does
not establish independent CI reproducibility, signing reproducibility or
clean-machine installer lifecycles. Those remain release gates.

Metadata preparation alone did not remove input timestamps from cabinets: two
copies of identical executable/agent bytes dated in 2020 and 2025 initially
produced different MSI hashes. With `-SourceDateEpoch`, the builders now stage
byte-verified copies of every payload, launcher and license input with consistent
cabinet timestamps, without changing originals. Reparse points and destinations
inside the source tree are rejected. Repeating those different-timestamp
technician builds produced identical SHA-256
`96d33dd08f5f30a49d9d318017e7ff343f4e20f87f837ad0499b5ae4c4c709e5`.
The staging checks passed on Windows PowerShell 5.1. Cross-time-zone builders
and independently downloaded binary inputs remain unverified.

Metadata format references: Microsoft's [package codes](https://learn.microsoft.com/en-us/windows/win32/msi/package-codes),
[summary revision](https://learn.microsoft.com/en-us/windows/win32/msi/revision-number-summary)
and [compound-file root directory](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/026fde6e-143d-41bf-a7da-c08b2130d50e).

Windows Installer policy can prohibit unmanaged per-user installations. A
standard-user test of the signed `e6b87b9` package on Server 2022 build
`20348.587` returned error 1625: the MSI log reported effective `DisableMsi=1`,
although no explicit registry policy value was present. No payload or per-user
registration remained. Repair and uninstall were not run after that failure.
An absent registry value therefore does not establish that installation is
permitted. Company setup reports this policy rejection and directs the user to
their administrator for an approved deployment; it does not change the policy
or silently retry with elevated privileges. See Microsoft's
[DisableMSI documentation](https://learn.microsoft.com/en-us/windows/win32/msi/disablemsi).

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
the receipt. The same registration recovery now supports customer MSI releases,
with additional service and configuration-task ownership checks.

`Restore-ReleaseMsi.ps1 -Edition technician -InspectOnly` performs read-only identity and
registration planning and exits before invoking Windows Installer. It is a
diagnostic, not evidence that native rollback succeeds. Production recovery
uses the signed agent's prior hash/publisher checks and does not select this mode.

A separate staged recovery task implements logon and periodic retry even when
the installed agent has been removed during rollback. Explicit MSI uninstall
cancels recovery before removing its files; the paired uninstall rollback
restores the prior cancellation state. See [update recovery](UPDATE-RECOVERY.md).
Task execution and reboot recovery remain native acceptance gates.

## Evidence and remaining work

An actual unsigned MSI was built locally with WiX `5.0.2+aa65968c` from the
verified native `31b13aa` artifact. The MSI tables contain the four expected files,
stable upgrade code, per-user location, major-upgrade behavior and Windows
launch conditions. A second build rehearsed the extraction/build scripts.
Packages and their hashes remain outside Git. No supplier installer or product
installer ran on this host during these builds. Rust product tests and
PowerShell parsing pass.

A separate disposable administrator on Server 2022 build `20348.587` passed
install, missing-executable repair and uninstall for the stable-identity 1.5.0
MSI built with recipe `67786dd` and verified `e6b87b9` lab-signed payloads. The
MSI was unsigned (SHA-256
`f3cb6e56229488b9d6d4dec0e07318a5520b82ee66177435ae358c98faf348e7`).
All phases returned zero, and execution policy stayed unchanged. This does not
supersede the standard-user policy failure described above.

That isolated administrative profile also passed reinstall after uninstall,
the MSI 1.5.0-to-1.5.1 major upgrade, and uninstall of 1.5.1. All operations
returned zero; old/new product registration, payload hashes, shortcut and an
unowned fixture file were checked. Both packages contained the same application
payload. This does not demonstrate an application update or preservation of
real enrollment/consent state.

The full clean-machine matrix, company setup, upgrades between different native
application builds, signed automatic MSI updates and interruption rollback remain unverified.
Customer MSI authoring is available separately; end-to-end company
publication remains unverified. The bare MSI launch conditions do not replace the signed agent's
Windows compatibility preflight. Advertised Windows compatibility requires the
full acceptance matrix.

Sources: [WiX 5.0.2](https://github.com/wixtoolset/wix/releases/tag/v5.0.2),
[package scope](https://docs.firegiant.com/wix/schema/wxs/package/) and
[major-upgrade transaction scheduling](https://docs.firegiant.com/wix/schema/wxs/majorupgrade/).
