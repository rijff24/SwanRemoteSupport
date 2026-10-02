# Configurable product acceptance report

Development status, 1 October 2026. **The product is not complete or approved for
production publication.** A passing component test is not evidence of a working
native support session or a clean-machine installation.

## Evidence obtained

- Public baseline tag `swan-single-company-baseline-1.4.9` preserves the original
  Swan deployment and build instructions. Its source was not replaced.
- Twenty-three product tests pass on the local Windows development host, including
  enrollment/trust preservation across stale writes, trusted key rotation, refusal to automatically reinstall a removed endpoint, and explicit consent changes surviving older refreshes. Consent API tests also reject delayed enable requests and older revocation retries; same-revision revocation takes precedence.
  The preceding eleven-test component source passed tests and release compilation on
  [Windows 2022 and Ubuntu 24.04 CI](https://github.com/rijff24/SwanRemoteSupport/actions/runs/36856716860)
  at commit `411269b`.
- A fresh, isolated company passed real HTTPS tests against newly built agent
  and management executables: setup/TOTP, enrollment, pending-device denial,
  technician login and group inventory, own-session history, logout token
  revocation, unattended-consent denial, single-use grants, signed branding
  revision synchronization and device revocation. Exact executable hashes and
  private results stay under the Git-ignored local test data directory. This
  test uses a test RustDesk ID and proves **no native remote desktop behavior**.
- Flutter bridge generation passed for the technician request stream. Native
  Rust parsing and changed PowerShell parsing passed. The graphical technician
  interface and in-memory ticket broker compile in the shared endpoint source;
  native use remains unverified.
- Native lockfile resolution at `75632a2` adds managed-agent dependencies while
  preserving every pre-existing package version, checksum and Git commit. A
  [pinned Windows x64 build at `c78a471`](https://github.com/rijff24/SwanRemoteSupport/actions/runs/36873808239)
  passed Rust/Flutter compilation, unsigned customer package generation, and
  product-component compilation. It predates the capability-policy changes.
  The [build at `31b13aa`](https://github.com/rijff24/SwanRemoteSupport/actions/runs/36967791061)
  also passed and produces a distinct technician portable artifact. It predates
  subsequent native file/audio capability guards, consent ordering, complete
  backup and installation/repair changes. A new build at `7e589ba` is queued.
- The unsigned Windows artifact from `31b13aa` downloaded successfully. Its
  digest matches GitHub's artifact digest and both endpoint package checksums
  match `SHA256SUMS.txt`. Both endpoint packages and all three product-component
  executables report `NotSigned`; no downloaded executable was launched on the
  production host. This proves artifact integrity, not trusted publication.
- A fresh-company HTTPS test at `71ebbec` passed with rebuilt agent and server
  executables. It additionally verifies enrollment-preserving repair, rejection
  of a changed transport identity, explicit consent opt-in/revocation,
  administrator-only group policy changes, capability fields in signed grants,
  and denied lease renewal after policy reduction. This is API/agent evidence,
  not proof of native receiver enforcement or installer repair.
- Existing RustDesk, hbbs and hbbr services remained running. Only the isolated
  development management process was rebuilt and restarted.
- Windows hypervisor API probing reports an available hypervisor. QEMU 11.1.0
  passed its published SHA-512 checksum and runs from the separate local lab.
  Official Server 2025 evaluation media downloaded successfully; its size and
  SHA-256 are recorded outside Git. Guest boot attempts and failures are
  documented in `deployment/windows/VM-TESTING.md`. No evaluation guest
  installation has passed yet.

## Requirements still to demonstrate

A fresh-company HTTPS lifecycle run at `d6f1665` passed with clean source and
newly built executables. It additionally verifies saved-target network
diagnostics, management certificate/company-signature validation, and refusal
to claim external or UDP reachability. Enrollment, permissions, branding and
offline-consent recovery checks also passed in that run; native transport and
clean-machine installation remain unverified.

Initial setup now verifies installed endpoint and agent hashes/publishers against
signed release metadata and records the release sequence before enabling the
customer watcher. Technician setup copies its portable executable, agent and
launcher to a durable per-user directory and creates a Start menu shortcut;
launch no longer installs opportunistically from the extracted download. Rust
compilation/component tests and PowerShell parsing pass. Actual signed setup,
signed same-release repair, launcher use and uninstall remain unverified or incomplete.
The explicit repair trust gate passes component tests and requires exact pinned
metadata without allowing automatic-update replay; see INSTALLER_REPAIR.md.

The built product executables at `20f88e3` additionally passed a fresh-company
HTTPS lifecycle test with an offline-consent fault injected through the local
proxy. Revocation remained durable locally during HTTP 503 responses; the
server retained its older choice until the real agent watcher synchronized it,
then denied unattended grant requests. Executable hashes, exact harness source
dirty-state and the recovery result are recorded privately. Native sessions
remain unverified.

| Requirement | Current evidence and remaining work |
| --- | --- |
| Reproducible server, customer, technician and worker builds | Product components pass pinned builds. Native Windows build/package run 36970913876 passed at exact source `7e589ba`; the later manifest/installer-locking build 36973173582 remains in progress. Baseline rebuild and clean-machine installation are not newly proved. |
| Fresh-company Windows/Linux setup | HTTPS/API setup works. Administrator network settings and server-vantage DNS/TCP/HTTPS company-signature diagnostics are implemented with access tests. Equivalent full setup wizards, transport provisioning, certificate management and external reachability checks remain incomplete; see NETWORK_DIAGNOSTICS.md. |
| Roles, groups, permissions and revocation | MFA/group/device denial tests pass. Signed capability fields, administrator group policy APIs, and receiver capability bounds are implemented in source. Policy reduction denies lease renewal in component tests. Administration controls, native enforcement and direct-connection bypass tests remain unverified. |
| Configurable customer and technician apps | Signed profile sync passes. Graphical login/inventory/history, logo/color rendering, contacts, company shortcuts, offline operation and native restart/upgrade preservation remain to verify or complete. |
| Signed profiles and rotation | Component tests reject tampering, wrong-company, expired and older profiles and untrusted key changes. Trusted cached key/endpoint rotation preserves enrollment in tests. Real HTTPS rotation and installed-app behavior still need verification. |
| Approved managed sessions without password/direct bypass | Grant signature, target, challenge and replay checks pass component/API tests. Actual receiving-device enforcement, direct/relay paths, lease expiry and live revocation are unverified. |
| Customer approval and unattended consent | API consent denial and durable local revocation tests pass. Real attended prompts, stop/uninstall, unattended opt-in, restart, sign-in screen and UAC are unverified. |
| Transfer, clipboard, multiple monitors | Native end-to-end tests pending. |
| Public company transport without Tailscale | Source uses company transport and fails closed before setup. External NAT traversal, relay fallback and negative-access tests pending; no public endpoint has been changed. |
| Windows worker and public/protected downloads | Queue, worker, fixed bundles and access checks are implemented. A real signed worker job, both EXE/MSI installation products, protected technician download and public customer website installation remain unverified or incomplete. |
| Signing and exact corresponding source | Certificate/hash/publisher pins and source metadata are implemented. Provider approval, production signatures, company signing integration and production release checks remain gates. |
| Approved automatic updates | Metadata replay/expiry and session exclusion tests pass. Signed installed manifests now bound customer DLLs/assets and reject unlisted native code in component tests; real signed installation remains unverified. Real rollout, pause/windows, agent replacement, technician executable handoff, interruption repair/retry/rollback and recovery remain incomplete or unverified; see RELEASE_PAYLOAD.md. |
| Backup, restore and migrations | Encrypted round-trip/tamper checks cover management data, transport trust keys and SQLite state, deployment configuration and an explicit TLS identity. Actual CLI export/restore and a live isolated HTTPS management-server rehearsal preserve signing trust, MFA, accounts, device state and group policy; revoked access remains denied. Restored SQLite quick_check passes. Live transport recovery, complete certificate-service storage, post-backup reconciliation and migration rollback remain unverified or incomplete; see SERVER_BACKUP.md. |

## Clean Windows matrix

All rows are intended targets, not verified compatibility claims. Record exact
edition/build, media hash, product commit, package signature/hash and results.

| Operating system | Clean installation | Repair/upgrade/recovery | Native session behavior |
| --- | --- | --- | --- |
| Windows 10 x64 | Unverified | Unverified | Unverified |
| Windows 11 x64 | Unverified | Unverified | Unverified |
| Server 2016 Desktop Experience | Unverified | Unverified | Unverified |
| Server 2019 Desktop Experience | Unverified | Unverified | Unverified |
| Server 2022 Desktop Experience | Unverified | Unverified | Unverified |
| Server 2025 Desktop Experience | Unverified | Unverified | Unverified |

See [VM testing](../deployment/windows/VM-TESTING.md) and
[local HTTPS testing](../deployment/windows/LOCAL-TESTING.md). Test credentials,
signing keys, certificates with private keys, VM images and raw operational logs
must remain outside public source. The project provides no shared company VPS.

First-run branding now reads the protocol defaults from the management service,
including the bundled swan/gear SVG and teal primary color. The server keeps its
Swan wordmark and mark; existing company profiles are not rewritten. Product
workspace tests (23) and administration JavaScript syntax checks pass. Graphical
rendering and installed-app behavior remain to verify.

Update recovery now retains session exclusion after process exit through the
pending-update receipt. A new regression test verifies rejection after the
exclusive process lock has closed; all 24 workspace tests pass. Installed-agent
hash/publisher and Windows compatibility are included in recovery validation.
Full signed agent replacement, retry/rollback and VM recovery remain unverified
or incomplete.

Staged signed-agent handoff and agent replacement are now implemented in source,
including receipt-based retry and validation of the customer SYSTEM task before
restart. Windows Rust compilation and existing 24 product tests pass; signed
self-replacement, interrupted installer retry, scheduled-task restart and
technician handoff require actual VM tests. Rollback remains incomplete.

Recovery trust regression checks and shared validation pass with 25 Windows
product workspace tests. Durable retry delay and failed-helper task restart are
implemented in source and PowerShell parsing passes. These checks do not prove
signed installation, power-loss recovery or rollback on clean Windows guests.

Technician MSI authoring and a pinned WiX extraction/build recipe now produce
an actual unsigned package locally. MSI tables and build-input hashes were
inspected; no installer ran on the host. Technician setup/update support MSI
containers and explicit external bootstrap input. Clean-guest install/upgrade,
signing, customer MSI and complete uninstall cleanup remain acceptance gates;
see deployment/windows/TECHNICIAN-MSI.md.

Customer MSI authoring and the CLI recipe now build an actual unsigned package
from the verified native 31b13aa payload, without executing product installers.
The package includes the complete Flutter payload, local license and agent;
service and Windows-owned broker copy/removal tables were inspected. Three new
authoring tests and the existing three manifest tests pass. Clean-machine
installation, signing, service/task lifecycle and recovery remain unverified;
see deployment/windows/CUSTOMER-MSI.md. Native CI includes both MSI recipes.

Explicit MSI repair now requests forced replacement and source re-caching
through `/fvamus`, while retaining the exact pinned-release trust gate. Native
x64 compatibility preflight and technician MSI launch conditions now reject
ARM64; the preflight identifies the current host as windows_11. Clean-machine
repair and ARM64 rejection still require runtime validation on those guests.

Customer MSI registration now records its current product code and installation
location. The customer app validates that registration against the installed
Windows Installer product before requesting removal, resolves the Windows-owned
msiexec executable directly, and refuses malformed registrations. Three MSI
authoring tests and Rust parsing pass. Native compilation and clean-guest
uninstall, service removal and complete scheduled-task/state cleanup remain
unverified or incomplete.

Native CI run 36982373150 compiled the apps but failed MSI packaging because
the WiX extractor emitted COM return values alongside its executable path.
Those values are now suppressed. A real pinned-package extraction verifies
exactly one string result and WiX version 5.0.2; the CI rerun remains pending.

All 25 existing product workspace tests passed on Windows after the MSI
registration change. Shared key parsing additionally rejects weak Ed25519
public keys; six protocol tests pass, including rejection of identity and
zero encodings and acceptance of a generated key. Native packaging-fix CI
run 36992817681 was confirmed queued; completion is not yet established.

MSI update recovery now reads the verified staged package's product identity
and queries Windows Installer registration before choosing installation or
forced repair. An installed product uses `/fvamus`; an unknown or advertised
product uses `/i`. Other-user and corrupt registrations fail closed. The real
uninstalled lab MSI read-only probe returns `/i`, and all 26 product workspace
tests pass. Installed-product repair, interruption recovery and rollback still
require clean-guest tests. Product state meanings follow
[Microsoft's Installer API](https://learn.microsoft.com/en-us/windows/win32/msi/installer-productstate-property).

The installer worker now isolates claim parsing, build, upload and completion
errors from its polling loop. Transient job failures are logged without exiting
the worker; existing server claim expiry retains recovery. Windows cargo check
passes. Immediate upload retry, idempotent completion and a real signed worker
job with network fault injection remain incomplete or unverified. Native runs
36992580109 and 36992817681 were confirmed in progress and queued respectively.
