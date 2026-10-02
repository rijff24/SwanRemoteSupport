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
