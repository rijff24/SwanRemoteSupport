# Configurable product acceptance report

Development status, 3 October 2026. **The product is not complete or approved for
production publication.** A passing component test is not evidence of a working
native support session or a clean-machine installation.

## Evidence obtained

### Corrected native Windows build inputs, 3 October

Native Windows x64 workflow `37083233120` completed successfully for exact source
`4768d530ebc79f84e84260bed42db608dd439c9c`, including the MSI registration,
watcher quiescence and durable restart-recovery changes. Artifact `11259832707`
was downloaded and verified: 109,625,027 bytes, archive SHA-256
`9419bbb73e3a99514e794e6684f3fd5884483dd7dbf6f0b5e1cd1df55abc737f`.
All 18 entries passed path/size/CRC and published payload-checksum checks.
These are unsigned build inputs. Signing, installation and corrected automatic
upgrade/interruption tests have not yet passed for this source. Its management
binary predates the HTTPS-recipe backup correction at `b6d7f78`.

The corrected agent was subsequently signed in the disposable Server 2016
guest using its private nonexportable lab certificate. SDK verification
`/pa /all /tw` passed with an RFC3161 timestamp; unsigned input hash
`e9ec48b545204f5ad8a4f0e3cb2816189a42b49e7144fe8187e1ce92782b580b`
remained unchanged, and signed output hash is
`1e02bdc3dd0302dc7cfd6a6d9b5c65512e46c0ed3fa2dbeb4e977c98659b113b`.
This proves private lab signing only. The corrected agent is not yet installed,
and no production signing approval or corrected automatic upgrade is claimed.

### Restricted technician account on installed Windows server, 3 October

The installed Server 2016 company service created a separate technician-role
account, with credentials and TOTP material retained only in protected lab
files. Actual HTTPS MFA login passed. Before group assignment, raw inventory
was empty, a grant for the approved customer device was denied, and the
administrator account-list endpoint rejected the technician. Assignment to
the lab support group exposed exactly its one approved device. The technician
session then signed out. An initial PowerShell fixture assertion counted an
empty JSON array as one pipeline item; inspection of the raw `[]` response
corrected that harness error. These are real server-role checks, not proof of
login or a remote session in the installed graphical technician app.

The installed technician agent from native source `6840d91`, verified against
its package hash and private lab Authenticode publisher, subsequently passed
actual HTTPS MFA login as that technician, authorized device inventory,
session-history retrieval and logout. Reusing its logged-out session was
rejected. Its company-state file remained byte-for-byte unchanged, so this
flow persisted no credentials there. Execution used the Administrator Windows
lab harness while the company account had technician permissions. Standard-user
CLI authentication, graphical login and native remote connections remain
unverified; earlier standard-user installation/rendering evidence is separate.

Fresh private unattended media for Server 2025 Standard Evaluation Desktop
Experience selects image 2 after reading the official ISO's WIM metadata.
Its credentials and SSH keys are distinct from the Server 2016 fixture. The
older uninitialized boot-only Server 2025 VM was retired with its disk preserved
to prepare this setup. The new fixture remains stopped: the host had about
2.5 GiB available, below its 4 GiB allocation plus host reserve. The working
Server 2016 VM and host production services remain running. This is preparation,
not a Server 2025 installation or compatibility pass.

### HTTPS configuration backup correction, 3 October

Review of the installed Windows server found that its company-data `Caddyfile`
was omitted from encrypted backups, even when certificate storage was included.
Export now includes that optional configuration, rejects links/reparse points,
and restore accepts its fixed filename in the protected replacement directory.
The complete backup test verifies its exact contents alongside transport keys,
environment and nested TLS storage. All three backup tests passed, including
wrong-password, tampering, nonempty-target and unsafe archive rejection.
Existing archives without the configuration remain supported.

The actual Windows GNU development CLI from `b6d7f78` then exported the
disposable Server 2016 management database and restored it into a separate
protected directory. A wrong password failed before creating its target.
Restored profile-signing material, setup token and Caddy recipe matched the
original files; the management database was restored. The original HTTPS
service remained healthy throughout, and no service control was performed.
A separately protected, hash-verified copy of the restored SQLite snapshot
passed `quick_check` and `foreign_key_check`, retained schema 2 and company
profile revision 7, and derived the original public profile trust from its
active signing key. Its enabled administrator retained password/MFA material;
the approved device retained unattended consent off. All four imported release
envelopes verified with the original release trust, including approved sequence
3. These checks output no credentials or private keys.

A protected copy of those restored management files subsequently started the
same Windows GNU development CLI as an isolated loopback HTTP backend on the
test host, with transport components disabled. Its signed profile verified under
the original pinned key. Actual administrator MFA login, TOTP replay rejection,
approved-device inventory, preserved unattended consent off, anonymous inventory
and grant denial, denial of an unattended grant without consent, retained release
approval and logout invalidation all passed. The owned rehearsal process stopped
afterward. The original VM service was not modified. This proves restored
management behavior; it does not prove a restored Windows SCM deployment,
HTTPS front end or transport, and transport/TLS storage was not included in
this narrower rehearsal. The coordinated full-server
rehearsal command was rejected by automatic approval review with only
“blocked by policy”; it was not executed. Full Windows deployment restore
using this correction remains unverified.

### Automatic MSI update failure and focused corrections, 3 October

The signed Server 2016 customer fixture approved release sequence 3 (MSI
1.5.2) after verifying pause, zero-percent rollout and closed maintenance
window rejection. Its agent committed sequence 3 and removed the pending
receipt, but Windows Installer still registered MSI 1.5.1. Both packages
contained the same signed native binaries: checking executable hashes alone
incorrectly skipped installation. The helper also failed to restart the
already-running configuration watcher. **This automatic upgrade failed
acceptance; a committed sequence is not proof of an installed MSI upgrade.**

The correction checks the signed package's product code and version against
Windows Installer before skipping installation, after installation and during
explicit completion recovery. A read-only test on the actual guest accepted
its registered 1.5.1 and rejected 1.5.2. The restart script now stops only the
validated installation-owned SYSTEM task before its bounded wait. Executing
that script in the disposable guest replaced exactly one watcher and preserved
device identity, credentials and consent. Agent tests passed (21 passed,
two explicitly gated integration fixtures ignored). These focused checks do
not prove a corrected signed automatic upgrade or durable post-commit recovery;
both still require end-to-end verification. At the failure, the registered MSI
was 1.5.1 while agent state claimed sequence 3.

The existing authenticated Windows worker then generated a public repair bundle
for the exact already-approved sequence 3 package (job
`07e60da5-3900-4453-91bc-334dadbd7f7f`, bundle SHA-256
`a8e2c3a76c18fbbbac18822cfae7be60688ef04ed10fa66283bd365cfe654bc8`).
Normal bundled `Install-Company.ps1 -Repair` completed in the disposable guest.
Windows Installer now registers MSI 1.5.2 and agent state remains sequence 3.
Device identity, credential, consent and consent revision were preserved,
the setup marker was removed, and both customer service and watcher run.
This reconciles the earlier failure using the historical `6840d91` signed
payload; it does not prove the corrected agent's automatic-update behavior.

The subsequent recovery change writes a signed `configuration-restart.json`
marker before committing a customer update. Successful restart removes it;
the staged recovery task retries it after validating the committed release,
installed payload and MSI registration. Support sessions remain blocked while
the marker exists, including after the updater's process lock disappears.
The agent suite passed again, and the extended session-exclusion regression
passed. A newly built, signed native agent and an interrupted-update VM test
are still required to prove this recovery behavior end to end.

The updater now quiesces the validated configuration task before replacing
customer files, including updates launched independently of the watcher.
The disposable Server 2016 test confirmed that stop-only left no installed
watcher process, then restart created exactly one watcher and preserved
identity and consent. Explicit setup completion also checks MSI registration
and retires a previous restart marker only after verifying its installation;
another automatic update is rejected while restart recovery remains pending.
The Windows GNU agent build passed. These changes still need verification in
the signed native update flow.

### Windows full-stack company server and technician worker, 3 October

The disposable Windows Server 2016 Standard Evaluation guest (Desktop
Experience, build 14393) installed the native management service from source
`6840d910c252d8fb7d5c34ca7dffbc2a2ddffb4b` using the installer at `c27e9b7`.
Its mandatory publisher verification passed with a private, guest-trusted lab
code-signing certificate. The installer deployed the pinned rendezvous, relay
and HTTPS components. Stopping the management service stopped all three owned
children; restarting it restored service. This does not prove crash recovery
or product behavior after reboot.

A private HTTPS fixture deliberately disabled automatic CA trust installation.
TLS 1.2 requests rejected its untrusted certificate, then succeeded after
explicitly importing its public CA into the guest's root store. Fresh company
setup, separate customer/technician branding, administrator TOTP login,
rejection of the consumed setup code, and worker registration passed over
HTTPS. Host trust, DNS, production services and firewall were not changed.

The real outbound Windows worker consumed an administrator-approved,
Ed25519-signed test release, downloaded and verified its company-signed
technician EXE and agent, and uploaded a completed ten-entry installation
bundle. Anonymous technician download was rejected. An authenticated download
matched SHA-256
`8308a3842d1536150585097b019acb77f4219a0cd19246778ff9c743bdfacbc6`.
The bootstrap contained only schema, edition, company identity, HTTPS endpoint
and public profile/release verification keys. Credentials and signing private
keys remain in the isolated lab, outside the repository.

These results prove a technician **EXE bundle generation and access-control
flow** on that guest. They do not prove installation of that bundle, customer
package generation, MSI signing, automatic updates, native sessions, public
Internet reachability, other Windows versions, or production signing approval.

The subsequent standard-user bundle installation passed HTTPS company setup
and profile pinning, then failed package verification with Windows access
denied: the agent attempted to write its verifier beside the package in the
administrator-extracted bundle's protected verifier file. The agent now runs its
embedded verifier as an encoded PowerShell command with quoted literal
arguments, without creating or trusting a helper in that directory. A rebuilt
GNU development agent passed package verification under the same standard
account against the same signed package and protected verifier file. Its agent
suite passed 21 tests with two explicit integration fixtures ignored. This is
verification regression evidence only: the replacement development agent is
unsigned, was not published in a bundle, and has not completed installation.

### Customer package signing, installation and explicit setup recovery

On the same Server 2016 guest, the worker generated a customer MSI bundle and
the server served it without authentication. Its downloaded hash matched the
worker result. Signing only that MSI container was deliberately insufficient:
setup enrolled the device but final installed-publisher verification rejected
the unsigned embedded customer executable. The durable setup marker remained
and the configuration task was not enabled; this was a failed acceptance test,
not a successful installation.

The customer executable was then signed and timestamped in the guest. The
other native files were compared with the original 96-file source manifest
before export. Pinned WiX built a lab-only replacement MSI with that signed
executable and the signed agent. The host used the explicit test-input build
mode because the private signing root was installed only in the guest; no
host trust bypass was introduced. The replacement MSI was separately signed,
timestamped and verified in the guest, imported as signed test metadata,
approved, bundled by the worker, and downloaded publicly. Its bundle SHA-256
was `2e7a4fe91820f2b67b2220cdc545fab3434a85987e966e51fdd6eab1941cf27d`.
Package version 1.5.1 is a private fixture built from the unchanged native
`6840d91` payload with a newly signed executable, not a published software release.

Explicit recovery now requires a strictly newer, same-edition release from the
pinned issuer, retains the original failed setup receipt, and atomically replaces
the blocking marker under session/update exclusion. Tests reject another issuer,
another edition and an older sequence while retaining state and the old marker.
The actual guest recovery used a separately rebuilt, unsigned GNU development
agent to exercise this new preparation command, followed by the worker bundle's
normal installer and its signed agent. This is not yet proof of the new
`-ReplaceFailedSetup` switch in a newly generated native release bundle.

The corrected company installation completed all package, installed-payload
and publisher checks. Both native customer and company services run, and the
SYSTEM `watch` task runs from its exact installed agent path. The setup marker
cleared, sequence two was recorded, and the original failed receipt was retained.
Device identity, device credential, consent revision and default attended-only
consent were unchanged. The server still reports the device as pending approval.
The guest has no Tailscale service. This proves local company installation and
enrollment recovery; it does not prove Internet transport, reboot, desktop
sessions, UAC, automatic-update recovery, fresh signed installation on another
guest, or compatibility on the other advertised Windows versions.

The installed signed customer agent also fetched branding revision two over
HTTPS without reinstallation. Device identity/credential, consent and software
sequence stayed unchanged. With the owned company service stopped, the agent
validated its cached profile and installed release; the service was restored
afterward. Separate protected copies of state rejected a tampered profile
signature, an older genuinely signed profile, and a mismatched company identity.
The actual installed state remained unchanged by those negative tests. This
proves agent cache behavior while management is unavailable, not visual branding
in Flutter, operation on a completely disconnected network, or offline permission
to establish a support session.

### Standard-user technician installation and read-only verification

The actual worker-generated technician EXE bundle from `6840d91` completed
installation from the standard account's Downloads directory. Package,
installed application and signed-agent verification passed; sequence one was
recorded and the pending marker cleared. The Start menu shortcut targets the
installed launcher. That launcher refreshed branding to revision two and
started the native app in the standard desktop session. Its rendered Flutter
screen shows the updated company name/logo, company HTTPS domain, username,
password and authenticator fields, plus product/upstream/AGPL attribution.
This proves installation and login-screen rendering, not successful GUI login,
inventory interaction or a remote session.

Further inspection showed that the earlier verifier regression covered an
administrator-owned helper file, rather than a directory denying every write.
Compatibility, MSI identity and MSI mode helpers still wrote beside inputs.
They now execute embedded scripts with encoded commands and literal arguments.
A new guest fixture explicitly grants the standard account read/execute only;
an attempted new file was denied. A rebuilt unsigned development agent verified
both the signed technician EXE and signed customer MSI from that fixture, without
changing any directory inputs. The customer verification fixture contains only
public bootstrap/profile information and no device credential. The agent suite
passed 21 tests with two integration fixtures ignored, including rejection of
a publisher string attempting to execute a command; no injection file appeared.
These changes still require a new native build and signed bundle verification.

The [native build at `82c8c65`](https://github.com/rijff24/SwanRemoteSupport/actions/runs/37076900806)
passed. Its 18-entry unsigned artifact was downloaded, checked against GitHub's
SHA-256 `9f9728adfda93a9cf15f8ade9338ae5986f3ebfc83d6c49c305c9f6ce6db697f`,
and extracted with path, size, CRC and published payload checksum verification.
It predates setup-replacement recovery and the remaining read-only helper fixes.
No production signing, new package installation or compatibility claim follows
from this build alone.

### Earlier component evidence

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
| Fresh-company Windows/Linux setup | HTTPS/API setup works. The pinned Linux management image passes actual non-root/read-only fresh company setup, MFA, approval/policy rejection and restart persistence. Administrator network settings and server-vantage DNS/TCP/HTTPS company-signature diagnostics are implemented with access tests. Equivalent full setup wizards, transport provisioning, certificate management and external reachability checks remain incomplete; see NETWORK_DIAGNOSTICS.md. |
| Roles, groups, permissions and revocation | MFA/group/device denial tests pass. Signed capability fields, administrator group policy APIs, and receiver capability bounds are implemented in source. Policy reduction denies lease renewal in component tests. Administration controls, native enforcement and direct-connection bypass tests remain unverified. |
| Configurable customer and technician apps | Installed customer profile sync/cache and identity/consent preservation through signed explicit/automatic upgrades and reboot pass in the Server 2016 lab. Technician GUI branding/sign-in rendering and native CLI MFA/inventory/history/logout pass. Graphical login/session flows, full visual branding, shortcuts and the remaining Windows matrix are unverified. |
| Signed profiles and rotation | Component tests reject tampering, wrong-company, expired and older profiles and untrusted key changes. Trusted cached key/endpoint rotation preserves enrollment in tests. Real isolated HTTPS trusted key rotation, missed-transition rejection and restored rotated trust pass for both CLI agents. Graphical installed-app behavior remains unverified. |
| Approved managed sessions without password/direct bypass | Grant signature, target, challenge and replay checks pass component/API tests. Actual receiving-device enforcement, direct/relay paths, lease expiry and live revocation are unverified. |
| Customer approval and unattended consent | API consent denial and durable local revocation tests pass. Server 2016 reboot preserves enrollment and consent and starts background services/watcher without desktop login. Real attended prompts, native unattended sessions, stop/uninstall, remote sign-in-screen access and UAC are unverified. |
| Transfer, clipboard, multiple monitors | Native end-to-end tests pending. |
| Public company transport without Tailscale | Source uses company transport and fails closed before setup. External NAT traversal, relay fallback and negative-access tests pending; no public endpoint has been changed. |
| Windows worker and public/protected downloads | Real authenticated signed Windows worker jobs generated customer and technician bundles. Public customer download, protected technician download denial, customer MSI installation/repair/explicit upgrade, and standard-user technician EXE installation pass in the isolated Server 2016 lab. Current-source packaging, production website deployment and the full EXE/MSI/Windows matrix remain unverified. |
| Signing and exact corresponding source | Certificate/hash/publisher pins and source metadata are implemented. Provider approval, production signatures, company signing integration and production release checks remain gates. |
| Approved automatic updates | Signed customer MSI automatic upgrades, matching-payload MSI registration, rollout controls, no-op for installed/unapproved releases, interrupted-helper scheduled forward recovery and post-upgrade reboot persistence pass on Server 2016 with private lab trust. Metadata tamper/expiry and lock/snapshot/quarantine component checks pass. Active-session coordination, power loss during MSI, signed rollback, technician automatic upgrades, production signing and other Windows versions remain unverified; see RELEASE_PAYLOAD.md and the detailed evidence below. |
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

The Linux management image built from required inputs at exact source `059f675`
is `sha256:d4308f1b5714a1cd56987af5aa0c3f51db31e64b33c104e2e610638a0786129a`.
Its management executable hash is
`db7716c7dc61a13af14de68dbd4e197b30ca8b565966638e9e8303ec069b5d9b`;
the public certificate bundle matches the pinned builder. The corrected test
driver, maintained separately from the image build inputs, passed default
certificate-path checks, non-root/read-only startup, fresh setup, MFA/replay
rejection, approval/consent policy, graceful SIGTERM and restart persistence.
The expanded driver then passed actual encrypted management CLI export and
restore to a separate volume, wrong-password authentication rejection with no
target writes, private key permissions, retained branding/policy/device/consent,
fresh MFA login, replay/logout denial and revoked-device rejection. Revocation
in the restored deployment did not mutate the original volume. No backup
passphrase was placed in command arguments or Docker container metadata.
These results prove management-container recovery only. Full transport/TLS
volume recovery, public HTTPS/ACME, native sessions, installers and the clean
Windows matrix remain required acceptance gates.

The pinned Caddy `2.10.2` and RustDesk server `1.1.15` images now pass an actual
isolated component-stack rehearsal with the exact `059f675` management image.
The proxy recipe uses a disposable internal issuer with trust installation
disabled; HTTPS validates through an explicit test CA, while default system
trust rejects it. Fresh company setup, MFA replay rejection, pending enrollment
and logout denial pass through HTTPS. OpenSSL verifies actual management profile
signatures and rejects modified payloads and wrong keys. Both rendezvous TCP
ports return real native TestNatResponse messages before and after restart;
relay TCP accepts a local connection. Company signing identity, transport public
key and the configured HTTPS endpoint survive restart. The test removes only
its uniquely named containers, volumes and network. Production recipes, DNS,
firewall and host trust stores were not modified for this rehearsal.
These results do not prove UDP or external NAT traversal, native remote-session
authorization, public ACME issuance/renewal or full transport/TLS-volume restore.

The extended Linux stack rehearsal now passes complete encrypted CLI export of
management, actual transport keys/database, private deployment settings and
Caddy certificate storage, followed by restore into separate volumes. Incorrect
passwords fail before target writes; recovered transport keys, environment and
every TLS file match the originals. Replacement management, rendezvous, relay
and HTTPS containers start using the restored data. The original CA validates
HTTPS, company profile signatures and transport identity persist, and fresh
MFA login, replay/logout denial, pending enrollment, consent and device
revocation pass. The restarted original retains its pending device after the
replacement revokes its own copy. The compiled management image still uses
exact required source inputs at `059f675`; only excluded test drivers changed.
This establishes isolated component recovery, including actual transport and
TLS storage. It does not establish public ACME, external traversal, authorized
remote desktop sessions, populated transport-peer recovery, migration rollback,
native Windows installation or the compatibility matrix.

Windows server component preparation now passes with the actual pinned
RustDesk server 1.1.15 and Caddy 2.10.2 Windows archives. Both archive SHA-256
values match upstream release asset metadata; extraction produces only the
expected component executables and the public manifest. Read-only PE inspection
confirms x64 executables. An invalid archive is rejected before output creation.
All three supplier executables report `NotSigned`; hash pinning does not supply
production Authenticode trust. No supplier installer or component was executed,
and no services or firewall settings changed. Combined Windows server setup,
service supervision, HTTPS provisioning, signing and native lifecycle tests
remain incomplete.

Windows management now supports supervised packaged rendezvous, relay and
HTTPS children. The full-stack install script pins their executable hashes,
retains license/source notices, validates the company hostname, rejects
occupied ports and existing data, configures the proxy and private storage,
and checks management startup. Windows GNU management tests pass: 18 ordinary
tests, with two child-process fixtures invoked by lifecycle tests. Actual
Windows child processes are reaped on stop and partial startup failure;
component exit stops management. A process-owner crash test observes its
assigned child terminate through Windows job-object cleanup. Private preflight
with the actual supplier binaries accepts their hashes, and changing one byte
of the relay rejects it without starting services. PowerShell scripts parse,
and preparation preserves upstream licenses/source references.
The privileged installer itself has not run on a clean Windows VM. SCM startup,
firewall rollback, reboot/service recovery, public HTTPS/ACME, live Windows
transport sessions, repair/uninstall and the graphical setup wrapper remain
unverified or incomplete. These process tests do not satisfy those gates.

Server installation now records public ownership metadata in protected company
storage. `Uninstall-Server.ps1` validates that receipt and exact service command,
refuses redirected directories/files and unexpected firewall ownership, then
removes the company service, its rules and known packaged files while retaining
all company data. Removal-plan checks pass under both PowerShell 7 and Windows
PowerShell 5.1, including wrong executable/arguments, company paths, rule names,
duplicate rules, invalid policy types, a directory substituted for a file and
an actual directory junction. The fixtures preserve their identity data and do
not run SCM or firewall operations. The installer additionally requires a valid
management signature matching an explicit trusted certificate thumbprint.
Native privileged install/uninstall, retained-data reinstallation and full
Windows compatibility remain unverified; receipt planning is not their proof.

The Windows server setup form now collects company hostname, public release
trust, expected publisher, signed management and prepared components, then
invokes the installer with structured parameters in a background PowerShell
instance. It disables installation without administrator rights and prevents
closing during installation. Successful completion enables HTTPS company
setup; failures show the installer error. Its Windows PowerShell 5.1 preview
renders and was visually inspected, without displaying the form or starting
services. This proves layout construction only. Privileged wizard actions,
interactive setup, backend completion/error events and a signed packaged
launcher remain unverified or incomplete.

The exact `8cfdd8f` required public source inputs now build the pinned Linux
management image `sha256:0410d99dd9995bf95854dff50621030aeefa567f73b07e5b4f4dd381cdfd5fb3`.
The source archive SHA-256 is `3b40abde298095bad99347aad1bf5ee3a9b5ed855ea1788d8926038cc81cd72a`;
the management binary SHA-256 is `c0182c6785480dc47b5220689146da3bee7d00362f17ee1f400233e39c0199a6`.
The context check includes all 39 required inputs and excludes every private
configuration/key/cache canary. Both deployment drivers from the same source
pass: fresh setup, MFA/replay, approval, restart, management encrypted restore,
and complete transport/TLS replacement recovery using the original trust.
This is exact-source isolated Linux deployment evidence, not public ACME,
external traversal, native sessions or clean Windows compatibility proof.

The exact `8cfdd8f` Linux management executable also passes a separate offline
rebuild comparison. The rebuild reused the pinned toolchain and downloaded
locked dependency sources; old target artifacts were hidden by a new `volume-nocopy` volume
and verified empty before compilation. A full 84-second release compilation
without network access produced the same `c0182c6785480dc47b5220689146da3bee7d00362f17ee1f400233e39c0199a6`
SHA-256. The first attempt exposed Docker's default volume population and was
discarded as cache reuse, not reproducibility proof. This proves the Linux
management binary in that pinned environment, not byte-identical OCI images,
native Windows binaries, signed packages or all product components.

An x64 Windows .NET Framework test launcher now packages the setup wizard,
installer, management executable, pinned rendezvous/relay/HTTPS binaries and
license/source metadata into one EXE. Its first package compiled and rendered
the bundled form using Windows PowerShell in-process; the window shows
`UNSIGNED TEST`. Non-elevated preview exited successfully and removed its known
temporary payload, without registering services or changing firewall/trust
settings. Implicit production builds and a modified relay input are rejected
before output creation. The builder verifies compiled resource hashes; the
installer rechecks the copied management signature before registering its
service. Final package verification and elevated extraction/installation on
clean Windows VMs remain necessary. Test packaging is not signing approval,
production publication or clean-machine compatibility evidence.

The final test server package from clean source `eb1d580` passed compiled
resource verification and its non-elevated, noninteractive preview, leaving no
temporary payload. Its SHA-256 is
`db5a93275f277222b51719b01b2d0d6a909c4e3a556f34d86c691485625a5c25`;
its management input is the existing debug test binary, and the package remains
unsigned. Separately, [Windows x64 build 37030567046](https://github.com/rijff24/SwanRemoteSupport/actions/runs/37030567046)
completed successfully for source `215dfef3fc80799f7077cfe314ecad7b1c1ebc30`,
including the native app and company components. The resulting
`SwanRemoteSupport-1.5.0-x64-unsigned` artifact is a build result, not installation,
session, reproducibility or signing proof. Later source changes still require
their own applicable validation.

Server installer preflight now aborts if TCP, UDP, firewall or service inspection
fails, and checks the management port for management-only installations too.
Enumeration is checked before filtering, so a missing named resource is not
confused with an inspection failure. Windows PowerShell 5.1 and PowerShell 7
parse the updated installer; read-only host endpoint and firewall enumeration
succeeded. No privileged installation or host configuration was performed.

The Windows build artifact above was downloaded and verified against GitHub's
published archive digest
`d61e9589d671d10db286cf7007e6cf2c3e58627b34ce1df75c184c8420509d87`.
Safe extraction validated entry paths, duplicates, sizes and ZIP CRCs; all four
EXE/MSI checksums match the included checksums. Read-only MSI inspection confirms
both packages report version `1.5.0`, `x64;1033` and their distinct stable upgrade
codes. The customer MSI SHA-256 is
`b8268332cd61d26c37fd4a2aa8c56fa9c84648b5dd7dc9484a0b04b3d216f47d`;
the technician MSI SHA-256 is
`e997dc9afa490155b9bf9be1c54645cea1ff9fe5a2d73d3e2461454d3bbd8184`.
Both remain unsigned and were not installed on the host.

A separate fresh Server 2016 Standard Evaluation Desktop Experience VM accepted
private unattended media and completed file preparation, then failed while
processing the `specialize` pass. Its disk is preserved for read-only setup-log
diagnosis. This is a lab provisioning failure, not a product installation result;
the Windows compatibility row remains unverified.
Read-only inspection of its Panther logs identified `Microsoft-Windows-Deployment`
`RunSynchronousCommand/Path` as the invalid value (`0x80220005`). The setup
command was 280 characters; Microsoft's documented limit is
[259 characters](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/microsoft-windows-deployment-runsynchronous-runsynchronouscommand-path).
A new private provisioning seed uses a 192-character command and validates this
bound before generation. The failed disk remained hardware read-only throughout
diagnosis. The corrected seed still requires an actual successful installation.

The corrected private VM subsequently completed installation of Windows Server
2016 Standard Evaluation with Desktop Experience, x64 build `14393`. Encrypted
NTLM WinRM verified the guest identity and administrator context without changing
host trust settings. Both unsigned `1.5.0` MSIs identified above passed actual
`msiexec` install, repair and uninstall with exit code `0` and no requested reboot.
The harness SHA-256 was
`26ad362ee1d5d1793ee3ee35548d18ab84efc2973f4a663a3d62de50a53bf385`.
Customer installation and repair verified all 96 manifest files, the agent hash,
and the running LocalSystem service's exact executable. Repair restored a
deliberately damaged packaged license file. A subsequent read-only cleanup audit
confirmed all 96 files were absent and Windows Installer reported the customer
product unregistered (`ProductState=-1`). Technician checks verified its main
executable and agent hashes; repair did not deliberately damage a technician
file. Both lifecycle tests ran under the guest administrator account.

This is bare MSI lifecycle evidence for one Windows build. Standard-user
technician installation, upgrade, reboot, company bootstrap/enrollment, trusted
publisher validation, automatic updates, remote sessions and the remaining
Windows compatibility matrix are still unverified. VM credentials and raw
installation logs remain in the private lab outside the repository.

A separate standard account was tested on the same Server 2016 VM. Initial
batch launch was denied by the guest's batch-logon rights; granting only that
right allowed the task to execute without administrator membership. SSH and
batch MSI attempts returned `1601` because they could not connect to Windows
Installer. After an actual standard-user desktop logon, the MSI instead returned
`1625`. Its log reports effective `DisableMsi=1`, rejects a non-assigned
application for a non-admin account, and confirms the package is marked for
installation without elevation. No installer policy was changed. The denied
installation left the product unregistered and endpoint/agent absent. This
does not establish successful standard-user installation; company-managed
assignment and client Windows editions still need testing. See Microsoft's
[DisableMSI policy](https://learn.microsoft.com/en-us/windows/win32/msi/disablemsi).
The temporary private VM logon password was removed automatically, with cleanup
and disabled auto-logon verified after reboot. This reboot provisioned the test
desktop; it is not product reboot acceptance.

The lifecycle runner now records actual account elevation, supports an explicit
standard-user technician requirement, and checks installer registration plus
packaged-file, shortcut and component-registration cleanup. It reads OS version
from the registry so standard-user reports do not require WMI permission, and
writes a failed result even if process creation fails. The failed interactive
test recorded runner SHA-256
`4c70cfe5a4f8ce60e599460febb7bfab5a166860af995e9260df13f12be96396`.

[Native Windows build 37065751842](https://github.com/rijff24/SwanRemoteSupport/actions/runs/37065751842)
completed successfully for source
`6840d910c252d8fb7d5c34ca7dffbc2a2ddffb4b`, including the later company
management and transport-supervision source. Its unsigned artifacts still
require their own integrity, installation, company setup and session checks.

That newer artifact was subsequently downloaded and checked against GitHub's
published SHA-256
`4099acc0e992e8369738515c74a34d26625e51fc567ba5e3955f82b76d28a813`.
Safe ZIP extraction checked all 18 entry paths, sizes, CRCs and included payload
checksums. Its unsigned management, agent and worker binaries remain separate
from the earlier MSI lifecycle results.

Actual optional company signing now passes on the private Server 2016 VM using
Microsoft Windows SDK SignTool from the Microsoft-owned NuGet package
`Microsoft.Windows.SDK.BuildTools 10.0.26100.1`. The package matched its published
SHA-512; the x64 tool SHA-256 is
`38da9179b56d88adc0694ebf7393b9c44727c344e2df8dfa32399a4c9694715c`.
Its Microsoft Authenticode signature was verified on the host and guest before
execution. A non-exportable software-provider lab key and a certificate trusted
only in the disposable guest signed a new copy of the source `6840d91` agent.
The original agent SHA-256
`d36cc9e4fea3b3331d64013a9c8a25f17e80f1456c84be16aef074b4d2bdd526`
stayed unchanged; the signed copy is
`956ecc4462841cfb94372429be6d33dfe1b82a25069213693b601aca4f9722cf`.

The first SDK attempt rejected the HTTPS timestamp URL. The integration now
accepts the operator's explicit HTTP or HTTPS RFC3161 URL, without automatic
fallback, and rejects credentials, queries and fragments. DigiCert's documented
HTTP endpoint returned a timestamp that SDK verification accepted. Server 2016
PowerShell nevertheless returned a null `TimeStamperCertificate`; the integration
now relies on SDK `/pa /all /tw` verification rather than that unreliable field.
An otherwise trusted signature with no timestamp produced warning exit `2` and
was rejected by the same verification gate. The valid signed agent passes the
pinned publisher/certificate verifier under both administrator and standard
accounts. Wrong publisher, wrong certificate and a one-byte tamper are rejected;
the tampered fixture reports `HashMismatch`. Unsafe timestamp URLs created no
artifact or signing receipt. The tested signing-script SHA-256 is
`2c1baf6d00fe7e876f553341d221389b97dbd10fe6a4eeed6c5b3801aa2d1f4c`.

No lab signing key was exported, no host trust store changed, and no production
signing approval was obtained. This validates one EXE signing integration, not
signed MSI publication, worker consumption, complete company installation,
automatic updates or native sessions. Raw logs and test artifacts stay private.

### Corrected customer upgrade candidate: source 4768d53

The complete customer payload from successful native CI run `37083233120`
was extracted read-only from its MSI with 7-Zip; no MSI action was executed.
All 96 installed-file hashes matched the CI manifest. The native application,
which embeds the agent library, is included alongside the standalone corrected
agent; this avoids retaining the older application's session-lock behavior.

A separate payload copy was built into lab MSI `1.5.3` with pinned WiX
`5.0.2+aa65968c`. The build used `UnsignedTest` because the private disposable
publisher is trusted only inside the guest, not on the host. The main executable,
agent, and final MSI were independently signed in Server 2016 build `14393`
using the non-exportable lab certificate and explicit RFC3161 timestamp endpoint.
The signing integration required successful SDK `signtool verify /pa /all /tw`.

- Signed native main SHA-256: `89a92acab159d2b34ddaf3e99ebcaa6ad1a45c6450876598b48237c1fc531a2b`.
- Signed agent SHA-256: `1e02bdc3dd0302dc7cfd6a6d9b5c65512e46c0ed3fa2dbeb4e977c98659b113b`.
- Unsigned MSI SHA-256: `5ae64da9b413994e94bee388c15c06142d5ac00e633f158aef490df8e91af3c2`.
- Signed MSI SHA-256: `ed87625128fe6c84738cf7b6fd084da25413ddd538dd8896bac0af74efcbd6f1`.
- Installed manifest SHA-256: `0f9e04800d770eeda45eadb043468923fc57e71e5b4d6a5e75711dec4eaef568`.

This is a private upgrade candidate, not an approved production release.
It has not yet been imported, installed, or tested through automatic rollout.
The existing repaired installation remains version `1.5.2`; full automatic
upgrade and interruption/recovery acceptance remain outstanding.

The corrected candidate was subsequently imported as `4-Customer-test`, approved
while automatic rollout was paused, and bundled by the existing authenticated
Windows worker (PID `3568`). Build `9506bd95-b3da-4119-877b-06255a2a9e60`
completed; public bundle SHA-256 is
`e28a2f73f79d9e86342f2ca0a817498b32d1d58859ca5e68693fee5dc6fc6c52`.
The public download's hash was checked before extracting and invoking its normal
bundled installer, without repair mode. On Server 2016 build `14393`, the explicit
upgrade completed with registered MSI version `1.5.3` and release sequence `4`.
The installed main and agent match the corrected signed hashes listed above.
Device identity, enrollment token, unattended consent and consent revision were
compared privately and remained unchanged; the pending setup marker was absent,
and both customer service and configuration watcher were running. This used the
existing source `6840d91` server/worker packaging recipe with source `4768d53`
endpoint binaries. It proves this explicit lab upgrade, not automatic rollout,
interrupted recovery, current-source worker packaging or the Windows matrix.

### Corrected automatic MSI upgrade: Server 2016

A new lab MSI `1.5.4` was built from the same signed source `4768d53` endpoint
payload as `1.5.3`, specifically exercising the case where executable hashes
match but registered MSI version must change. The final signed MSI SHA-256 is
`c1dad7360fd47a4eb773b4405b55caa217558f49b47341e4d3b3940b690d6ea0`;
SDK timestamp verification passed. Signed release `5-Customer-test` was imported,
approved, and enabled with full rollout and open maintenance window. An expired
administrator session stopped the first harness attempt before release approval
or update task creation; a fresh private MFA login allowed the actual attempt.

The installed corrected agent's `update` command ran as SYSTEM through task
`SwanLab-CustomerUpdate5`; it downloaded and launched its signed staged helper.
While helper PID `3384` was live, the receipt moved through installation and
configuration restart. Windows Installer changed from `1.5.3` to `1.5.4` before
release sequence committed from `4` to `5`. Final inspection found no pending
setup, update, or configuration-restart markers, one installed agent process,
a running configuration watcher and customer service, and task exit `0`.
Device identity, enrollment token, unattended consent and consent revision were
compared privately and preserved. No helper was restarted because of observation
timeouts. This demonstrates the corrected matching-payload automatic MSI upgrade
on Server 2016 build `14393`, using private lab publisher trust. It does not prove
interrupted recovery, active-session coordination, reboot behavior, automatic
technician upgrades, other Windows versions or production signing approval.

A subsequent invocation of the installed signed agent's `update` command returned
`Update handed off: false` while release `5` remained the highest approved release
and newer `6-Customer-test` was imported but unapproved. Sequence stayed `5`, no
pending-update marker appeared, and private identity/token/consent comparisons
passed. This verifies the installed-release no-op and that the unapproved newer
candidate did not reach installation; it is not a forged-metadata replay test.

The private interrupted-upgrade candidate is MSI `1.5.5`, source `4768d53`,
signed SHA-256 `c1d8d2f937e047b424fb68077c61beb83dd5da59aedd1a5d40821768f52107dd`.
Its SDK timestamp verification passed and its signed metadata was accepted as
`6-Customer-test`, unapproved. No interruption or recovery test has yet run for
this candidate. The recovery task source validates its exact staged helper,
principal and command, with startup and five-minute retry triggers; runtime
recovery still requires direct evidence.

Release `6-Customer-test` was then approved and the installed updater started as
SYSTEM. The intended interruption harness incorrectly interpreted receipt phase
`installing` as evidence that MSI execution had begun; this phase is assigned
before helper execution. Its guard failed without terminating any process. A
follow-up guarded interruption attempt found installation already completed and
also terminated nothing. The original update was neither restarted nor replaced.
Final inspection showed registered MSI `1.5.5`, sequence `6`, no recovery markers,
running customer service and configuration watcher, one installed agent process,
and preserved identity/token/consent comparisons. This is a second completed
automatic upgrade, **not interrupted-update recovery evidence**. The interruption
harness must observe a reliable execution boundary on a new candidate.

### Interrupted staged helper: recovery pending

A new private signed MSI `1.5.6` (SHA-256
`426995bfa864ea0081b3acee7ad544baadb5763291f467bcd688f8563c2aea0b`)
was imported and approved as `7-Customer-test`. The guarded lab harness observed
its live staged `apply-update` helper, verified its exact executable hash,
release/previous sequence and registered SYSTEM recovery action, then terminated
only helper PID `772`. This interruption was intentional; no MSI child, service,
worker or host process was terminated. Immediate inspection found MSI `1.5.5`,
sequence `6`, the retained pending-update receipt, unchanged enrollment/consent,
and the original running customer service and configuration watcher. The staged
helper was absent and its recovery task was Ready with the normal retry scheduled
for `2026-10-03T02:25:25Z`. Recovery completion is not yet demonstrated.

Source now offloads final configuration verification/restart from async `main`
to `spawn_blocking`, preserving existing error propagation. Windows GNU
`cargo check --manifest-path product/Cargo.toml -p swan-agent --locked` passed.
This change is not part of the installed source `4768d53` test binary.

The normal scheduled retry ran at approximately `2026-10-03T02:25:25Z` without
manual task start or helper relaunch. Its result was `0` and one staged helper
was observed live. Subsequent inspection found registered MSI `1.5.6`, committed
sequence `7`, no pending setup/update/restart markers, running customer service
and configuration watcher, one installed agent process, and preserved private
identity/token/consent comparisons. The staged helper had exited. This demonstrates
automatic forward recovery after terminating the verified pre-install staged
helper on Server 2016 build `14393`, using private lab signing trust. The completed
recovery task was still Ready with its next normal cleanup check scheduled for
`02:30:30Z`; self-removal has not yet been observed. Power loss during MSI,
rollback after installer failure, reboot/startup recovery, active remote sessions
and other Windows versions remain separate acceptance requirements.

### Reboot persistence after recovered update

The disposable Server 2016 guest was rebooted through its own `shutdown.exe`,
after verifying no active update helper or recovery marker remained and saving
protected private state/hash comparisons. The host and production services were
not rebooted or changed. A different guest boot time was observed. At inspection,
no `explorer.exe` desktop session existed, while `SwanCompanyServer`, the customer
service and `Swan Company Configuration` watcher were Running. MSI registration
remained `1.5.6`, release sequence `7`, both endpoint hashes matched the pre-reboot
snapshot, and identity/token/consent comparisons passed. No recovery marker was
present. The completed release7 recovery task removed itself at startup (zero
matching tasks). Nothing was manually launched to satisfy these startup checks.
This demonstrates reboot persistence and automatic background startup on Server
2016 build `14393`; it does not demonstrate a remote logon-screen session, UAC
control, recovery from power loss during MSI, or other Windows versions.

### Installed native consent synchronization

On Server 2016 build `14393`, installed signed agent source `4768d53` rejected
`allow-unattended` without `--confirm-unattended`; local consent/revision and
identity/token comparisons remained unchanged. With the isolated company policy
temporarily allowing unattended support, `sync` followed by explicit
`allow-unattended --confirm-unattended` enabled consent locally and in the actual
HTTPS server inventory with an increased consent revision. `revoke-unattended`
then disabled both local and server consent with another revision increase.
Enrollment comparisons passed. Cleanup revoked consent and restored/synchronized
the original company policy. Credentials and raw state remained private.
This tests the installed native CLI consent path; graphical consent controls,
actual unattended sessions and termination/rejection of live session grants
still require separate evidence.

The installed native consent rehearsal also exercised actual HTTPS grant/lease
APIs: an authorized administrator obtained an unattended signed grant after
explicit consent, the device claimed it and renewed its live lease, and a second
claim of the same grant was denied (`403`). After native consent revocation,
renewal of that claimed grant and issuance of a new unattended grant were both
denied (`403`). Original company policy and disabled local consent were restored;
enrollment comparisons passed. Grant bodies, device tokens and account sessions
were kept private. These are real server authorization checks, not an established
native remote session: receiver termination after lease expiry and graphical
technician/customer interaction still require end-to-end verification.

The actual HTTPS group-permission API was tested against the enrolled Server 2016
device: after claiming an attended grant with the original support permissions,
removing keyboard and clipboard permission from `lab-support` caused renewal of
that existing broader grant to be denied (`403`). A newly issued signed grant
contained both capabilities disabled. Test grants were closed and the original
group policy restored. This verifies live server policy reduction and grant
capability issuance; it does not prove receiving-device input/clipboard denial
or termination of an established native session.

### Windows worker SCM installation rehearsal

The worker service source `03a8739` built with the pinned Windows GNU debug
 toolchain (`cargo build --manifest-path product/Cargo.toml -p swan-worker --locked`).
Unsigned SHA-256 is `04f361a9b9c25521041f0482ced1870fa079295cb18abd88a53bc9a22ab8ae6d`;
its separately timestamped lab-signed copy is
`f5a4dfb3528ea9794b148dabc2edd80f1c61b1d033cb869fdd20b4411038df8f`.
The new installer initially stopped before mutation because PowerShell 5 hides
`C:\ProgramData` without `Get-Item -Force`; the corrected parent check allowed
installation. On Server 2016 build `14393`, `SwanInstallerWorker` started as
LocalSystem with automatic startup (PID `1020`). A real standard-user SSH session
could read neither the service registry environment nor its protected installation
receipt. Credentials were never printed. Signed build completion under this new
service, stop/restart, reboot, removal and log collection remain unverified or
unfinished. This GNU fixture is not a production release or MSVC matrix result.

Controlled SCM stop/restart of the signed GNU worker service passed: the verified
installed worker process PID `1020` exited after Stop-Service, and Start-Service
created a new owned process PID `1856` with the same service executable identity.
This is service lifecycle evidence, not an authenticated completed build or
reboot-startup result. Native build artifacts now include `Install-Worker.ps1`
alongside the worker binary for matched-source deployment testing.

The signed GNU worker service also passed guest reboot startup. A new Server 2016
boot time was observed, SCM automatically launched the owned worker as PID `1556`,
and no desktop session was logged in. Protected service environment values and
worker executable hash matched private pre-reboot comparisons. The company and
customer services were Running; customer enrollment/token/consent were preserved.
No manual worker start was used. Authenticated build completion by this new worker
service and its MSVC release remain pending. Current-source native CI run
`37090939153` (`6bd54c21fab4ec8d3203f513c616060af3fd2d26`) was confirmed in progress;
its result is not yet known.

Worker removal ownership validation and `-WhatIf` were exercised against the
installed signed GNU service on Server 2016. The original running process ID
and executable remained unchanged; no service or file was removed. Actual
removal, retained-data reinstallation and server credential revocation remain
pending. Native CI run `37090939153` was still in progress on the latest check;
no duplicate build was dispatched.

Worker service logging source `148eae7` was built with the pinned Windows GNU
 toolchain and separately lab-signed (SHA-256
`facfd31f73a1949031523e88ad0213d65c973f71fe37c4fcd453749e97dceea5`).
The disposable service's previous executable and receipt were preserved privately
before controlled replacement. Temporarily substituting an invalid worker token
produced `Worker polling failed: 401` in the protected service log. A private
comparison confirmed the real token was absent from the log. Cleanup restored
the original service environment and restarted the logging-enabled worker.
This proves runtime stderr capture for this GNU service fixture; standard-user
log denial, rotation and authenticated build completion remain to verify.

A real standard-user SSH session on Server 2016 was denied reading the created worker-service.log. Native CI run 37090939153 remains in progress; no replacement run was dispatched. Runtime log rotation and authenticated service build completion remain unverified.

Actual owned-worker removal and reinstallation passed on Server 2016 with signed GNU source 148eae7. The service and executable were removed, the unchanged protected receipt and artifacts were retained, and the lab explicitly archived that owned data directory before reinstalling. The worker, company server and customer services were Running afterward. Removal now disposes its ServiceController before SCM deletion to avoid retaining a deletion-blocking handle. This is not automatic retained-data migration or server-side credential revocation evidence.

Administrator worker listing and revocation now have management handler coverage.
The locked worker-completion test group passes all eight tests, including
credential-free listing, rejected technician revocation, atomic cancellation of
running/uploaded jobs, denied revoked claims/completion, rejected late uploads,
preserved unrelated/completed jobs, and audit records. This is local backend
coverage; the new endpoints still require paired-server VM verification. Native
run 37090939153 has progressed to Build branded client for source 6bd54c2.

Worker revocation is now available in the installer administration view. JavaScript syntax validation passed. The eight locked worker-completion tests passed again with real Axum route checks for unauthenticated and technician denial, administrator list/revoke, unknown-worker 404 and revoked claim/upload 401. Claims recheck credential status inside their database transaction to close the authentication-to-claim revocation race. These route tests do not replace paired-server VM acceptance.

Source `11fd72a` management was built with locked dependencies and tested as a
separate process on disposable Windows Server 2016 Desktop Experience, build
14393. Test binary SHA-256:
`3ad52ad04c4af5be55e29a1bef036d2b5a9dbc2a0f7832931ccb880117b5959a`.
A fresh private data directory and guest loopback HTTP listener passed company
setup, subsequent MFA login, worker creation, enabled claim, unauthenticated
worker-list denial, credential-free administrator listing, administrator
revocation, disabled status, audit event, and revoked claim/upload 401 responses.
The owned test process was stopped afterward; existing company, worker and
customer service statuses remained unchanged. Credentials, logs and test data
stay outside Git. This unsigned GNU fixture does not establish production
MSVC signing, HTTPS deployment, cancellation of a real in-flight worker build,
or the complete fresh-company installer workflow.

Native Windows CI run `37090939153` completed successfully for exact source
`6bd54c21fab4ec8d3203f513c616060af3fd2d26`. Installer artifact `11263621948`
was downloaded outside Git and verified against GitHub's archive SHA-256
`e91bfb162cc8ac5bf1428fadffd057b1741514c505ae987c68f41d7b605a9c7a`
(109,618,735 bytes). All 19 flat ZIP entries passed bounded extraction, CRC and
published payload checksum checks. This includes customer/technician EXE and MSI
test packages and matching MSVC management, agent and worker executables.
Management SHA-256: `58dc0db9e3aaaf1cf3fda3a4552ce5b8e4766721c5877d59d840d1d85a31531e`;
worker: `c33a766574a5a91ff57fc1661af2ace7c3799aace8df3490c599fd96cf375f7c`;
agent: `870ead5b2307c289d3e94542b53756488ea0ae6e2a558ed4131e9e49e6eaa9ee`.
These are unsigned build outputs, not production releases or installation/session
acceptance. Changes after this source, including worker logging, removal and
administrator revocation, require separate native verification.

The matching native MSVC management, worker and agent from `6bd54c2` were signed
inside disposable Server 2016 using its private, nonexportable lab certificate.
All three passed Authenticode plus `signtool verify /pa /all /tw` timestamp
verification. Signed SHA-256 values: management
`27285aa8b371f38d26ab91c16a00c3a27fbc90e42c03085ab8e358e4b8ef8ef7`, worker
`f147d31480e40d436f32c160cd47b22c821129c2a39652bcbe81e8d11b42bc13`, agent
`68cef92acfc983445c59fc43da7e5817096ae76cde2e6260c2f8bc47113c01fd`.
The signed MSVC management executable then passed a fresh isolated company setup,
MFA login, worker creation, authenticated empty-queue claim, unauthenticated
claim denial and worker creation audit test over guest loopback HTTP. Cleanup
stopped that owned test process and verified existing service statuses unchanged.
This is native server runtime and lab signing evidence; it does not prove HTTPS
worker build completion, Windows service installation of these binaries, client
sessions, the wider Windows matrix, or production signing approval.

The signed native `6bd54c2` management fixture also passed trusted HTTPS access
through a separately launched Caddy instance on a guest loopback port. Its
company signing identity was distinct from the existing disposable deployment.
The proxy reused the existing guest-only lab certificate read-only, with its
admin API and automatic HTTPS management disabled and separate proxy storage.
Both owned test processes were stopped after verification; existing service
statuses remained unchanged. No host DNS, trust, firewall or service setting was
changed. This establishes isolated HTTPS management runtime, not public
reachability or completed worker generation.

Matching lab-signed native MSVC management and worker source `6bd54c2` completed
real customer and technician build jobs on isolated Server 2016 over trusted
guest HTTPS. The administrator configured the isolated company endpoint,
imported and approved signed test releases, created a worker credential and
queued both jobs through the API. The foreground worker verified signed inputs,
generated bundles, uploaded them and reported successful completion. Downloaded
bundles matched the reported hashes and each contained the exact ten-file recipe
with the correct company endpoint, profile trust pin and edition in bootstrap.
Customer job `4ebc6935-83ff-4214-9763-257b354346e2`:
`7b6d797dfe3a7bf7095a65293025995067d715cd73c5777ab6bf5cb1bf3d4e9c`.
Technician job `1604fde0-6007-4af7-8994-04cdcf21b706`:
`73783c1d987691bec8ca802a2b443c98a00619b1fa2980b049443fceb342f4be`.
Customer download succeeded without authentication; technician download returned
401 without authentication and succeeded with the administrator session. Cleanup
stopped the owned foreground worker, proxy and management processes and checked
existing service statuses. Test credentials and artifacts remain outside Git.
Payloads were earlier signed lab releases from `4768d53` (customer MSI) and
`6840d91` (technician EXE), not newly signed `6bd54c2` client packages. This proves
the matching native server/worker generation path, not worker service completion,
fresh client installation, production publication, or remote-session acceptance.
