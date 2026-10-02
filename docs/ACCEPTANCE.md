# Configurable product acceptance report

Development status, 1 October 2026. **The product is not complete or approved for
production publication.** A passing component test is not evidence of a working
native support session or a clean-machine installation.

## Evidence obtained

- Public baseline tag `swan-single-company-baseline-1.4.9` preserves the original
  Swan deployment and build instructions. Its source was not replaced.
- Seventeen product tests pass on the local Windows development host, including
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
  A new build at `31b13aa` also produces a distinct technician portable artifact;
  its outcome remains pending.
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

Initial setup now verifies installed endpoint and agent hashes/publishers against
signed release metadata and records the release sequence before enabling the
customer watcher. Technician setup copies its portable executable, agent and
launcher to a durable per-user directory and creates a Start menu shortcut;
launch no longer installs opportunistically from the extracted download. Rust
compilation/component tests and PowerShell parsing pass. Actual signed setup,
same-release repair, launcher use and uninstall remain unverified or incomplete.

The built product executables at `20f88e3` additionally passed a fresh-company
HTTPS lifecycle test with an offline-consent fault injected through the local
proxy. Revocation remained durable locally during HTTP 503 responses; the
server retained its older choice until the real agent watcher synchronized it,
then denied unattended grant requests. Executable hashes, exact harness source
dirty-state and the recovery result are recorded privately. Native sessions
remain unverified.

| Requirement | Current evidence and remaining work |
| --- | --- |
| Reproducible server, customer, technician and worker builds | Product components pass pinned builds. Native x64 build/package remains pending; baseline rebuild is not newly proved. |
| Fresh-company Windows/Linux setup | HTTPS/API setup works. Equivalent full setup wizards, transport provisioning, certificate management and reachability checks remain incomplete. |
| Roles, groups, permissions and revocation | MFA/group/device denial tests pass. Signed capability fields, administrator group policy APIs, and receiver capability bounds are implemented in source. Policy reduction denies lease renewal in component tests. Administration controls, native enforcement and direct-connection bypass tests remain unverified. |
| Configurable customer and technician apps | Signed profile sync passes. Graphical login/inventory/history, logo/color rendering, contacts, company shortcuts, offline operation and native restart/upgrade preservation remain to verify or complete. |
| Signed profiles and rotation | Component tests reject tampering, wrong-company, expired and older profiles and untrusted key changes. Trusted cached key/endpoint rotation preserves enrollment in tests. Real HTTPS rotation and installed-app behavior still need verification. |
| Approved managed sessions without password/direct bypass | Grant signature, target, challenge and replay checks pass component/API tests. Actual receiving-device enforcement, direct/relay paths, lease expiry and live revocation are unverified. |
| Customer approval and unattended consent | API consent denial and durable local revocation tests pass. Real attended prompts, stop/uninstall, unattended opt-in, restart, sign-in screen and UAC are unverified. |
| Transfer, clipboard, multiple monitors | Native end-to-end tests pending. |
| Public company transport without Tailscale | Source uses company transport and fails closed before setup. External NAT traversal, relay fallback and negative-access tests pending; no public endpoint has been changed. |
| Windows worker and public/protected downloads | Queue, worker, fixed bundles and access checks are implemented. A real signed worker job, both EXE/MSI installation products, protected technician download and public customer website installation remain unverified or incomplete. |
| Signing and exact corresponding source | Certificate/hash/publisher pins and source metadata are implemented. Provider approval, production signatures, company signing integration and production release checks remain gates. |
| Approved automatic updates | Metadata replay/expiry and session exclusion tests pass. Real rollout, pause/windows, full payload/agent identity preservation, technician executable handoff, interruption repair/retry/rollback and recovery remain incomplete or unverified. |
| Backup, restore and migrations | Encrypted round-trip/tamper checks cover management data, transport trust keys and SQLite state, deployment configuration and an explicit TLS identity. Protected Windows restore passes component tests. Live server restore, complete certificate-service storage and migration rollback remain unverified or incomplete; see SERVER_BACKUP.md. |

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
