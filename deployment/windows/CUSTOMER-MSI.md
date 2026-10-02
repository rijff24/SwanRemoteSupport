# Customer MSI packaging

`generate-customer-msi.py` authors a per-machine x64 MSI from the complete
project Flutter payload. It uses stable project upgrade and component identifiers,
independent of company display names. The canonical application path is
`%ProgramFiles%/Swan Remote Support/Swan Remote Support.exe`; the agent installs
under `%ProgramData%/SwanRemoteSupport`. Company profiles, device identity and
consent are not MSI-owned files and remain separate from release files.

The package includes all payload libraries/assets and a local AGPL license
notice. Its signed-release manifest covers those installed files. Windows
Installer owns service installation, start/stop/removal and transaction-based
major-upgrade removal. The service runs the project executable with `--service`
as LocalSystem. Windows Installer copies the local operating system's
RuntimeBroker into the application directory and removes that copy on uninstall;
it does not redistribute a project copy of Microsoft's binary. Runtime verification
compares its hash with the current Windows system binary.

```powershell
./deployment/windows/Build-CustomerMsi.ps1 `
  -WixExe C:/BuildTools/WiX/bin/wix.exe -SourceDirectory C:/Release/swan-client `
  -AgentExe C:/Release/swan-agent.exe -Version 1.5.0 `
  -Output C:/Release/SwanRemoteSupport-Customer-unsigned.msi
```

The recipe uses pinned WiX 5.0.2 and verifies its tool publisher/version.
Production executable inputs need valid signatures from the same publisher.
The builder produces an unsigned MSI, generated authoring, installed-file
manifest and build-input hashes. Sign the MSI separately and publish signed
release metadata before production approval. `-UnsignedTest` permits clearly
marked test payloads; company workers/installers/updaters retain their signature
checks. Regenerate hashes and metadata after final signing.

Companies consume the unchanged signed project MSI through their Windows
worker and distribute its public customer bundle. `Install-Company.ps1` confirms
the company HTTPS identity, verifies trust and the package, protects the state
directory, enrolls the device and enables configuration synchronization.
`-BootstrapPath` plus `-ConfirmedCompanyDomain` provides explicit managed
configuration input without embedding credentials. The bare MSI does not enroll
or silently select a company. Full deployment must use the company setup flow;
new devices stay unavailable until approved.

## Evidence and remaining verification

A real unsigned package was built from the hash-verified native `31b13aa`
artifact. Its complete 95-file payload was recovered without executing any
artifact binary; compressed-file checksums were verified. The final package
adds the project license and agent. Windows Installer tables and generated
manifests were inspected, and the CLI build recipe succeeded with pinned WiX.
Three authoring tests cover full asset/license manifests, service lifecycle,
stable component identities, unsafe names/versions and incomplete payloads.
The supplier tool was extracted through MSI-table reads, not installed on the
host. Packages, extraction records and hashes remain outside Git.

No customer installer ran on this host. Signed clean-machine installation,
service startup, SYSTEM enrollment, consent, UAC/logon-screen behavior, MSI
repair/major upgrades, runtime-broker servicing, task cleanup, uninstall and
interrupted-install rollback remain unverified. Native build CI now includes
both MSI recipes; its exact results must be recorded separately. MSI service
rules and a successful build do not establish these acceptance conditions.

Customer MSI uninstall now invokes its installed configuration agent under
SYSTEM before file removal to remove the separately registered configuration
task. The task's executable path, `watch` arguments and SYSTEM principal must
match this installation; an unrelated task is refused. Company identity and
consent files remain protected for explicit reinstallation. Major upgrades skip
task removal. A paired MSI rollback action restores the previous enabled/running
state after an interrupted uninstall, using protected rollback state and a
validly signed restored agent. A task absent before cleanup is not created by
rollback. The agent compiles on Windows and both scripts parse; three authoring
tests pass. Pinned WiX compiled a real unsigned package and its Installer tables
confirm rollback, cleanup and file-removal order. Actual task deletion, MSI
rollback and reinstall on clean guests remain unverified.

Failed software updates can now restore the previous customer MSI when both
previous signed release metadata and its agent support rollback protocol 1.
Recovery uses the complete verified snapshot and original installer, restricts
registration removal to the exact failed ProductCode and checks the customer
service's exact executable path and LocalSystem account before removal. The
restored service must run automatically. The verified previous agent restores
only its owned SYSTEM configuration task, deferring startup until recovery
finishes. Enrollment and consent files remain current. Unexpected registration,
service or task ownership leaves recovery pending. A separate staged recovery
task now implements startup and periodic retry across removal of the installed
agent; see [update recovery](UPDATE-RECOVERY.md). Customer EXE/service rollback
remains unfinished. This path has component and read-only planning evidence only; native
signed installation, removal, restore and restart remain acceptance gates.

References: [WiX service installation](https://docs.firegiant.com/wix/schema/wxs/serviceinstall/)
and [Windows-owned file copying](https://docs.firegiant.com/wix/schema/wxs/copyfile/).
