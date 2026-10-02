# Configurable product acceptance report

Development status, 2 October 2026. **The product is not complete or approved for
production publication.** A passing component test is not evidence of a working
native support session or a clean-machine installation.

## Evidence obtained

- Public baseline tag `swan-single-company-baseline-1.4.9` preserves the original
  Swan deployment and build instructions. Its source was not replaced.
- Thirty-three product tests pass on the local Windows development host at `6438ce2` (18 agent, eight management, seven protocol), including
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
  backup and installation/repair changes. These builds are historical evidence;
  their results do not establish acceptance of later changes.
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
| Reproducible server, customer, technician and worker builds | Product components pass pinned builds. Native Windows build/package run 36998413349 passed at exact source `2eec008`. Later native source, including consent/reconnect/version and rollback changes, still needs CI and VM verification. Baseline rebuild and clean-machine installation are not newly proved. |
| Fresh-company Windows/Linux setup | HTTPS/API setup works. Administrator network settings and server-vantage DNS/TCP/HTTPS company-signature diagnostics are implemented with access tests. Equivalent full setup wizards, transport provisioning, certificate management and external reachability checks remain incomplete; see NETWORK_DIAGNOSTICS.md. |
| Roles, groups, permissions and revocation | MFA/group/device denial tests pass. Signed capability fields, administrator group policy APIs, and receiver capability bounds are implemented in source. Policy reduction denies lease renewal in component tests. Administration controls, native enforcement and direct-connection bypass tests remain unverified. |
| Configurable customer and technician apps | Signed profile sync passes. Graphical login/inventory/history, logo/color rendering, contacts, company shortcuts, offline operation and native restart/upgrade preservation remain to verify or complete. |
| Signed profiles and rotation | Component tests reject tampering, wrong-company, expired and older profiles and untrusted key changes. Trusted cached key/endpoint rotation preserves enrollment in tests. Real isolated HTTPS trusted key rotation, missed-transition rejection and restored rotated trust pass for both CLI agents. Graphical installed-app behavior remains unverified. |
| Approved managed sessions without password/direct bypass | Grant signature, target, challenge and replay checks pass component/API tests. Actual receiving-device enforcement, direct/relay paths, lease expiry and live revocation are unverified. |
| Customer approval and unattended consent | API consent denial and durable local revocation tests pass. Real attended prompts, stop/uninstall, unattended opt-in, restart, sign-in screen and UAC are unverified. |
| Transfer, clipboard, multiple monitors | Native end-to-end tests pending. |
| Public company transport without Tailscale | Source uses company transport and fails closed before setup. External NAT traversal, relay fallback and negative-access tests pending; no public endpoint has been changed. |
| Windows worker and public/protected downloads | Queue, worker, fixed bundles and access checks are implemented. A real signed worker job, both EXE/MSI installation products, protected technician download and public customer website installation remain unverified or incomplete. |
| Signing and exact corresponding source | Certificate/hash/publisher pins and source metadata are implemented. Provider approval, production signatures, company signing integration and production release checks remain gates. |
| Approved automatic updates | Metadata replay/expiry, session exclusion, complete snapshots and signed failed-release quarantine tests pass. Real HTTPS download tamper/interruption rejection passes. Portable technician restoration and resumable rollback are implemented, but signed native restoration is unverified. Customer/MSI rollback, service/installer consistency, full rollout, interruption and clean-machine recovery remain incomplete or unverified; see RELEASE_PAYLOAD.md. |
| Backup, restore and migrations | Encrypted round-trip/tamper checks cover management, transport trust keys, SQLite, private deployment configuration and complete TLS storage trees. Real CLI export/restore and isolated HTTPS rehearsals preserve signing trust, MFA, accounts, devices, groups, consent and rotated keys; revoked grants remain denied. Restored SQLite quick_check passes. Schema upgrade requires encrypted pre-upgrade backup in component tests. Live transport/ACME renewal, post-backup reconciliation and full migration rollback remain unverified; see SERVER_BACKUP.md. |

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

Worker completion reporting now permits exact repeated terminal results from
the original active worker and refuses conflicting results or revoked workers.
A real management-handler regression test passes for initial completion, lost
response retry, conflict and revocation, with saved-result integrity checked.
The worker retries completion reporting up to three times for transport errors,
server failures or rate limits; Windows compilation passes. Signed upload retry
and full network-fault recovery still require implementation and verification.

Artifact upload persistence now accepts an identical retry from the owning
worker while the job is uploaded, and refuses changed bytes, another worker
or uploads after completion. The storage regression test checks preserved
bytes and state; all 28 product workspace tests pass. The worker uses bounded
transient retries with a cloned buffered request body. This is storage-level
evidence, not a signed end-to-end worker fault test. QEMU reports running but
a fresh Server 2025 screenshot still shows boot; clean installation is unproven.

The administrator branding/policy form now exposes stable/test channel and UTC
maintenance start/end controls, saving through the existing authenticated signed
profile API. Equal-hour and overnight descriptions match protocol semantics.
JavaScript syntax and diff checks pass. Rendered form interaction and installed
client rollout behavior remain unverified.

Administrator account controls now expose technician disable and group-access
revocation. Group removal closes affected grants transactionally; re-granting
access cannot revive them. Account disable removes login sessions and closes
all account grants in the same audited transaction. Expanded real API lifecycle
checks pass for administrator-only removal, inventory/grant/renewal denial,
re-grant non-revival and disabled-token rejection. JavaScript syntax passes.
Native active-session termination on lease expiry and rendered controls remain
unverified; no live company account was changed by these tests.

Customer and technician branding forms now edit their logo and consent text
independently. Saving no longer overwrites technician values with customer
values. Both use the existing signed, validated edition profile fields;
JavaScript syntax passes. Rendered interaction and installed edition branding
synchronization remain unverified. Native runs 36992580109 and 36992817681 are
still confirmed in progress; no completion result is inferred.

At clean source 5be190d, the Windows GNU release workspace build succeeds.
A release-profile local-CA rehearsal stops at the expected TLS trust check:
release binaries ignore the debug-only test CA, and that protection was retained.
A separately built development rehearsal profile then passes the actual isolated
HTTPS lifecycle with fresh data and built agent/server executables, including
group removal, inventory and new-grant denial, non-revival after re-grant, prior
MFA/enrollment/branding checks and offline consent recovery. Exact binary hashes
and private result records remain in ignored test data. This does not verify
production certificate deployment, native sessions, signed installers or VMs.

Device revocation/pending transitions and device group moves now close existing
grants in the same audited transaction, preventing re-approval or moving back
from reviving them. Unattended consent revocation permanently closes unattended
grants while preserving attended grants. All 28 workspace tests pass, and the
expanded API lifecycle additionally verifies re-consent non-revival, attended
authorization preservation, device re-approval denial and group move/back denial
including administrator grants. Native session shutdown on lease expiry remains
unverified. Current native CI was confirmed building the branded client, with
no terminal result yet.

Group capability reductions now close incompatible grants transactionally.
Restoring capabilities does not revive withdrawn authorization. Grant creation
checks device/user/group access, snapshots permissions and inserts its grant
and audit event in one transaction, avoiding a stale permission insertion
after reduction. The expanded API lifecycle test passes, including restored
policy non-revival and separate attended-consent preservation. Concurrent fault
injection and native session termination remain unverified.

Disabling company-wide unattended support now closes all unattended grants in
the profile-update transaction. Restoring that setting cannot revive them, and
attended authorization is preserved. Grant creation reads company policy within
the same transaction as access checks and permission snapshots. Expanded API
lifecycle tests pass for global disable/restore non-revival, attended renewal
and a fresh grant followed by local consent revocation. Concurrent fault tests
and native termination behavior remain unverified.

Signed edition branding now supports public support-contact text and up to eight
named HTTPS website shortcuts. Administration edits each edition independently;
both native endpoint pages use a shared contact/link component. Shortcuts cannot
execute shell commands, local files or non-HTTPS schemes. Empty fields preserve
existing serialized branding, and old profiles deserialize with defaults. All
29 workspace tests pass, including credential-bearing/unsafe URLs, invalid
labels, excessive links and control characters. JavaScript syntax and native
Rust parsing pass. Flutter compilation, rendering and installed synchronization
remain unverified. Upgrade clients to compatible builds before publishing new
nonempty branding fields; older strict clients will reject unfamiliar fields.

Optional local company artifact signing now has an explicit pinned-input helper
using a company certificate's local private-key provider and Microsoft-signed
SDK SignTool. It signs separate copies, requires trusted publisher/certificate
verification and a timestamp, and records final hashes. PowerShell parsing and
wrong-input-hash rejection pass against a real unsigned agent with no source
change or output. No actual signature was made. Provider/certificate signing,
worker consumption, signed installs/updates and release-issuer migration remain
unverified or incomplete; see deployment/windows/COMPANY-SIGNING.md.

Technician approved-release discovery now uses its authenticated Rust session
and shares signature, edition, channel, expiry and sequence validation with the
installer. The GUI receives only public release identity; bearer credentials
remain in Rust. Installed sequences are not offered again, and pause/maintenance
policy applies. All 29 workspace tests and native Rust parsing pass. Flutter
compilation and end-to-end authenticated discovery remain unverified. This is
discovery only: automatic GUI shutdown, session-safe installation handoff and
full interrupted-update recovery/rollback remain incomplete.

Outgoing managed connection loops now own their activity locks from before
transport establishment until loop exit. Closing one window does not permit an
update while another connection remains active, and the last disconnected
connection no longer keeps a process-wide lock forever. Proof generation also
checks the installation guard before consuming its ticket. All 30 workspace
tests pass, including simultaneous independent locks and Windows installer byte
lock exclusion. Native Rust parsing passes. Native connection lifecycle and
GUI shutdown/restart during signed installation still require end-to-end tests.

Portable technician and configuration-agent replacements now write and flush a
complete, hash-verified staging file on the destination volume before replacing
the installed file. Replacement failures retain the previous destination and
remove staging files; Windows sharing conflicts use a bounded retry. All 31
workspace tests pass, including hash rejection, successful replacement, unchanged
source and failed-replacement cleanup on Windows. This reduces partial-write
failure exposure; power-loss testing, signed executable replacement, MSI rollback
and complete interrupted-upgrade recovery remain unverified or incomplete.

Technician refresh now requests automatic update handoff through its Rust login.
Open connection windows defer the request; native locks and signed recovery
receipts coordinate other processes. The GUI closes only after a verified helper
has been launched, and the helper restarts the verified technician executable
without credentials after success. Existing pending handoffs use bounded retry.
All 31 workspace tests and native Rust parsing pass. GUI compilation, signed
end-to-end shutdown/install/restart, offline pending recovery and full rollback
remain unverified or incomplete; this flow is not yet release acceptance.

New update installation now refreshes signed company configuration and queries
current release approval after downloads. Withdrawn or changed selections defer
handoff; the newest saved pause/maintenance policy is checked again under session
exclusion before writing the receipt. All 32 workspace tests pass, including a
persisted signed-policy change that pauses or moves the window after an older
snapshot was eligible. Real HTTP withdrawal during downloads and signed Windows
installation still require end-to-end evidence. Existing installation receipts
retain recovery semantics rather than starting a new policy-controlled upgrade.

Technician refresh now attempts pending signed-installation recovery before
network sync, login or inventory. The local native action cannot choose a new
release; it retains receipt signature, sequence, staged helper, publisher and
session-exclusion checks, and runs blocking verification outside the async
executor. Cached branding is displayed before these operations. Native Rust
parsing passes; the existing 32 component tests remain the last workspace
evidence. This new GUI path still needs native compilation and a real offline
interrupted-installation test. The QEMU process is confirmed live; no completed
clean Windows installation is demonstrated and VirtualBox remains uninstalled.

Automatic updater Windows compatibility checks, publisher verification, pending
recovery and final receipt/helper handoff now run in blocking workers. Final
session exclusion is acquired and released wholly within one worker, without
holding its file lock across an async wait. All 32 workspace tests pass. Native
GUI executor responsiveness and signed update handoff remain unverified.

Administrators can now withdraw individual release approval from the release
page or DELETE its approval endpoint. Approval withdrawal and its audit event
commit atomically; technicians are denied. Eight management tests pass, including
withdrawal stopping update selection, edition/channel separation, pause and
zero-percent rollout. Web JavaScript syntax passes. Browser interaction and real
HTTP withdrawal during client downloads remain unverified. Previously generated
bundles and already-started installation recovery are not removed by withdrawal.

Release withdrawal now also fails matching queued/running/uploaded build jobs
within its transaction and refuses new downloads of completed bundles while
approval is absent. Existing files are retained; already downloaded copies and
started installation recovery are unaffected. Build creation keeps its approval
check and queue insertion under the same database lock. All eight management
tests pass, including three job states and completed-bundle download denial.
Worker/network race fault injection and browser behavior remain unverified.

The withdrawal regression now exercises a real authenticated worker claim and
stored fixture artifact: late persistence, failure completion and success
completion are rejected after withdrawal, stored bytes remain unchanged, and
restoring approval does not revive or requeue cancelled claims. Eight management
tests pass; the final expanded withdrawal test also passes independently. This
uses handler/storage fixtures, not a signed installer worker end-to-end run;
simultaneous network faults and production package publication remain unverified.

Technician inventory/history now display before optional update discovery and
handoff. Update errors have a separate visible status and no longer discard
fresh inventory. The native recovery response exposes a public pending flag;
pending/failed recovery is visible and disables new connection controls while
native receipt and file-lock enforcement remains authoritative. Native Rust
parsing and diff checks pass. Flutter compilation, rendering and responsiveness
during long downloads remain unverified. The branding native run is confirmed
live in dependency installation; later native runs remain queued.

Technician update preparation now runs as one native background task instead
of holding the refresh request open through downloads/signature verification.
The UI polls public running/failed/handoff flags independently, keeps login
credentials in Rust, and closes only after verified handoff. Recovery avoids
launching another helper while that task is active. Final native session locks,
fresh server approval and pending receipts remain installation gates. Rust
parsing and diff checks pass; source inspection confirms the task uses the
existing persistent Tokio runner. Native compilation, task responsiveness and
GUI/session races still require end-to-end validation.

Consumed native technician ticket handles now retain bounded, login-generation
bound reconnection context without grants or proof keys. A reconnect requests a
new server grant and ephemeral proof key, verifies the original device/peer and
consent mode, and rejects logout or login changes. Attended sessions still need
customer approval; reconnection never upgrades them to unattended access.
Logout/login/update handoff clear the context. Native Rust parsing and diff
checks pass. Reboot reconnection, concurrent-window behavior, revocation during
reconnect and full native compilation remain unverified.

Managed peer-information handling now clears inherited peer passwords and skips
upstream password/address-book synchronization. Legacy UI password submission
is refused before constructing a network login; managed challenge proofs still
use their separate native path. These guards apply only to the managed build,
preserving baseline source behavior. Rust parsing and diff checks pass. Native
compilation, saved-settings inspection and negative legacy-UI tests remain
unverified; receiver-side signed authorization remains the access boundary.

Customer consent changes now use acknowledged asynchronous service IPC through
the existing company request bridge. Enable acknowledges only after server
acceptance and local persistence. Revoke persists locally first, then reports
whether server sync succeeded; the UI distinguishes pending sync and refuses to
claim success after an error/timeout. Consent buttons prevent overlapping UI
requests. Existing consent revision checks still reject a delayed enable after
revocation. Rust parsing and diff checks pass. Full Windows service IPC, offline
revocation feedback, persistence failures and UI rendering remain unverified.

An independent official Windows Server 2016 evaluation VM is now provisioned
and running with a separate 64 GB disk, 2 GB memory, one CPU and user NAT without
forwarded ports. Its fresh screenshot confirms the Windows Setup language page.
The evaluation download completed with recorded origin/hash; private VM metadata
and screenshots remain outside the repository. The existing Server 2025 VM is
still running at its boot screen and was not restarted. No clean Windows install
or product installation has passed yet. Native build 37002003558 is queued at
0c71544, covering the acknowledged customer-consent source.

The Server 2016 VM has now selected Standard Evaluation with Desktop Experience
and started installation on its empty virtual disk. File preparation reached
81%; this is provisioning progress, not a completed OS or product test.

Management backup accepts `--tls-directory` for the complete HTTPS storage tree.
Nested certificate-key and ACME account fixtures survive authenticated encrypted
export/restore. Restore rejects traversal, Windows filename collisions and
file/directory conflicts before creating its target. Nine management tests pass;
the three backup tests also pass after adding malformed encrypted archive cases.
Live certificate renewal, transport restore and migration rollback remain open.

Server 2016 has completed file preparation, features and its first reboot. It
is now at the initial administrator-password screen; desktop provisioning and
product tests are pending. The computer-use authentication boundary requires
user completion of that screen.

Customer MSI now schedules a SYSTEM configuration-task cleanup action before
file removal, with paired rollback and exclusion during major upgrades. It
checks exact task ownership, preserves company state, and records the previous
enabled/running state for rollback. Windows agent compilation, script parsing,
three MSI authoring tests and real unsigned WiX packaging pass. Installer-table
inspection verifies action order. This does not establish actual uninstall,
rollback, task cleanup or service behavior; clean-guest tests remain open.

Configurable endpoint source now identifies itself as 1.5.0 (Flutter build 68),
separately from the tagged 1.4.9 Swan baseline. Cargo, portable packer, lockfile,
Flutter and Windows artifact versions agree in a local source check. Native CI
checks app/packer/Flutter/artifact version agreement before building. This is a
development version change; no release tag or signed production publication has
been made, and native build/upgrade behavior must still be demonstrated.

Management now provides offline prepare/activate profile-key commands. They
publish a replacement under the existing key before activation, require
operator confirmation of client synchronization, and atomically update active
key/profile state in SQLite. Activation closes grants and invalidates installer
jobs with old bootstrap trust. A process lock prevents simultaneous upgraded
management instances. Schema-1 migration requires an encrypted pre-upgrade
backup; restored active keys and revisions survive component tests. Eleven
management tests pass, including rotation, migration and backup recovery.
Installed-app HTTPS rotation and live migration rollback remain unverified.

An isolated real HTTPS rehearsal with management/agent binaries built at
7843334 now passes trusted profile-key rotation for both agent editions.
Enrollment, device token and revoked consent persist; a client missing the
transition rejects the new key; activation without explicit confirmation fails;
restart retains the active key. Renewal of a claimed, still-unexpired grant is
denied after activation. Private results record binary hashes and note the
uncommitted harness in the worktree. Owned fixture listeners stopped afterward.
Graphical installed apps, actual session termination, production certificates
and full migration rollback remain separate unverified acceptance gates.

The HTTPS rotation harness now exercises actual management CLI encrypted export
and restore of rotated company data, private configuration and real HTTPS
PFX/certificate storage. A replacement HTTPS proxy uses those restored files;
the active profile key and device credential remain valid and the closed grant
stays denied. The rehearsal passes again after rebuilding startup and rotation
operations onto blocking worker threads. Private results record the binary
hashes and dirty source state; both fixture listeners stop. Live ACME renewal,
hbbs/hbbr restoration and Windows native acceptance remain unverified.

The administration branding form now uses separate shortcut name/HTTPS-address
fields with add/remove controls and an eight-entry limit. Valid names and URLs
containing pipe characters survive editing without delimiter ambiguity. A local
JavaScript regression check verifies exact pipe/whitespace preservation, blank
row omission, incomplete-entry rejection and HTML attribute escaping. JavaScript
syntax and diff checks pass. Browser interaction, accessibility and installed-app
shortcut rendering remain unverified.

The Windows installer worker now stages downloaded files through asynchronous
filesystem operations and runs Authenticode verification, ZIP compression,
artifact hashing and immutable publication in a blocking worker task. Windows
compilation passes. Real signed worker packaging, upload faults and publication
remain unverified. After company profile-key activation, operators must update
the worker's privately configured profile-key pin before rebuilding packages;
automatic worker trust-pin rotation is not implemented.

At source `7335010`, all 37 product workspace component tests pass on the local
Windows GNU toolchain. Update downloads now flush unique staging files before
atomic publication rather than overwriting a complete staged artifact in place.

The subsequent recovery change re-downloads missing or hash-damaged artifacts
from the exact signed pending receipt, rechecks the publisher and unchanged
receipt, and retains session exclusion at handoff. Technician recovery runs in
the background with the existing native progress guard. The agent Windows build
and its 20 ordinary component tests pass. A separate explicitly ignored real
HTTPS download test passes through `test-update-download.js`: an HTTPS redirect
carries no authorization or cookie headers, and tampered or interrupted responses
leave complete staging intact. Private evidence records the executable hash and
dirty source state. This transport regression does not execute a native installer,
prove successful signed recovery, or complete automatic rollback. Native Rust
syntax parses locally; the changed Rust/Flutter integration requires CI and VM
verification.

GitHub native build run 36998413349 at `2eec008` is confirmed successful.
Run 37005806648 at `4164964` remains queued as of the latest direct API check;
neither run proves the subsequent online recovery implementation.

The superseded queued run 37005806648 was subsequently cancelled after checking
its current queued status. Replacement native build 37010780710 targets
`111abe3`, covering online recovery and background technician handoff; it was
confirmed queued when dispatched.

New update preparation now requires a recorded signed installed release and
preserves its full endpoint manifest and configuration agent in a verified,
immutable rollback snapshot before writing the update receipt. The receipt
includes the exact previous signed envelope. The helper rejects a mismatched or
damaged snapshot before changing installed files. Branding, credentials, identity
and consent remain outside the software snapshot. Windows agent compilation and
21 ordinary agent tests pass (the separate real-HTTPS transport test is explicitly
ignored in this ordinary run). Further targeted tests pass for complete snapshots,
tampered and missing sources, unsafe paths, cleanup, immutable publication, and
previous-release signature/edition/sequence checks after metadata expiry. This is
verified rollback preparation, not completed automatic rollback: restoration,
failed-release suppression, MSI/service consistency and native power-loss testing
remain unfinished.

Portable technician EXE rollback is now implemented for a previous release whose
signed metadata declares `rollback_protocol: 1` and whose verified agent confirms
that support. It preserves the old installed sequence, writes a durable pending
rollback phase before restoring signed snapshot files, verifies the restored
identity, and publishes a signed-envelope failed-release quarantine before
removing the pending marker. New discovery/installation suppresses that sequence
and older ones; later approved releases remain eligible. Current branding,
enrollment and consent are not restored from software snapshots. Customer/MSI
restoration and service/registration consistency remain unfinished.

All 39 ordinary product workspace tests pass for this change. The real-HTTPS
download test is explicitly ignored in that ordinary run. Targeted quarantine
and receipt tests additionally pass after initial receipt publication became
atomic and unable to overwrite an existing receipt. The Windows agent build
passes, and its read-only capability command returns protocol 1 without creating
company state. Native Rust syntax parses locally; native Rust/Flutter compilation
and actual signed rollback, reboot/interruption and installer tests remain gates.

Update discovery now filters approved releases by the installed EXE/MSI format.
Clients derive the query from the verified signed installed-release envelope and
independently reject a mismatched server response. Server component regressions
select the older matching MSI despite a newer EXE, select the EXE when requested,
retain legacy unfiltered discovery, reject unsupported formats and unauthenticated
requests, and stop offering a withdrawn matching release. These checks use valid
release metadata. The targeted management test and Windows product workspace
compilation pass. Real signed mixed-format rollout and native upgrades remain
unverified.

Installer retention now preserves exact hash/publisher-verified installation
packages during setup, repair and completed update/recovery. New software
snapshots include the previous installer alongside the verified files, allowing
future MSI registration restoration to use its actual original package. Missing
old caches can re-fetch only the exact recorded artifact before update handoff.
Wrong hashes and unsigned installers are rejected before cache publication in a
local Windows negative test; actual signed Swan installer retention remains unverified.

The same retention regression subsequently passes a positive Authenticode check
using a Windows-signed executable copied to the private temporary fixture, with
the actual publisher and certificate fingerprint. Wrong publisher and certificate
pins retain the previous valid cache. That executable is never executed and no
certificate is created or installed. Actual signed Swan installation packages
and MSI registration recovery remain unverified.

Review also corrected the misplaced installing-only phase check that prevented
rollback handoff after restart. Completion-only recovery now owns that check;
pending rollback remains eligible for helper handoff. The shared phase/quarantine
regression passes, but native reboot/rollback still needs testing. Windows agent
and worker builds and workspace compilation pass. Their Windows PowerShell
commands now use the OS system directory and built-in module path; actual built-in
hash/signature command availability and unsigned-package rejection pass when
invoked from the PowerShell 7 development environment. Company installer script
syntax passes. No signed installer, MSI restoration or clean Windows acceptance
is implied by these component checks.

After these corrections, all 39 ordinary product workspace component tests pass
again on Windows. The separate HTTPS download test is explicitly ignored in that
run. This includes actual Authenticode positive retention and wrong-certificate
rejection for the copied OS fixture, not execution of an installation package.

MSI installation, repair, update commands and Windows worker packaging now
check the package database identity after signature validation. The edition's
fixed UpgradeCode, canonical ProductCode, exact signed ProductVersion and x64
summary template must match. The database is opened read-only; inspection does
not run installer actions. Explicit fixture tests passed against the actual
unsigned customer and technician 1.4.9 MSIs, rejecting the opposite edition and
wrong version and confirming unchanged artifact hashes. Synthetic checks also
reject malformed product identifiers and non-x64 packages. These identity-only
fixture tests do not bypass publication signature requirements. Agent and worker
Windows builds pass; signed installation and native MSI rollback remain gates.

Technician rollback now includes MSI registration recovery using the verified
previous installer. Recovery validates edition-specific related registrations
and their cached product identities before narrowly removing the failed product
and reinstalling or repairing the previous one. Unexpected registrations fail
closed. Durable rollback retries re-fetch only the exact signed failed MSI when
its staged copy is missing, because its ProductCode is needed for narrow removal.
Previous software identity must verify before failed-release quarantine clears
session exclusion; enrollment and consent remain current. All 21 ordinary agent
tests pass (two fixture tests excluded); the actual technician MSI read-only
planning fixture separately passes, including incorrect previous-version
rejection and no installer execution. Script syntax and Windows compilation
pass. Native MSI rollback, reboot/interruption, customer service rollback and
the complete clean Windows matrix remain unverified.

Customer MSI recovery now shares the verified registration path, with exact
customer service executable/account checks before removal, automatic/running
service verification after restoration, and owned configuration-task recreation
after signed installed-file verification. Task startup is deferred until helper
completion. Component tests reject another installation's service, a different
account, stopped/manual service and unexpected executable arguments without
querying or modifying any native service. Both actual MSI fixtures pass
read-only planning and reject incorrect previous versions. Rollback eligibility
regressions accept both MSI editions and reject cross-edition, protocol-0,
format-changing and customer-EXE rollback. Windows compilation, script syntax
and customer MSI authoring tests pass. No installer or service operation ran on
the host. Customer EXE rollback and autonomous reboot recovery between removal
and restoration remain unfinished; native MSI/service/task restoration and all
clean-machine acceptance remain unverified.

Durable update task handoff now registers the verified staged helper outside
MSI-owned installed files. Customer startup/SYSTEM and technician logon/limited
user tasks retry every five minutes, derive state from their staged location,
and revalidate signed pending metadata and helper identity. Own-task cleanup
refuses mismatched task path, actions, arguments or principal. Explicit MSI
uninstall cancels recovery and sessions before file removal; paired uninstall
rollback preserves any earlier cancellation. Signed MSI recovery excludes
those cancellation actions. This implements the reboot retry mechanism but
does not demonstrate native reboot recovery.

All 21 ordinary agent tests pass again, including cancellation session rejection;
agent and worker Windows builds pass. Task ownership and isolated cancellation
file tests pass, including malformed rollback rejection. Read-only task planning
and Scheduled Task object creation confirm intended principals and five-minute
repetition without registering any task. Both unsigned MSI recipes compile
against historical 1.4.9 native inputs; read-only tables confirm cancellation
rollback at 3498, cancellation at 3499 and file removal at 3500, excluded during
upgrades and signed recovery. No product installer, service or native task
mutation ran on this host. Signed task registration, startup/logon, reboot,
explicit cancelled-update reinstall handling and customer EXE rollback still
need implementation/verification as applicable; the full clean Windows matrix
and current native build remain gates.

Explicit setup now handles cancelled pending updates after incoming package
validation: it takes the activity lock, atomically prepares an immutable setup
marker and archives the exact old receipt without changing identity, consent or
sequence. Active recovery, tampered/foreign metadata, sequence replay,
conflicting archives and competing setup markers are rejected. Identical
preparation retries preserve evidence; exact committed setup metadata can finish
interrupted marker cleanup without allowing ordinary replay. The wrapper checks
the prepared marker instead of truncating it. A Windows component rehearsal
passes actual hash/publisher verification using a copied Windows-signed OS
executable, without executing it; bad bytes leave both pending evidence and
setup state unchanged. Targeted signed-recovery regressions, script parsing and
Windows agent/worker compilation pass. The MSI mode inspector reads an actual
unsigned uninstalled technician package and selects installation without running
it. Company bundles now include the inspector, and installer execution resolves
the OS system directory. Actual Swan-signed setup, MSI repair, cancelled-update
uninstall/reinstall and complete clean Windows acceptance remain unverified.

Customer EXE rollback now uses its retained verified original package, refuses
MSI-managed and unrelated registrations/services, verifies automatic service
startup and full installed identity, then applies the existing quarantine without
restoring enrollment or consent. Explicit EXE uninstall embeds a fixed encoded
cancellation/task-ownership script in its elevated uninstall batch; no profile
field provides commands. Stable native service/process names containing spaces
are quoted. SYSTEM install command execution avoids the interactive elevation
route. Silent portable packages now wait for their extracted installer and
propagate its exit status instead of racing installed-file verification.

Targeted signed recovery regressions and Windows agent/worker builds pass. Pure
service ownership tests pass for both MSI and EXE scripts without querying or
modifying services. Three standalone Windows portable-wait tests pass, including
actual child completion/failure and launch failure; these test the exact wait
module used by the packer, not a packaged application install. New scripts and
native Rust syntax parse. Full current native app/packer compilation, EXE
uninstall/service restoration, signed rollback, reboot and clean Windows matrix
acceptance remain unverified. No native uninstall/cancellation script, installer
or service operation executed on this development host.

Linux deployment now has a Dockerfile-specific deny-by-default context for named
public build inputs; Git-ignored company environment files, keys, databases and
build caches cannot intentionally be copied by the deployment recipe. A new CI
check uses Docker's scratch exporter against tracked public source and synthetic
configuration/key/cache canaries, and rejects missing or unexpected exported
files. A separate real-image rehearsal exercises non-root/read-only operation,
private storage permissions, fresh company setup, MFA/replay rejection, pending
enrollment, unattended-policy denial, graceful Docker SIGTERM shutdown, persisted
identity/approval/consent and logout after restart. Linux management now handles
SIGTERM as well as Ctrl+C.

Python syntax and fixture assembly (38 required public inputs and 11 exclusion
canaries), Compose schema/configuration and Windows GNU management compilation
pass locally. Neither new Docker execution check has passed yet: the local
Docker Desktop startup reports an inference-service socket error before its
Linux engine is usable. Existing Docker data was not reset. The new CI job must
pass before claiming this Linux deployment evidence. Public HTTPS, native
transport, full backup/restore/migration recovery, real package installation and
the complete clean Windows matrix remain separate acceptance gates.

Distribution validation now accepts the complete ten-file worker recipe for both
editions and EXE/MSI formats. It rejects changed scripts, hidden duplicate ZIP
entries, inconsistent archive headers, altered metadata and wrong company trust.
The worker verifies both artifact signatures and applies MSI identity checks only
to the installer. Nineteen management/protocol tests and the worker verification
regression pass; these fixtures do not demonstrate signed native installation.

A disposable Debian 12 VM now runs Docker. Its real scratch-export check passed:
all 38 required public inputs were included and all 11 synthetic private-data
canaries were excluded. Older Docker required root exclusion rules and explicit
child exclusions after directory allowances; both shipped ignore files now agree.
The complete management image and container lifecycle rehearsal remain pending.

At source `9988038`, the complete Windows GNU product-workspace test run passed
41 tests with zero failures. Two explicitly fixture-dependent tests were skipped:
the isolated HTTPS download-recovery harness and read-only MSI identity check.
These skips do not prove those requirements at this source. The agent suite
exercised actual read-only Authenticode checks and expected rejection paths;
no native installer, service installation or desktop session was executed.

Installer upload validation now hashes expanded binaries through a fixed 64 KiB
buffer instead of allocating a complete expanded binary. The 512 MiB per-entry
limit, signed hash comparison and ZIP CRC rejection remain enforced. Seven
management worker/distribution regressions pass, including a real compressed ZIP
fixture that verifies exact-boundary success, expansion-limit rejection and CRC
corruption rejection. Full native package generation remains unverified.

At clean source `059f675`, the dedicated HTTPS recovery harness also passed
against the current agent test executable. It verified redirect handling,
artifact tamper/interruption rejection, retention of complete staged bytes,
cleanup and absence of credentials in artifact requests. Both customer and
technician unsigned 1.4.9 MSI fixtures passed the current read-only edition,
version and recovery-plan checks, including wrong-identity rejection and unchanged
package bytes. These checks close the ordinary suite's two skipped fixture tests
for their stated scope; they do not establish current native MSI installation,
trusted production signing, service/task changes or upgrade restoration.

The first Linux management image built and its intermediate container rehearsal
passed fresh company setup, non-root/read-only execution, private storage, MFA
and replay rejection, pending enrollment and unattended-policy rejection, clean
SIGTERM shutdown, persisted company/device/consent state and logout denial.
The rehearsal exposed a test bug: Docker changed its dynamically assigned
loopback port after restart. The harness now refreshes and validates that mapping
on every start. This intermediate image required an explicit SSL_CERT_FILE test
environment; the exact `059f675` image and its default certificate environment
remain pending. This evidence does not prove public HTTPS or native transport.
