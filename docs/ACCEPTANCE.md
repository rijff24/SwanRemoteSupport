# Configurable product acceptance report

Evidence through 28 October 2026. **The product is not complete or approved for
production publication.** A passing component test is not evidence of a working
native support session or a clean-machine installation.

## Latest source checks, 28 October

The Server2022 endpoint installed the actual generated 1.5.4 customer bundle
again after its prior uninstall. The original zero-byte cancellation marker was
retained for shipping setup to resolve. Enrollment, all 96 signed files, service
and watcher passed; pending-device, unassigned-technician and unattended-without-
consent grants were denied. Its installed verifier accepted the exact repair MSI
and rejected changed bytes, invalid metadata signatures, wrong publishers,
wrong certificates and replayed release metadata without an installation handoff.
A standard user then opened the actual customer interface, saw company branding,
and enabled/revoked unattended consent; independent server checks matched and a
new unattended grant was denied after revocation. This is a reinstall checkpoint,
not a new pristine-OS or complete remote-feature qualification. See
[the Server2022 evidence](../deployment/windows/a6670d6-server2022-reinstall-and-standard-user-findings.json).

That standard-user test also found two release blockers. The advertised shortcut
started MSI repair because detection could not inspect the deliberately private
configuration-agent keypath (MsiInstaller event1004). The UI eventually opened,
but ordinary startup must not invoke that uncoordinated repair. “Stop support”
also requested administrator credentials through the inherited service-uninstall
path; the prompt was canceled. The 1.5.5 source disables advertised shortcut
generation and changes managed Windows stop/resume to service IPC with receiver-
enforced pause, generation-bound authorization and cancellation of in-flight
renewal. A native regression and installed-shortcut check were added. Its consent
dialog also becomes scrollable after long policy text was visibly clipped.
A subsequent source audit found the remote toolbar still handing file-transfer
windows a legacy password token. Managed windows now return only a current,
target/company/login-bound opaque request handle, which the toolbar passes as
the managed preset so native authorization obtains a fresh grant and proof for
each file-transfer connection. A native regression checks stale login, wrong
company, wrong target and expiry; no signing proof or reusable credential is
returned to Dart. Real file-transfer operation remains unverified.
Both 1.5.5 native builds passed. Comparison run 40616863517 verified identical
original bytes for all eight shipping outputs, matching compiler inputs and exact
public source/bridges. The native pause and transfer-handle regressions passed.
The existing nonexportable lab key signed seven outputs; the Windows worker
generated both company bundles, whose source texts, signatures, hashes and public
customer/protected technician downloads verified. Automatic rollout remains
paused. The actual Server2019 technician upgraded from 1.5.4, opened its native
UI and passed MFA login with three approved devices and history visible; the
server audit independently matched that login. See the
[build/source receipt](../deployment/windows/e95010e-first-build-and-source-verification.json)
and [distribution/technician receipt](../deployment/windows/e95010e-lab-distribution-and-technician-upgrade.json).

The actual Server2022 customer also upgraded from 1.5.4 with identity, credential,
trust and revoked consent preserved. Its ordinary-user shortcut directly opens
the app and is not advertised. The first real ordinary-user stop test persisted
native pause without elevation; independent checks verified that the GUI token
has no Administrators SID, the same Windows service process remains running,
the configuration watcher remains running, and no new Swan MSI event occurred.
A normal kernel reboot then preserved pause, identity and revoked consent;
the service and watcher started automatically and the shipping verifier passed.
Normal restart/clean event-log shutdown were verified with no new unexpected-
shutdown event. The ordinary user reopened the actual shortcut and resumed
support without elevation; persisted state, the same post-boot service process,
all installed files and zero new MSI events verified. The consent dialog scrolled
to the complete policy with both buttons available; cancel preserved consent-off
and a fresh unattended grant was denied. See the
[ordinary-user controls evidence](../deployment/windows/e95010e-server2022-upgrade-and-standard-user-pause.json).
Active-session termination and real file transfer remain pending qualifications. This test also exposed an inherited
footer that incorrectly calls paused support a stopped Windows service and
offers a duplicate legacy start link. The next 1.5.6 source labels managed pause
correctly and leaves resume to the acknowledged company control. Its first
[native build passed](https://github.com/rijff24/SwanRemoteSupport/actions/runs/40628082360),
but installed visual verification is still pending. No complete
ordinary-user/remote-feature or clean Windows compatibility matrix is claimed.

The fresh Windows 11 Enterprise Evaluation 26300.9457 installed the actual
1.5.5 customer company bundle. All 96 signed files, company profile, enrollment,
automatic service/watcher startup, default consent-off and absence of Tailscale
verified. Company approval and negative pending-device, unassigned-technician
and unattended-without-consent checks passed. No native session is claimed.
The genuine ordinary-user technician setup failed closed before MSI launch:
publisher verification exceeded its 120-second budget. A separate read-only
ordinary-user diagnostic accepted the same MSI hash and Authenticode certificate;
cold PowerShell startup and hashing together exceeded two minutes, while the
actual signature query returned Valid promptly. The source increases only that
bounded verification budget to five minutes; all trust checks and timeout
cleanup remain. The new 1.5.7 candidate also includes the prior footer fix.
Windows/Linux component workflow 40647664923 passed for fix source
`07bb8d0b99e2be002a30e5d97eebe67a9c2794ad`; five local source-archive and six
installer-metadata tests also passed. Native build 40647688419 is still in
progress at the recorded checkpoint. Neither the installed change nor the
technician installation is qualified yet. The original timeout and guarded pre-MSI continuation are preserved separately;
customer identity, credential and consent remained unchanged. See the
[Windows 11 findings](../deployment/windows/e95010e-win11-fresh-customer-and-technician-trust-findings.json).

The maintained Linux transport now has two-run binary reproduction evidence:
independent successful CI jobs produced identical original `hbbs` and `hbbr`
bytes from identical project trees and all 140 matching archived source entries.
The tracked upstream/shared inputs, reviewed UDP patch and six public recipe
files were independently checked; OCI images and ZIP timestamps are outside
this comparison. A separate private runtime used those verified binaries,
checked embedded source, private-key permissions and TCP startup, then stopped
without changing the company. Before moving the owned company lab to that
runtime, a current 439,647,643-byte encrypted backup retained all 14 installers.
Its independent restore verified HTTPS/profile trust, administrator MFA, all
four devices and consent, release approvals, the complete TLS/configuration
files, service-account readability of all artifacts, matching current customer
and technician downloads, anonymous-technician rejection and withdrawn-package
denial. Only the two transport image references were then changed in the owned
Linux lab. Private/public keys, transport pin, management/HTTPS container IDs,
devices, consent and release approvals verified unchanged. Normal production
Compose pins are unchanged. This is not proof of native direct/relay sessions,
UDP traversal or public Internet operation through the new runtime; those
qualifications remain pending. See the
[maintained transport and current restore receipt](../deployment/linux/maintained-transport-and-current-restore-20261028.json).

The current Linux management, configuration agent and worker also matched
byte-for-byte across independent CI runs 40647659405 and 40647664923. Both
component source archives verified all 221 expected public files, with identical
Rust 1.90.0 compiler identities and no shipping-file normalization. The PR merge
and push commits have identical project trees; their distinct source labels are
retained. This comparison does not qualify endpoint runtime behavior or claim
identical ZIP metadata. See the
[Linux component comparison](../deployment/linux/07bb8d0-component-reproduction.json).

## Earlier source checks, 27 October

The staged customer update exposed another real compatibility failure: Windows
servicing replaced System32's RuntimeBroker, leaving the endpoint's generated
privacy-helper copy stale. All 96 signed project files remained unchanged, but
strict installed-payload verification correctly rejected that stale OS copy.
A guarded lab-only refresh from the current Windows file allowed the unchanged
installed verifier to pass and the automatic update to reach its helper. The
temporary diagnostic watcher action then triggered the helper's task-ownership
guard; its original native action was restored without launching another
installer. Existing durable recovery then completed the actual 1.5.1/sequence4
to 1.5.3/sequence6 upgrade. Independent checks verified all 96 project files,
publisher signatures, MSI registration, original enrollment credential, trust
and consent, restored native watcher/service, and recovery cleanup. The excluded
Server 2019 customer stayed on 1.5.1. Further lab rollout is paused.
The upgraded Windows 10 VM then passed one normal kernel reboot: its automatic
LocalSystem service and native watcher started without intervention, and all
96 signed files, MSI registration, trust, identity and consent verified again.

The 1.5.4 source adds narrowly scoped OS-cache maintenance under installation
and session exclusion. It retains project signature/hash checks, rejects links,
defers locked files and active recovery, and never takes its source from company
configuration. Linux/Windows component checks passed for a6670d6. Both complete
native runs 40520718494 and 40534403214 passed; comparison run 40549751275
verified identical original bytes for all eight shipping outputs, matching
compiler inputs and exact public source/bridges. The first full archive and
all public source objects were also verified locally. The company worker then
generated both lab-signed bundles. A real 20% rollout automatically upgraded
Windows 10 to 1.5.4/sequence7 without a manual installer, cache refresh or task
repair. All 96 signed files, MSI registration, publisher, enrollment and consent
verified; a normal kernel reboot started the service and watcher automatically.
With rollout paused, replacing only the generated helper with an older genuine
Microsoft RuntimeBroker exercised the installed native watcher: it restored the
current System32 bytes and timestamp automatically, while all 96 project files
and identity/trust/consent remained unchanged. The shipping verifier passed.
The Server2019 technician also upgraded to 1.5.4, opened its actual native UI,
and passed MFA login with two approved devices and history visible; its process
and loaded libraries matched the reproduced payload, with no new crash event.
These bounded checks leave fresh installs, the full OS/feature matrix and
production signing open. See
[the lab runtime evidence](../deployment/windows/a6670d6-lab-upgrade-cache-reboot-and-technician.json) and
[the original-byte 1.5.4 comparison](../deployment/windows/a6670d6-native-independent-comparison.json).

An actual 1.5.3 technician on Server 2019 connected to the upgraded Windows 10
customer after its visible Accept prompt. Desktop pixels and a remote pointer
selection were independently observed on the customer console. Revoking that
technician's group access closed the exact claimed server session and the native
connection required fresh company authorization. Original group access was
restored without reviving the old grant. This session does not qualify its
transport path, clipboard, transfer, UAC, reboot or multiple monitors. See
[the bounded update and session evidence](../deployment/windows/7bfce49-win10-automatic-recovery-and-native-revocation.json).

The same two 1.5.3 apps also demonstrated explicit unattended consent, native
desktop access without another Accept prompt, and remote Windows sign-in after
the owned guest was signed out. The customer then revoked consent from its own
UI: the company disabled consent and closed all five outstanding unattended
leases, the native connection required new authorization, and a fresh grant
request from an authorized technician with a valid proof key returned 403.
Consent was left disabled. Multiple reconnect records during the sign-in
transition require session-history cleanup review; this does not qualify
clipboard, transfer, UAC, multiple monitors or public transport. See
[the bounded unattended evidence](../deployment/windows/7bfce49-win10-unattended-signin-and-revocation.json).

**1.5.3 now passes installed native technician startup and MFA login on the
Server2019 lab endpoint.** The generated company bundle upgraded 1.5.2/sequence5
to 1.5.3/sequence6, preserving bootstrap, branding and the separate customer
identity/consent. Actual MSI registration, installed signatures and hashes were
verified independently. The visible native app showed both approved lab devices
and history, and its MFA login matched the server audit. Its running child and
loaded Rust/Flutter libraries matched the reproduced payload, with no new
application crash event. A path-only verifier initially rejected the native
launcher's equivalent `/./` path; canonical inspection passed while retaining
exact-byte and process-creation checks. The later Windows 10 session and normal
reboot are recorded above; other OS qualification remains pending. See
[the bounded startup and distribution evidence](../deployment/windows/7bfce49-lab-distribution-and-technician-startup.json).

Both native runs 40468079337 and 40489196703 passed. Comparison run 40499642535
verified identical original bytes for all eight shipping outputs, exact public
source and six bridge files per build, and matching recorded compiler inputs.
The first full archive and source were also verified locally; the runner-side
comparison avoided a second large host download. All seven new lab signatures
and original receipts were checked against the existing nonexportable guest key.
Public customer and protected technician bundle downloads passed. The lab
completed a 20% staged rollout to the eligible Windows 10 customer while Server
2019 was excluded, and is now paused. Closed-window checks passed on both
installed agents. Failed 1.5.2 downloads remain withdrawn. See
[the original-byte comparison](../deployment/windows/7bfce49-native-independent-comparison.json).

The first 1.5.3 workflow, run 40468079337 from pinned source `7bfce49`, now
passed its native DLL-search regression and headless service installation gates.
The later evidence above supersedes its then-pending archive, reproduction and
GUI checks; real customer reboot qualification remains pending. Both installed Server2019 agents
also honored paused updates: no installation was handed off, executable hashes
were unchanged, and enrollment, consent and trust were preserved. Separately,
the existing 1.5.1 Windows10 customer failed service auto-start after a full
restart and needed one manual service start. Its identity and lab signing key
survived. Windows10 was two hours behind externally checked host time; its clock
was corrected forward before further expiry-sensitive tests. No application
expiry checks were changed. See [the bounded pause and restart evidence](../deployment/windows/lab-restart-and-paused-updates-20261027.json).

A fresh encrypted Linux company backup and separate restore now preserve both
current enrolled devices, their approvals and revoked unattended consent,
administrator MFA, the active profile signing key, original HTTPS trust, and
release approvals including the withdrawn 1.5.2 downloads. The recovery
management and HTTPS containers were stopped, and the original company
remained healthy with signed profile revision 5 and updates paused. This does
not qualify Windows company restore or a native session against the restored
copy. The installed Server2019 customer also authenticated to the restored
company with its unchanged credential, which stayed inside the original guest.
Windows10 initially stalled at SSH command startup; after normal shutdown and
boot of the same disk, its unchanged credential also authenticated to the
restored company. The original failure is retained. No credentials left their
guests, and neither customer was reenrolled. The customer service was running
automatically after this recovery, but Windows retained its earlier kernel boot
time, so a full restart remains a separate test.
See [the current recovery checkpoint](../deployment/linux/pre-153-company-backup-restore-evidence.json).

**1.5.2 failed native technician startup and both sequence-5 lab releases were
withdrawn.** The generated installer upgraded the existing Server2019 technician
from 1.5.1 to 1.5.2, with registered MSI state 5, matching new hashes/signatures,
and preserved bootstrap, branding and customer identity/consent. A wrapper
postcheck path error and a later property-helper check were retained; direct
inspection verified the installation without repeating it. However, the actual
GUI then crashed with application events 1000/1001 and exception `0xc06d007e`.
The unpacked executable and required DLLs matched the reproduced build. The
outer portable launcher returned success despite the native child crash; that
exit status is not accepted as a startup pass.

Startup's hardened DLL search omitted the executable directory, so the newly
delayed desktop DLLs could not resolve after bootstrap. An actual isolated
Windows loader fixture reproduced error 126 with the old flags and loaded the
application-directory probe with corrected flags, while continuing to reject
CWD and PATH probes. The source now adds only application-directory search,
retains delay loading, and adds a real child-process native Windows regression.
Unreleased binaries advance to 1.5.3; its installed technician GUI retest now
passes as described above, while customer upgrade and reboot tests remain open.
See [the failed runtime and withdrawal evidence](../deployment/windows/9ba592c-technician-startup-failure.json).
Both generated 1.5.2 downloads now return 404. Updates remain paused; existing
baseline customer services and release evidence were preserved.

Native source `9ba592ca0431ce734a1edf7391aec183f5c100fa` built successfully in
run 40435586938. Its full corresponding source archive matched 1,203 pinned public
Git files and all six generated bridge files matched that run's retained bridge
artifact. See [exact-source evidence](../deployment/windows/9ba592c-native-source-evidence.json).
The technician helper rejection and actual unsigned customer MSI service-start
checks also passed; [their evidence](../deployment/windows/9ba592c-native-startup-ci-evidence.json)
states the limits. Independent runs 40435586938 and 40442895972 now produced
identical original bytes for all eight shipping EXE/MSI outputs and identical
618-file MSVC input proofs. Each source archive matched all 1,203 public Git files
and its own six retained generated bridge files. No shipping bytes were
normalized or executed for this comparison. See
[the independent comparison](../deployment/windows/9ba592c-native-independent-comparison.json).
These results do not qualify
installed technician startup, customer cold-boot recovery, automatic upgrades,
the complete Windows matrix or production signing. The source-specific runtime
and historical sections below retain the earlier results and failures.

The same verified source subsequently produced new lab-signed 1.5.2 customer
and technician MSIs, with signed application, agent, management and worker
executables. Both MSI builds used the pinned default recipes and 219 verified
inputs. All seven output signatures and transferred original hashes were
independently checked, including timestamp receipts and the new machine-local
key's nonexportable policy. The existing customer service remained running.
[Lab signing evidence](../deployment/windows/9ba592c-lab-signed-msi-evidence.json)
does not establish installation or update acceptance. The preserved Windows
company VM and its signing identity were untouched; these signatures do not
represent production signing approval.

The company then served the exact 1.5.2 MSI payloads, agent and full corresponding
source archive over HTTPS with matching hashes. Its existing worker verified the
new public lab publisher; identity, credential, consent and both running services
were preserved. Bad signatures and request-supplied trust overrides returned 403;
unapproved build requests returned 409 without creating jobs. Explicit approval
produced both company bundles: anonymous customer download returned 200,
anonymous technician download returned 401, and authenticated technician download
returned 200. Both envelopes, payload hashes and seven bundled source texts
verified. The deployed worker and installer recipe paths are unchanged in the
new source; the new worker binary has not been deployed. Signed policy fields
remain at revision 5 with updates paused; only normal signature lifetime renewal
changed the envelopes. See [distribution evidence](../deployment/windows/9ba592c-company-bundles-evidence.json).
No 1.5.2 installation, automatic rollout, session coordination or interruption
recovery is established by these distribution checks.

On a newly created standard Windows10 Pro 22H2 x64 VM, build 19045.3803, the
existing lab-signed da0c621 technician MSI passed administrative installation,
repair after retaining and removing its verified executable, and uninstall.
All three phases returned 0, installed executable and agent signatures matched
the explicit lab publisher, and 14 original records matched their guest hashes.
See [the bounded fresh Windows10 evidence](../deployment/windows/da0c621-win10-fresh-technician-msi-evidence.json).
The initial certificate-cmdlet and script-policy failures occurred before MSI;
the full administrator token was independently confirmed, exact public lab
trust was verified, and the successful test used a dedicated child process
without changing persistent execution policies. A timed-out evidence-helper
transfer was also retained; direct collection subsequently succeeded. These
results do not qualify standard-user installation, native technician login,
sessions, customer behavior, software upgrades or full Windows10 acceptance.

The independent Linux company subsequently generated a Windows10 customer bundle
from its existing signed 1.5.1 baseline. Anonymous download returned 200; both
signed envelopes, public-only bootstrap fields, payload hashes and seven bundled
source texts were verified. On the fresh Windows10 VM, that generated installer
created a unique enrolled identity, accepted profile revision 5 and release
sequence 4, retained unattended consent off, and started its automatic LocalSystem
service. Installed executable and agent hashes and publishers matched, no setup
recovery marker remained, and five original setup records matched their guest
hashes. No Tailscale service was present. The private laboratory DNS mapping and
public CA were provisioned separately, with HTTPS verification retained; this
does not prove public Internet hosting.

Before company approval, the new device was absent from authorized technician
inventory and its grant request returned 403. Explicit approval added it to the
authorized group inventory; unassigned access and unattended access without
consent remained denied. The original Server2019 device record was unchanged.
See [the bounded installation and enrollment evidence](../deployment/windows/da0c621-win10-company-customer-evidence.json).
The idle-MSI guard and its subsequent path-regex error were retained as preflight
failures before setup. No installer or service was terminated to proceed. These
results do not qualify a Windows10 native session, cold reboot, customer repair
or uninstall, desktop features, 1.5.2 upgrades or full Windows10 acceptance.

## Lab continuation, 26 October

The current da0c621 customer and technician lab-signed MSIs passed bounded
administrative lifecycle checks on a newly created Server2019 Standard Evaluation
Desktop Experience VM, build 17763.3650 x64. Customer install, repair and uninstall
returned 0; a real reboot preserved all 96 packaged files and valid publishers,
and the LocalSystem customer service started automatically. Technician install,
missing-executable repair and uninstall also returned 0 in its first application
context after the customer test. Thirty-one original records were collected and
hash-verified. See [the bounded evidence](../deployment/windows/da0c621-server19-fresh-msi-evidence.json).

The OS setup's lab bootstrap command had returned 1 and left the test tools
absent. Its original cause is unproven. The pinned bootstrap was run manually
after reaching the administrative console, lab access was verified, automatic
login disabled, and setup media detached before Swan testing. This is not proof
of unattended OS provisioning. These MSI checks do not establish enrollment,
sessions, standard-user installation, software-version upgrades or automatic
updates on Server2019; company-approved test metadata still targets Server2016.

The company VM's private lab HTTPS binding was subsequently enabled with its
rollback backup retained. Company HTTPS recovered and device identity and
consent were preserved. From the separate Server2019 VM, normal HTTPS validation
accepted the explicitly trusted public lab CA, the public profile returned 200,
its signature verified against the original configuration pin, and anonymous
device inventory returned 401. Profile 18 remains paused. See
[the bounded peer connectivity evidence](../deployment/windows/server19-company-peer-https-evidence.json).
No private TLS key was transferred. This removes the private HTTPS connectivity
block for subsequent tests, while public reachability remains unproven.

The saved lab transport profile still points to parallel-test ports with no
active parallel listeners; installed transport processes use their normal ports.
The maintained transport candidate remains separate from shipping component
pins. Current native enrollment and direct/relay sessions still require testing.
The single Windows10 standard lab account was subsequently reset by its
administrator. The new credential was accepted and the recorded old value
rejected; that saved old value had also failed before the reset. Account identity
and standard-user status were preserved, both credential records are protected,
and one-time guest and host inputs were removed. Permanent execution policies
were retained. See [the bounded repair evidence](../deployment/windows/win10-lab-account-repair-evidence.json).
This resolves the lab credential repair; it does not establish the cause of the
earlier guest hang or complete Windows10 product acceptance.

The additional dedicated lab drive removed the Windows11 storage shortage. The
large local source, media and VM files were moved with SHA256 content verification
of 49,822 files (327,044,501,570 bytes), preservation of access restrictions, and
runtime checks of the existing company services and signed paused profile 18.
Machine paths and private migration receipts remain outside this repository.
The frozen client distribution was not booted; its earlier unresolved integrity
question is not closed by verification of this move.

The Server2022 standard-user console was reached and the desktop test task
launched PowerShell in console session 1. It stalled before writing the first
driver observation or starting Windows Installer. The bounded attempt was
terminated and disabled; its process was confirmed absent and no msiexec process
was running. This is an inconclusive harness startup failure, not a passing MSI
test or an established product defect. Original event and final-state records
are retained privately. The earlier batch-context failure remains unresolved.

No application binary changed for these lab operations. All current release
gates below remain open except the shortage of lab storage.

## Current status, 24 October

The latest tested application source is `da0c621845057bed321a98a1886ef15891e431a9`.
Later evidence/documentation commits do not change those application binaries.
The product remains incomplete and is not approved for production publication.

- **Reproduction:** two independent da0c621 native builds produced identical
  original bytes for all eight shipping outputs and the same 618 recorded MSVC
  inputs. SDK/actual CMake command provenance and production signing remain
  separate gates. See [the comparison](../deployment/windows/native-da0c621-rebuild-comparison.json).
- **Distribution:** the existing company worker generated lab-signed customer
  sequence 14 and technician sequence 15 bundles. Public customer and protected
  technician downloads and independent content checks passed. The generated
  technician setup passed explicit sequence 13-to-15 upgrade in an administrative
  profile. That is not an automatic software-version upgrade. Company profile 18
  remains paused; approved test metadata targets Server2016 only. See
  [distribution evidence](../deployment/windows/da0c621-lab-distribution-evidence.json).
- **Fresh Server2022:** on 20348.587 x64 Desktop Experience, the current customer
  MSI passed administrative install, repair, a real reboot with automatic
  LocalSystem service startup, and uninstall. All 96 payload hashes and installed
  publishers passed. Technician administrative install/repair/uninstall also
  passed after customer testing on that VM. See
  [the bounded fresh-VM evidence](../deployment/windows/da0c621-server22-fresh-msi-evidence.json).
- **Standard-user technician:** the Server2022 batch-context test failed with
  MSI1601/Windows Installer connection denial 0x80070005. A non-admin token and
  separate profile were verified. Temporary batch-logon rights were restored and
  test tasks disabled. An actual desktop-context test is staged but has not run;
  console activation failed twice and awaits restored UI access. See
  [the original failure](../deployment/windows/da0c621-server22-standard-batch-failure.json).
- **Windows10:** bounded current MSI/reboot checks passed, but a later guest hang
  has no established cause. A legacy lab password disclosure still requires the
  separately requested reset and cleanup; automatic approval review blocked
  those actions. The affected VM remains offline. These are unresolved failures,
  not complete Windows10 acceptance. See
  [the bounded evidence and incident](../deployment/windows/da0c621-win10-bounded-evidence.json).

Current cross-machine enrollment/sessions and public direct/relay transport,
customer consent/revocation and desktop features, current branding/key-rotation
runtime checks, approved automatic rollout/interruption recovery, the remaining
clean-machine Windows matrix, full Windows TLS/transport backup/restore and
production signing remain release gates. Existing partial and older-source
results below do not close those requirements. Private company HTTPS binding
and full Windows restore approval remain pending; Windows11 awaits additional
storage. A read-only observation after the Server2022 tests confirmed the company
server, worker and original customer process still running and the public profile
endpoint returning HTTP200. No working-company rollout was unpaused for these tests.

## Historical status, 23 October

The preserved company VM now runs the lab-signed `f84ffb0` management service
and worker after a verified encrypted management-only backup and coordinated
upgrade. Identity, consent, HTTPS binding, transport trust, paused profile 18,
customer process/watcher and interrupted customer setup evidence were preserved.
The new worker's generated bundle installed the already-approved signed
`d8fad20` technician payload in a second administrative account while preserving
the first account: real MSI state 2 became state 5. Exact new installer texts,
signed profile/release and installed identity were independently checked.

The signed `d8fad20` customer MSI passed Server 2016 reinstall, damaged-license
repair and uninstall. A real standard-user desktop technician MSI attempt was
blocked by effective Windows Installer policy (1625); no policy or privileges
were relaxed. These are bounded installation results, not native session tests.
Independent `d8fad20` native builds reproduced all eight shipping files. New
`f84ffb0` component checks/source verification and both native builds passed.
Their independent comparison failed for the client packages; the server setup,
management, worker and agent matched. Client reproducibility remains a release
gate. The later `7928f87` generic Flutter feature-guard correction compiled
successfully for Windows and Linux x64; the broader workflow remains incomplete.

Current receiving-device/session features, separate-machine and public transport,
fresh matching Windows deployment, the complete Windows matrix, current automatic
rollout/recovery, full Windows TLS/transport restore and production signing remain
release gates. Private HTTPS binding and full Windows backup/restore approval
blocks remain unresolved. Management-only backups do not prove full restore.
Latest successful graphical sessions still use the earlier technician and older
receiver on one VM, as detailed below. No complete-product acceptance is claimed.

The following early-23-October summary and 22-October snapshot are historical;
the final dated sections contain subsequent source-specific evidence.

**Earlier Windows company-lab state:** after explicitly approved cold migration of
the existing company disk to KVM, all three services and the HTTPS API were
observed healthy. Original customer identity and consent, profile revision 18,
paused updates and the existing HTTPS binding were preserved. Both original
sequence-9 bundle jobs completed; public customer download, protected technician
download, file hashes and independent bundle-content verification passed.
The original bundles contain unsigned CI application payloads and are not complete
release packages. The later corrected technician bundle passed explicit setup
recovery in an administrator profile; customer installation and end-to-end support
acceptance remain open. Earlier guest unresponsiveness, Caddy faults and the separate endpoint-lab
storage hash discrepancy remain recorded below; recovery is not proof that their
causes have been resolved. No production or full compatibility acceptance is claimed.

The subsequent generated-technician installation test reached final verification
and correctly rejected the unsigned application inside the lab-signed MSI.
Its release sequence stayed zero until a corrected sequence-10 package, built
from signed payloads, completed explicit recovery. The old pending marker was
archived unchanged, installed signatures/hashes passed, and sequence 10 was
recorded. This is an existing Server 2016 administrator-profile recovery result,
not fresh-machine, standard-user, graphical-session or automatic-update acceptance.

The correctly signed customer sequence-11 bundle also passed generation, public
download and independent content verification. Its in-place installation was
correctly refused because the existing lab MSI is registered as 1.5.7 although
its application binary reports 1.5.0. The original application, identity and
consent remain intact; configuration syncing was restored after the failed setup.
A pending sequence-11 setup marker remains preserved. A source fix now checks
related MSI versions earlier and restores the previous configuration-task state
on later failures. Targeted Windows PowerShell checks pass; matching rebuilt
binaries and fresh-client qualification are still required.

The latest accepted-desktop lab run used technician source `e6b87b9` against the
preserved older `bd13500` customer on Server 2016 with the parallel transport
candidate and experimental `9f97ef0` source-built engine. Standard-user MFA, inventory/history, customer approval, desktop
capture, claimed/closed server history and logout passed. Both editions were on
the same guest: this does not verify a corrected receiver, separate-machine
input, relay fallback or Internet connectivity. An earlier relay attempt selected
a local direct connection and the guest later became unresponsive; it recovered
with identity and consent preserved, but that earlier failure's cause is unproven.
Two runtime configuration fixes are in native build `39600988305`; its build
and artifact/source checks passed, but complete corrected-customer runtime
acceptance remains open. Subsequently, technician source `c0bf4b4` passed
standard-user graphical MFA, inventory/history, customer refusal, live name sync
and offline cached-branding checks against the existing lab service. That does
not establish an accepted desktop session with both corrected editions. See
the final dated sections for exact scope and failures.
Earlier entries below retain their original source-specific observations;
later evidence does not retroactively validate older packages or every Windows
version. The full release gates remain open.

## Earlier release-gate snapshot, 22 October

The dated evidence below is source-specific. Earlier passing packages do not
qualify the current candidate. Native source `ebef5a6` passed workflow
`39704802586`, including the managed-policy and view-only control regression
tests and the unsigned technician MSI lifecycle under the CI account. Its
downloaded archive, 1,122 tracked source files, six generated bridge files and
95 native payload files passed independent provenance/hash checks. Its
authorization-log and control-permission corrections still need receiving-device
runtime verification. The earlier `becf3cd` standard-user tampered-cache GUI check
confirms the legacy upgrade card is absent. Its separate standard-user native
`--update` run logged the managed-update refusal and preserved the installed
customer/profile; GUI installation callbacks remain to verify at runtime.
This does not prove binary reproducibility.

The application still pins the reviewed upstream Windows Flutter engine by
archive hash, whose source-to-binary mapping remains unverified. A separate
source-built candidate completed CI at `9f97ef0` and passed artifact, public
source-tree and generated-command verification. Its isolated test with the
older app bundle failed before GUI startup because the snapshot formats differed.
Experimental build `39730064190` recompiles that bundle with a matching compiler;
build `39730660383` additionally checks the new recorded UTC build-date input.
Both workflows completed successfully, including native policy/control
regressions, package creation and the CI-account technician MSI lifecycle.
The latter also passed the deterministic build-metadata regressions. Its
original artifact `12406886407` passed independent archive, 1,138 tracked source,
six generated bridge and pinned engine-evidence verification. All 95 extracted
payload files matched the installed-file manifest. Its source-specific GUI
smoke test passed MFA, inventory/history, customer approval, desktop capture,
session close and logout against the preserved older receiver on the same VM;
this does not qualify a corrected receiver or the complete runtime matrix.
An independent application build, `39741804606`, uses the exact same
`e6b87b91cbdb413a64a306add7df733aa31a9795` commit and experimental engine
selection through the pinned `codex/rebuild-native-e6b87b9` branch. Original
application/installer output comparison remains pending.
The timestamp-controlled engine build at `b1d62b2` completed CI and passed
artifact/source/command/link-response and PE metadata verification. Its
independent same-recipe build is
`39730890327`, whose original artifact `12407033745` passed source/evidence
verification. The original DLL and import library are byte-identical across
the two builds. Their original PDBs differ, so complete engine-output
reproducibility remains open. The successful GUI test used the earlier
`9f97ef0` engine, not this timestamp-controlled DLL. Broader runtime
qualification remains open. The public upstream
source/patch reconstruction candidate and evidence limits are recorded in
[Source availability and licensing](SOURCE_AND_LICENSE.md#patched-windows-flutter-engine-provenance).

| Area | Latest established evidence | Still required for release |
| --- | --- | --- |
| Company setup | First-run branding/policy form contract and real Axum/SQLite setup tests pass on Windows/Linux; Linux deployment rehearsals pass. | Matching fresh Windows service/worker setup and equivalent full deployment acceptance. |
| Native technician | `e6b87b9` with the experimental source-built engine passed standard-user MFA, authorized inventory/history, customer-approved desktop capture, session close and logout against the older customer on the same VM. Earlier `c0bf4b4` also passed live branding and offline refusal/cache. | Accepted support session with both corrected editions and a separate receiving machine. |
| Receiving authorization | Older `bd13500` customer rejected a real stock client's legacy-password request. Native `c0bf4b4` and current `becf3cd` GUIs refused a tampered cached profile; the current UI also hides legacy upgrade cards. | Corrected receiver runtime, unauthorized direct connections, packet permissions, lease expiry and revocation. |
| Customer session features | Current technician/older customer same-VM approved desktop session; earlier graphical refusal. | Corrected receiver, separate-machine input, transfer, clipboard, UAC, logon screen, multiple monitors, unattended consent/revocation and relay fallback. |
| Installers | Source-specific administrator MSI lifecycle evidence is listed in the Windows matrix below. Current `becf3cd` technician MSI also passed standard-user install, missing-file repair and uninstall as a functional diagnostic on an expired Windows 10 evaluation. | Current-source company-generated EXE/MSI flow, standard-user qualification on valid Windows installations and the complete upgrade/recovery matrix. |
| Updates and recovery | Earlier signed lab update evidence, component rejection tests and real MSI rollback fixtures. | Current-source approved automatic rollout, session coordination, interrupted-update recovery and identity/consent preservation. MSI rollback alone does not prove these. |
| Public transport | Maintained transport candidates built; two Linux transport builds reproduced identical binaries with verified public source. | External direct/relay/negative-access tests without Tailscale and accepted production transport pins. |
| Publication | Corresponding-source/artifact verification and private lab signing are available. Management, agent and worker binaries matched across two independent jobs on each of Windows Server 2022 CI and Ubuntu 24.04 after the Windows linker correction. | Native app/engine and installer reproducibility, broader toolchain qualification, production signing approval/signatures and every release gate above. |

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

### Configuration-recovery native build verified (2026-10-21)

Native Windows run `39600988305` completed successfully for exact public source
`bd1350012227f226db168be51bc823d3f086e493`, containing both configuration-refresh
fixes. Artifact `12349480599` was downloaded once and verified against its
authenticated archive digest: 141,973,242 bytes, SHA-256
`31bf6a8905de084dfe837afa5efaafc87f1e14a196b3d7acbace2e6fd5cc6c0b`.
Its 24 entries passed bounded extraction, path and published checksum checks.
The corresponding-source archive matched 1,111 public Git files and all six
generated bridge files. The bridge input was independently fetched as artifact
`12344994676`, SHA-256
`d979023c7c01202fb93470a899cc2fdf7bf1d7825fd7151cb505a1ac6143930f`.
Six server-setup source inputs and the existing component pins also matched.
The build receipt still marks the checkout dirty; these source comparisons do
not establish a clean build or binary reproducibility.

Retained MSI lifecycle artifact `12349770219`, SHA-256
`b1313134dafac51a8758a1958861b374dc5113c11dd59534a93fb78acce21246`,
passed authenticated digest and ZIP checks. Its result records successful
installation, repair and uninstall on Server 2022 Datacenter build 20348.
The actual tested technician MSI, portable payload and agent hashes matched
the main archive. This is unsigned payload lifecycle under the CI account,
not company enrollment, standard-user GUI, signed upgrades, native sessions
or a complete Server 2022 acceptance pass. The configuration fixes' native
runtime recovery tests remain pending.

### Clean Server 2025 MSI lifecycle and Server 2016 recovery (2026-10-21)

A fresh official Windows Server 2025 Standard Evaluation Desktop Experience
guest completed installation as x64 build 26100.32230. Its independent guest
identity check passed. Provisioning disabled automatic sign-in and removed its
stored automatic sign-in password. A lab SSH-key ACL error was diagnosed and
corrected before product testing; Windows-provided SSH then passed the pinned
host-key check. Setup used 4 GiB RAM; the controlled runtime boot used 3 GiB,
retaining the host's 1.5 GiB reserve. Setup media was detached for that boot.
A clean pre-product disk checkpoint passed an offline qcow2 integrity check.

The independently verified `bd13500` customer and technician MSIs were staged
with exact-source test helpers and verified again inside the guest. All six
actual Windows Installer phases passed: install, repair and uninstall for each
edition, each returning 0 without requesting a reboot. Installed package and
agent hashes, MSI registration and technician shortcuts were checked. Customer
checks additionally verified its complete installed-files manifest and running
LocalSystem service. Customer repair replaced a deliberately damaged packaged
license file with its expected bytes. Uninstall checks verified removal of the
packaged files, relevant registration and customer service.

The six results and raw MSI logs were exported over pinned SSH to protected
host storage. Independent verification matched source revision, package and
agent hashes, the exact test-runner hash and OS build; every log contained the
zero-return completion marker without `Return value 3`. This establishes bare
unsigned MSI lifecycle as Administrator on this particular Server 2025 build.
Standard-user installation, company enrollment/branding, publisher acceptance,
upgrades and native sessions on Server 2025 remain unverified. It is not a
complete compatibility-matrix pass or production release approval.

The previously hung Server 2016 guest was powered off after its normal shutdown
request did not complete. This QEMU backend rejected memory snapshots. A full
stopped-disk copy was therefore retained privately and verified against the
original SHA-256; the original disk then passed its qcow2 integrity check.
After boot, device identity/token, consent and release sequence matched the
pre-reboot state, profile revision 16 remained accepted, and the customer,
management and worker services were Running. Management HTTPS and configured
status passed. The manually started candidate transport remained stopped as
designed. The event log recorded an unclean shutdown; it did not establish
the cause of the earlier unresponsiveness. This is crash-recovery evidence,
not a complete encrypted deployment-restore or migration rehearsal.

The following table preserves an earlier checkpoint. Use the current release
gates above and the later dated evidence for current status.

| Requirement | Historical checkpoint evidence and remaining work |
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

All rows remain intended release targets, not full compatibility claims. Passes
below cover only the stated source, package, privileges and OS build. Later
dated sections record package/media hashes and original failures; latest-source
full acceptance remains open on every row.

| Operating system | Clean installation | Repair/upgrade/recovery | Native session behavior |
| --- | --- | --- | --- |
| Windows 10 x64 | Standard Pro 19045.3803: administrator and standard-user technician tests, plus administrator customer MSI tests at `3bad169`; unactivated, license status 5, zero grace. Earlier LTSC evidence remains separately scoped. | Both accounts pass real technician 1.5.0-to-1.5.1 upgrade, missing-executable repair and uninstall. Customer install succeeds; repair returns 3010, actual reboot restores verified payload/service, then uninstall succeeds. Automatic updates and full qualification remain open. | Unverified. |
| Windows 11 x64 | Fresh activated Enterprise Evaluation 26300.9457: both unsigned MSI 1.5.0 editions from `9e4ea6d` install as Administrator; Secure Boot and TPM 2.0 verified. | Nine install/repair/uninstall operations return 0 across administrator customer/technician and standard-user technician tests; customer license repair and technician missing-executable repair pass. Upgrade and interruption tests remain open. | Unverified. |
| Server 2016 Desktop Experience | Existing 14393.693 customer deployment retained; fresh current-source company/customer setup unverified. | Administrative technician 1.5.0-to-1.5.1 upgrade, repair and uninstall at `699c9e0` pass; standard-user install returns policy error 1625. Full lifecycle/recovery remains open. | Older same-guest accepted session and `bd13500` receiver legacy-password denial. Later GUI/MFA/inventory checks include `699c9e0` technician-role API authorization and `3bad169` history-label regression. Separate-machine/full-feature acceptance open. |
| Server 2019 Desktop Experience | Latest `4c5accb` technician MSI installs in a fresh administrator profile on retained 17763.3650. Preserved insecure customer state is not replaced for a positive install. | Technician real 1.5.0-to-1.5.1 upgrade, missing-file repair and uninstall pass. Latest customer MSI rejects original insecure state through its deferred preparation action before payload copy; positive current-source clean customer lifecycle remains open. Historical signed fixtures remain separately scoped. | Unverified. |
| Server 2022 Desktop Experience | Both original unsigned `4c5accb` MSI editions install as Administrator on retained 20348.587. | Seven operations return 0: technician install/real 1.5.0-to-1.5.1 upgrade/repair/uninstall and customer install/repair/uninstall. Prior standard-user policy rejection and automatic update interruption remain open. | Unverified. |
| Server 2025 Desktop Experience | Both original unsigned `4c5accb` MSI editions install as Administrator on retained 26100.32230. | Seven analogous lifecycle/major-upgrade operations return 0. Prior standard-user technician policy rejection remains separately recorded; automatic update and full recovery qualification remain open. | Unverified. |

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

A second real generation test used matching `6bd54c2` native management, worker,
technician executable and configuration agent. The technician executable was
lab-signed with verified timestamp, SHA-256
`cb1d7cfcf4d429a1ecf59d9e5018e2686c40f04c477a420bda7e4106524461da`.
Signed test release sequence 8 named the exact public source and matching signed
agent. The isolated administrator imported/approved it and queued job
`de8d3b0a-e3ab-42ce-a322-4ed6c686b6e0`; the worker completed the ten-file bundle
with SHA-256 `345920e00869e3a6488c4867a16c25c493ee904db9cd3a71189d7da38b013579`.
Unauthenticated download returned 401; authenticated download, hash verification
and company/edition bootstrap checks passed. Owned test processes were stopped
and existing service statuses checked. This removes the earlier technician
payload source mismatch for generation evidence. Standard-user installation,
technician login and real sessions remain unverified for this new bundle.

The generated current technician bundle then passed portable installation through
its actual `Install-Company.ps1` as a real nonadministrator on Server 2016. The
test explicitly isolated LOCALAPPDATA/APPDATA beneath the user's disposable test
directory to preserve the existing technician setup; the previous Start menu
shortcut was restored afterward. The company domain confirmation was supplied
explicitly. Signed configuration bootstrap, package verification, installed
technician and agent hashes, `verify-installed`, online signed-profile sync,
company/edition identity, release sequence 8 and cleared installation marker
checks passed. The technician state contained no customer device enrollment or
unattended consent. The isolated management/proxy processes were stopped after
the user result, with existing service statuses unchanged. This is standard-user
portable installation evidence using isolated application data, not default-path
MSI installation, rendered branding, graphical MFA login or an actual session.

The installed `6bd54c2` technician agent passed real standard-user CLI
authentication against the isolated HTTPS management server. An administrator
created a technician role with a private authenticator secret. MFA login,
empty fresh-company device inventory and session history succeeded. Reusing the
same MFA code was rejected; the technician token could not create a worker;
logout revoked subsequent inventory access. The managed-state hash remained
unchanged and contained no returned session token. Test credentials were held
outside Git, removed after use, and cleared from the child process environment.
Owned proxy/server processes were stopped after the result. This proves the
installed agent's authentication API path only: graphical MFA login, nonempty
authorized inventory, device selection and native remote sessions remain open.

With the isolated HTTPS listener confirmed stopped, the installed `6bd54c2`
technician agent's `verify-installed` succeeded as the standard user using its
cached signed company profile. The managed-state hash was unchanged. This is
offline agent/installed-identity evidence; rendered offline branding remains
untested. Desktop-control capability was revalidated against the existing
disposable QEMU viewer and its Windows lock screen observed. No graphical
technician login or native session was established. Host free memory remained
approximately 2.5 GiB, below the second-VM launch guard.

Graphical test preparation encountered a desktop-tool limitation: the existing
TigerVNC lab viewer accepted mouse/keyboard input and exposed the VM lock screen,
but its menu key did not expose the guest secure-attention command. A second
viewer connection dialog exposed its owned options dialog to observation; both
coordinate input and subsequent Escape input failed with `failed to activate
captured window`. No guest login, credential entry or technician UI test was
completed. The existing VM/services remained running. This is a UI-tool blocker
for that viewer path, not evidence of a Swan application failure or session pass.

The installed signed MSVC `6bd54c2` technician agent passed a positive cached
profile/bootstrap control and rejected eight disposable altered copies:
invalid profile signature, modified signed payload, accepted-revision rollback,
wrong installer company, edition, management domain, profile trust pin, and
changed cached profile trust pin. Tests ran as the standard user through actual
`verify-bootstrap`; JSON was written without a BOM so parser failures did not
stand in for trust checks. The original installed managed-state hash remained
unchanged. This is native cached-profile/bootstrap rejection evidence, not
expired signed-profile, live endpoint rotation, package or session-grant testing.

Installed signed native `6bd54c2` technician and matching management passed live
company branding and profile signing-key rotation over isolated trusted HTTPS.
The administrator changed technician display name/color; the installed agent
synchronized them with a higher revision and unchanged trust pin. With only the
owned test management process stopped, native `prepare-profile-key` published a
transition signed by the old key. The client synchronized the authorized next
key while retaining its current pin. Native activation after that sync switched
the server key; the next client sync committed the replacement pin and a higher
revision. Company/edition, release trust, endpoint, consent, device fields and
installed release sequence 8 remained unchanged. After both isolated listeners
stopped, installed identity verification succeeded offline and the old bundle's
bootstrap pin was rejected. Existing service statuses were checked unchanged.
The first harness attempt stopped before rotation because the optional empty
support-contact field was omitted from serialized JSON and PowerShell attempted
to assign a missing property. Source inspection confirms `6bd54c2` supports that
field; the corrected harness tested display-name/color fields. This does not
establish rendered branding, contact-field sync, customer key rotation or a session.

The installed native `6bd54c2` technician subsequently passed signed management
endpoint migration between two guest-only HTTPS listeners on the same lab DNS
name. It learned the new URL through the existing signed profile, committed a
higher revision, and synchronized again after the old listener was retired.
Company/edition, both trust pins, installed release sequence 8, device fields
and consent remained unchanged. Offline installed-identity verification then
succeeded with the new endpoint persisted. Existing service statuses were
checked unchanged. This verifies HTTPS URL/port migration, not DNS hostname
migration, transport endpoint migration or a native session.

Native CI run `37095431510` completed successfully for exact source
`0a541c78609e8e3438e3c577abb352d714812b18`, covering implementation changes after
the earlier verified native build. All four jobs, including the branded Windows
x64 build, passed. Its outputs remain unverified until download, provenance and
artifact checks complete; build success alone does not establish native runtime
acceptance. Subsequent verification and runtime evidence is recorded below.

The Windows workflow now includes pinned transport/HTTPS archive preparation,
unsigned company-server setup compilation, source-manifest publication and its
SHA-256 checksum alongside endpoint installers. YAML parsing and both upload
entries passed local validation. The setup builder was exercised under Windows
PowerShell 5 using verified `6bd54c2` native management and existing pinned
components: all eleven embedded resource hashes passed, and the unsigned
64,093,696-byte package SHA-256 is
`14c2c794de937a20b16dc52f37b6238968c3fcaa6c3378fa80b5287838a0cb93`.
Its source receipt records recipe revision `4071147` and the dirty working tree;
it is a local test package, not a corresponding-source production release.
The already-running `0a541c7` CI build predates this workflow addition; a subsequent
full run must verify the new server packaging step. At that stage no setup
executable was run on the host; no host service, firewall, DNS or trust setting
was changed.

Installed native `6bd54c2` technician then passed live signed synchronization of
support contacts, a validated HTTPS shortcut, logo SVG and consent text from the
isolated administrator profile API. The harness explicitly added omitted
optional JSON properties before publication. The client committed a higher
profile revision while preserving bootstrap ownership/trust, release sequence 8
and consent. After the isolated management/proxy listeners stopped, offline
installed-identity verification accepted the signed cache without changing its
hash. Existing service statuses were checked unchanged. This is installed-agent
metadata synchronization evidence, not visual rendering or shortcut interaction.

The same hash-verified unsigned server setup package subsequently passed its
explicit `--render-preview` path on the unelevated Windows 11 host. The packaged
launcher extracted its bundled native management/components, loaded its embedded
wizard and exited successfully. Visual inspection of the generated 836-by-779
PNG confirmed Swan branding, the unsigned-test title, hostname and trust inputs,
bundled component paths and the preview-only status without clipped labels or
overlapping controls. The PNG SHA-256 is
`ca0e06e43bba6c841076015b667671895923cb252ba518f8c39ca42672883f53`.
The owned temporary payload directory was removed and no setup process remained.
Preview code disables installation and browser actions and does not show the
interactive form. This verifies packaged extraction and offscreen rendering,
not elevated installation, interactive setup, production signing or a supported
Windows compatibility result. No host service, firewall, DNS or trust was changed.

The hash-verified native `6bd54c2` configuration agent also passed
`deployment/windows/test-native-profile-time.py` on the Windows 11 host using
four isolated synthetic-company cached profiles. An ephemeral Ed25519 key signed
each profile and Python independently verified each signature before invoking
the actual native `verify-bootstrap` command. The valid control passed; expired,
future-issued and invalid-lifetime profiles were rejected with the profile-time
validation error, and all fixture state files remained byte-identical. Agent
SHA-256 was
`870ead5b2307c289d3e94542b53756488ea0ae6e2a558ed4131e9e49e6eaa9ee`.
The key was never serialized, the synthetic HTTPS address was never contacted,
and installed company state was not used. This proves native cached-profile time
rejection, not live expiry during a remote session or refresh of an expired cache.

Native `6bd54c2` portable technician recovery was exercised in a separate
standard-user-owned directory on Server 2016 using private lab-signed binaries.
The harness cloned the sequence-8 installation, prepared verified signed
rollback snapshots and a synthetic sequence-9 `rolling_back` receipt, then
damaged only its disposable canonical endpoint and agent. The signed staged
helper restored both exact binaries, removed the pending receipt and recorded
`failed-update.json` with phase `rolled_back`, failed sequence 9 and previous
sequence 8. An independent read-only verifier confirmed both restored hashes,
byte-identical company state and installed metadata against the source
installation, retained sequence 8, unchanged empty technician device fields and
disabled unattended consent. The native `verify-installed` command accepted the
restored installation and its pinned publisher.

The original SSH harness has not returned a terminal result; its post-recovery
app restart and cleanup command remain unresolved. Therefore this is verified
post-recovery state for a prepared rollback, not a complete automatic-update
failure/rollback test, replay-rejection test, GUI acceptance, MSI recovery or
power-loss evidence. The synthetic receipt SHA-256 is
`f3e7059dfd47309c406b2bbe0e988ee697bbc8cd8d8ed92bd3ba8aab1b2ce126`;
fixture files and signing material remain outside Git.

The restored Server 2016 technician fixture subsequently passed native
`verify-package` quarantine checks. Correctly signed metadata for failed sequence
9 was rejected specifically as quarantined; correctly signed sequence-10
metadata for the same trusted lab package passed signature, hash and publisher
verification as a positive control. Company state, installed metadata and the
quarantine receipt remained byte-identical before/after both commands. No
installer was executed and no sequence advanced. This proves native explicit
package verification enforces the recorded failed-release watermark; automatic
discovery, rollout of a later release and the original harness terminal result
remain unverified.

Using the same sequence-10 metadata and restored native agent, a Server 2016
negative package check flipped one byte at offset 4096 in a disposable copy of
the real lab-signed technician EXE. Native `verify-package` rejected the damaged
copy with a hash-validation error. The preceding intact-package positive control
had passed. The original package, company state, installed metadata and rollback
quarantine stayed unchanged, and no installer was executed. This verifies native
package-byte tamper rejection; it does not establish interrupted installation or
automatic-update acceptance.

Native run `37095431510` artifacts for exact source
`0a541c78609e8e3438e3c577abb352d714812b18` subsequently passed authenticated
GitHub provenance checks, the authoritative archive SHA-256
`1e77ecd393173b0a10cababc01b39d0d28bc177c5fbe8a5406bfc1148b17229b`,
bounded flat-file extraction, ZIP CRC checks and published payload checksums.
Twenty files were verified outside Git. These remain unsigned build artifacts.

The verified native MSVC management binary (SHA-256
`9046b27c9e7b6afdb581ee84a2c4d8a3c1128b34a138daf354cbd3603c2ab3e6`)
then passed a fresh isolated Server 2016 company setup and MFA login, worker
creation and enabled claim, administrator-only credential-free worker listing,
revocation, subsequent claim/upload authentication denial, persisted disabled
status and revocation auditing. The harness used guest-loopback HTTP and stopped
only its owned management process; existing service statuses stayed unchanged.
This verifies compiled native revocation behavior, not HTTPS/public transport,
signed distribution, job cancellation during active upload or a real session.

The verified `0a541c7` native management, agent and worker inputs were then signed
inside the Server 2016 guest using the existing non-exportable private lab
certificate and pinned company-signing recipe. Every original input hash was
rechecked, Authenticode status and certificate identity passed, and
`signtool verify /pa /all /tw` confirmed RFC3161 timestamps without warnings.
Signed hashes are management
`bf52a32557e96990616de28380c386b4eae7ece0b523f9677685e8f6c359c8bc`,
agent `1226def2eee9ade11d4fabd5b49321de5de4744973999fd517c85645086f0ca0`
and worker `6610f507d02ab4f02dc1080142f113f0c2aab7ec0b10564573216dcb861cad51`.
The key was not exported; host trust and installed service binaries were not
changed. These are lab fixtures, not production-signed releases or evidence that
the new worker has completed a service-mode build.

Matching lab-signed `0a541c7` native management and foreground worker then passed
real customer and technician bundle generation over the isolated guest HTTPS
endpoint. The administrator imported/approved earlier signed lab payloads from
`4768d53` and `6840d91` and queued both jobs. Customer job
`2d2fd057-850f-4d56-9340-8d51644d3cf1` completed with bundle SHA-256
`fd374569c890dc48aedcabd37d44f1b70f64e5d3b197e84e5d9b5db71b71522e`;
technician job `48303891-6097-4dbc-b09a-3e2b6cedbab1` completed with
`b7bbd3a197c2aba4c18e61645d9497bddc711f5009070667f478cf44d2cee648`.
Both downloaded hashes, ten-entry bundle recipes and company HTTPS URL/profile
pin/edition bootstraps matched. Customer download was public; technician download
required authentication and the unauthenticated request returned 401. Owned test
processes were stopped and existing service statuses were preserved. No bundle
was installed. This verifies current native server/worker generation using earlier
payloads, not newest-client installation, worker service-mode completion, public
Internet connectivity or production signing.

Matching `0a541c7` technician distribution then passed using the newly lab-signed
technician EXE and matching agent, signed release metadata sequence 11, and the
same native management/worker pair. Technician EXE SHA-256 is
`fe29807746ec9f406031f10f5872083b13e1374a7fa2f35b0d077a8221fb8722`;
its signature, pinned lab certificate and RFC3161 timestamp were verified in the
guest. Job `456c3b0a-dfa4-48ad-8cf7-1ae24024abef` produced bundle
`64d7b7b3119ec794b08d1cee8e205f2ed81b687062b1490c19883a8caedec019`.
Authenticated download, hash, ten-entry recipe and HTTPS/profile-pin/edition
bootstrap checks passed; unauthenticated download returned 401. This package
contains matching current native payloads, but was not installed in this test.
Production signing, service-mode completion and rendered technician/session
behavior remain open.

The matching `0a541c7` technician bundle subsequently passed its actual
`Install-Company.ps1` setup under the non-administrator `swanlabtech` account on
Server 2016. Separate fresh application-data directories preserved the earlier
installation. Installed EXE and agent hashes matched the approved signed inputs,
native installed-identity verification and HTTPS signed configuration sync passed,
sequence 11 was recorded and the pending-install marker cleared. No customer
device identity, enrollment token or unattended consent was created. The existing
Start-menu shortcut was restored, owned management/proxy processes stopped, and
existing service statuses remained unchanged. This is real portable technician
installation evidence; rendered login, authorized inventory, native sessions,
MSI installation and the remaining Windows matrix are still unverified.

Installed `0a541c7` technician CLI authentication then passed under the same
standard user against the matching native HTTPS management fixture. A new local
technician account completed MFA login; inventory/history requests succeeded
with empty results, replay of the consumed MFA code was denied, and worker
administration privileges were denied. Logout invalidated subsequent inventory
access. Managed identity stayed unchanged and no login token was stored in managed
state. Temporary account credentials were removed by the harness. This verifies
installed-agent authentication and role enforcement, not rendered login, populated
authorized inventory or a native remote session.

The installed `0a541c7` technician also passed standard-user offline
`verify-installed`: a loopback TCP probe confirmed the isolated company endpoint
refused connections, the native agent accepted its cached profile and signed
installed release/publisher, and managed state stayed byte-identical. The first
harness attempt used an administrator-only CIM query and failed before native
verification; replacing it with the unprivileged connection-refused probe
preserved the offline precondition. This does not authorize offline remote access.

The VM viewer became targetable again after refreshing and activating its
returned window. Its screenshot showed the disposable Server 2016 desktop is
locked. Graphical tests await manual unlock under the computer-use skill; no
authentication dialog or security setting was automated.

Standard-user portable repair passed for the installed `0a541c7` technician on
Server 2016. After preserving a hash-verified backup, the harness replaced only
the disposable installed EXE with four invalid bytes and ran the actual bundle
`Install-Company.ps1 -Repair`. The signed executable was restored to its exact
expected hash, native installed-identity verification passed, managed state stayed
byte-identical at sequence 11, and the pending-install marker cleared. The prior
Start-menu shortcut was restored. This proves damaged portable EXE repair, not
MSI repair, agent-loss repair or remote-session behavior.

A separate standard-user test then damaged only the installed `0a541c7`
configuration agent with four invalid bytes and ran the same trusted portable
bundle repair. The bundle's staged signed agent completed repair, native
installed-identity verification passed, managed state stayed byte-identical at
sequence 11 and the pending-install marker cleared. Protected fixture backups
and the previous Start-menu shortcut were retained/restored. This establishes
explicit portable-bundle repair of a corrupt agent, not automatic recovery from
a missing canonical agent or MSI/service repair.

The portable technician uninstaller added in `e9cda62` passed as a standard user
on the disposable Server 2016 machine using the existing signed `0a541c7`
installation. `-WhatIf` left files and recovery state unchanged. Confirmed removal
cancelled update recovery and removed the fixed application files while retaining
company state. Reinstall through the trusted existing bundle restored the payload,
passed native installed-identity verification, cleared the uninstall marker and
preserved managed state byte-for-byte. This verifies the new source script with
an older ten-entry bundle; generation and installation of the new eleven-entry
recipe, graphical uninstall confirmation, MSI removal and complete cleanup remain
unverified. The matching native build for source `e9cda62` has been dispatched.

The lab-signed native `0a541c7` management server passed an encrypted backup and
restore rehearsal in separate protected Server 2016 directories. Signing/setup
file hashes and the complete signed profile remained identical. Original and
replacement guest-loopback servers returned identical account, device, worker
and approved-release state; MFA login and replay denial passed independently.
Backup overwrite and restore into a nonempty directory were denied. Wrong-password
and byte-tampered archives failed authentication before creating target files.
Only owned foreground test processes were stopped; existing services retained
their prior status. The first harness expected the wrong authentication error text
and stopped; the corrected fresh-directory rehearsal passed. This is management-only
native recovery evidence, not transport/TLS restoration or migration rollback.

Deployment backup now has an explicit `--installer-artifacts-directory` option.
It includes canonical build-UUID ZIP files in the authenticated archive and
restores them under the replacement data directory, retaining files needed by
published installer links. Four compiled backup tests passed, including byte-exact
installer-file recovery, rejection of unfinished uploads, tampered archives,
unsafe restore paths and prior transport/TLS behavior. This component evidence
does not demonstrate native installer-download availability after deployment
restore; that check requires a rebuilt server with this option.

That installer-download check subsequently passed with native Windows source
`b0cb74045487003b2ba179f210ebfcc700e7b537`, built with locked dependencies using
Rust 1.90.0 GNU. Unsigned management SHA-256 was
`fbaaa1c0611734b42e8362c323fe7bdab241f294e6b89ddf5b690bd92705048a`;
the lab-signed fixture was
`645d17ab2dac1fad4789ee629cc23353d6e1af2ccd219bd959bdb7fc8c1953a6`
and passed Authenticode and required RFC3161 timestamp verification. Encrypted
export/restore of the isolated company's database and generated installer files
produced a replacement Server 2016 server that served the customer bundle publicly,
denied unauthenticated downloads of both technician bundles with 401, and served
both after MFA authentication. All three downloaded hashes matched the retained
build records. Trust, signed profile, accounts, devices, workers, releases and
MFA/replay behavior passed the prior comparison and negative checks. Existing
services retained their status. This supersedes the component-only installer
restore limitation above, but remains guest-loopback HTTP and lab signing evidence;
native transport/TLS recovery, migration rollback and production publication remain
unverified.

Native eleven-entry technician MSI generation passed with the lab-signed GNU
management build `b0cb740` and worker `8ec66d2`. The worker's signed SHA-256 is
`ce86aebf44178f93a981b4c296c3eecebb01cd8da923e75e7799884c01051117`;
the signed MSI fixture is
`bef611e1904bb0b6ecb183da38aa502cf52a68c66589f208408bba17824cddea`.
Both passed publisher and required RFC3161 timestamp verification. The MSI uses
recipe `8ec66d2`, version 1.5.7 and the retained signed `0a541c7` technician/agent
payloads; this is a mixed-source lab fixture, not a production source release.
Build `7373f93d-e7d3-40e8-b1e6-9e31dda99081` completed over trusted guest HTTPS;
the generated bundle SHA-256 is
`5c4b2f105e4afe6f238aa659d5a505c4e617aa4c808ec2d9e219dd305ff26957`.
Its eleven entries, company bootstrap and downloaded hash passed, and public
technician download returned 401. The later installation attempts below failed.

A new disposable standard account was prepared for MSI lifecycle testing, with
its password confined to the protected guest fixture. Initial harness failures
used a reserved PowerShell variable, assumed a nonexistent public-key location,
and generated a password that failed Windows complexity checks. The last failure
partially created the specifically marked account; its unset password was repaired
without changing another account. WinRM alternate-user process creation was denied;
a scheduled probe never ran, and the alternate-user SSH probe subsequently exited
with `0xC0000142` before either success or error markers were written. Its profile
directory exists, but no successful unprivileged process or MSI lifecycle result
was demonstrated. The specifically owned probe task was subsequently verified
idle/never-run and removed; the disposable account and profile fixtures remain.
These are test-environment failures, not proof of technician MSI installation,
repair, upgrade or uninstall.

CI run `37097192689` for `9c6bdbb9e2923d0b6dae50076a99670ee76013e6`
completed successfully. Artifact `11265830031` was downloaded with authenticated
provenance and archive SHA-256
`a01d6eab9a21b7c9e4b4ebd07ed51aa77fbdc744c48dcb9b9789972a6a9f1699`;
all 22 flat entries passed bounded ZIP/CRC extraction and published payload checksums.
The included server setup SHA-256 is
`63846db80c82c29ad0ac3f0f5984d9deb6cba9ea708255c2f12b62192dca891d`.
Reflection-only inspection verified its exact twelve-resource recipe, all eleven
payload hashes, matching embedded/external source receipt and matching standalone
management binary. Six fixed setup source inputs matched the public commit, allowing
normal Git checkout line endings; transport/HTTPS component hashes matched their
pins. The receipt declares a dirty CI working tree, so these checks do not establish
whole-tree source cleanliness, production signing or actual server installation.
The later `e9cda62` native CI run `37099948400` also completed successfully. Its
archive SHA-256 was
`f80696b2705ab43e550a5e5eb8c5dc6802e35ecc72ef1ac50b8abad6f7fb46b0`;
all 23 flat entries passed bounded ZIP/CRC extraction and published checksums.

A separately owned, loopback-forwarded SSH listener subsequently demonstrated
execution as the new standard Windows user with its actual LocalAppData profile
and no administrator membership. The original SSH listener was retained. The
generated eleven-entry bundle reached native technician MSI installation, which
returned `1601`. Its failed-setup marker was retained. Starting the guest Windows
Installer service without changing its Manual startup mode and explicitly retrying
the failed setup returned `1601` again. The isolated management/proxy processes
were stopped after both failed attempts, and existing company service statuses
were preserved. This supersedes the unsuccessful standard-user process probes
above, but does not demonstrate MSI installation, repair or uninstall. Earlier
noninteractive MSI attempts also returned `1601`; an interactive desktop test is
still required before attributing this failure to the product or environment.
Read-only inspection after both attempts confirmed sequence 0, a retained
failed-setup marker and no installed technician executable or configuration agent.
The event-log summary did not establish a specific installer-service root cause.
After those attempts, ownership checks confirmed no established sessions on the
temporary SSH listener. Its task, guest firewall rule and QEMU host forward were
removed. The original SSH banner and SSH/WinRM forwards were verified, existing
company service statuses were preserved, and private account/profile/failure
evidence was retained. No host firewall or production service was changed.

Native Windows CI run `37103083258` for
`11faae30b0dac275c7d8c795ca3c3c134ded36c9` completed successfully in all four jobs.
Artifact download and verification are in progress. CI success does not establish
clean-machine installation, native remote sessions or Windows matrix acceptance.

That download subsequently completed. Archive `5cb668d823167a5516020ec395289eff067cc10d29f5e4ba1a940e789e44d7b5`
(134,259,819 bytes) passed authenticated provenance, bounded ZIP/CRC extraction
and published checksums for all 23 entries. The server setup hash is
`fd604e329d444d201c59c4feb41fbd477cb63eb17488285b7e8627e1fb39f886`.
Reflection-only inspection verified its exact twelve-resource recipe, all eleven
payload hashes, embedded/external source receipt equality and matching standalone
management executable. Six fixed source inputs matched public commit `11faae3`,
allowing Git checkout line endings, and component pins matched. The receipt still
reports a dirty CI working tree; whole-tree cleanliness, production signing and
actual setup installation remain unproven.

The same archive's native MSVC agent
`cd2bdbc5c14cca1b9b24c16fe22f590cca96eb92cf45465085b3880dcd109631`
passed the expanded offline profile harness on the development Windows host.
Harness SHA-256 was
`2844f6148f7d2304010f6a74a0fa03871538fa10a2e8b9d0612576c5cce68fd7`.
All twelve cases passed: a valid cached profile was accepted; expired, future,
invalid-lifetime, tampered-branding, wrong-signing-key, wrong-company and older
revision profiles were rejected; company, edition, trust-key and endpoint
substitution in installer bootstrap were rejected. Each negative case matched
its specific expected error, and both state and bootstrap bytes remained unchanged.
Keys were ephemeral and not serialized. This is native CLI trust verification
against synthetic local fixtures, without network, installation, rendered branding
or remote-session evidence.

The current MSVC management executable from `11faae3` subsequently passed the
native encrypted installer-backup rehearsal on Server 2016 build 14393. Its
unsigned hash is `de623e216e4ef2b81785c18615fc23e83e4716c894225ed620a860e2aea9f817`;
the disposable lab-signed hash is
`194c837babd6a07f51b37743fb11400e37cfa6a41e48c16e2701032f2d921f3d`.
Authenticode and required RFC3161 timestamp verification passed. The restored
server served one customer bundle publicly and three technician bundles after
authentication, including the eleven-entry MSI bundle. Every download hash
matched its retained build record, and unauthenticated technician requests
returned 401. Trust files, signed profile, users/devices/workers/releases,
administrator MFA and replay checks passed; backup overwrite, nonempty restore,
wrong passwords and ciphertext modification were rejected. Both isolated
processes stopped and existing company service statuses were preserved. This
extends the GNU result to the current MSVC management runtime, over guest-loopback
HTTP; transport/TLS restore, migration rollback and production signing remain
separate unverified requirements.

The matched-source MSVC server and worker from `11faae3` then completed a real
technician MSI bundle job over trusted guest HTTPS. The worker's unsigned hash is
`73ce8e09c6473f9c32c7cd8c22d5eb0c75e68c448e1829a462df42e3843d5bd9`;
its timestamp-verified lab-signed hash is
`fa6bb94be7a44a4e2148b428af3fe1a0ca93045d0961aefd54e8c2188bb92c0c`.
Job `3444e8fd-4e4a-4624-887c-1e1b370afbd3` produced bundle
`478f2d3af2838f26245ed0723b5a009fd80ae53b58bd57418a8e2bf76e6832a9`.
All eleven entries, company/edition/endpoint bootstrap and downloaded hash passed;
unauthenticated technician download returned 401. The owned worker credential
was revoked and listed disabled, the administrator session logged out, all three
isolated processes stopped, and existing company service statuses were preserved.
This uses retained lab MSI recipe `8ec66d2`, signed endpoint/agent inputs `0a541c7`
and a newly signed test release sequence 13; it is mixed-source packaging evidence,
not installation or production source-release acceptance.

Two preceding attempts failed and remain separate evidence. The first isolated
Caddy startup crashed with `fatal: morestack on g0` before login/job creation; a
version probe of the same pinned binary succeeded. The second started HTTPS but
its worker could not fetch the older signed release's artifacts: the retained
port-443 lab endpoint had no listener. No existing endpoint was restarted. The
successful run instead used newly signed metadata pointing to the isolated
port-24442 endpoint, which served only the two named lab inputs alongside its
management proxy. Company policy was not implicitly changed, unrelated active
jobs were checked before enqueue, and component pins were unchanged. The startup
crash's cause and wider Windows server stability remain unresolved.

Five further validations of the same pinned Caddy executable and isolated
configuration passed on Server 2016, each returning exit code 0 and `Valid
configuration`. An initial harness used `Start-Process` process objects whose exit
codes were unavailable; those observations were not counted as passes. Direct
native exit-code capture produced the verified results. No listeners were opened
and existing company service statuses were preserved. This does not resolve the
earlier startup crash or prove long-running service stability.

Review of service failure handling found that management and worker report
`Stopped` with nonzero Win32 exit codes, while their installers configured recovery
actions without enabling recovery on those reported failures. Both installers now
set `sc.exe failureflag SERVICE 1` and fail setup if that configuration fails.
Microsoft documents the required flag in
[SERVICE_FAILURE_ACTIONS_FLAG](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_failure_actions_flag).
A separate unsigned, protected Server 2016 fixture demonstrated actual SCM restart
after a non-crash error stop, plus no restart after a successful requested stop.
The fixture service was disabled/stopped/deleted during cleanup and existing product
service statuses were preserved. Installer parsing and diff checks passed. This
proves the native recovery mechanism, not installation or reboot of the changed
product packages; no existing product service recovery settings were changed.

A fresh unsigned server setup package was subsequently compiled from clean
installer checkout `87a1b7f85881d31d7f43d61c5fe7bfe0ffd1048a`. All eleven embedded
payload hashes passed the builder's verification; reflection confirmed twelve
resources including the source receipt and the updated service recovery flag.
Package SHA-256:
`a5ef288f8747c44e7a3e44b9dfea027bfc207aebd428a53054186afa91107b13`.
The management executable remains the unchanged CI build from `11faae3`, hash
`de623e216e4ef2b81785c18615fc23e83e4716c894225ed620a860e2aea9f817`.
The clean installer receipt does not establish full runtime reproducibility.
The package was inspected without launching its wizard or installing services.

The new Windows corresponding-source archive was exercised against clean checkout
`c7d118c4b655d401c14fd95672ae2aff3b5ed679` and the six generated bridge inputs from
its running native CI build. GitHub artifact provenance and SHA-256 verified those
inputs before writing the previously absent ignored generated source files.
The archive included 1,107 source files and pinned `hbb_common` revision
`7e1c392c62d39c364127307cd408421dd5f8cfb0`. Every archived file hash and ZIP CRC
passed; two independently written archives had identical SHA-256
`74a7de90d0b3bc17782203b53605c917be3811dced1eefbc5ad1d2be64a4163c`.
Four focused fixture checks separately verified private/untracked file exclusion,
generated input completeness, matching/mismatched submodule pins and link denial.
This proves the source archive operation with real build inputs, not complete
native compilation, binary reproducibility, signing or release acceptance.

### Technician MSI lifecycle on a disposable Windows CI runner

Independent run `37112721733` completed successfully using fixture source
`76e89b6fffc9acd602679b8049a7520b6739ba08` and the pinned unsigned technician
MSI/application/agent from successful native build `11faae3`. Build provenance,
installer checksums and the independently verified agent hash were checked before
execution. The downloaded evidence artifact passed its GitHub SHA-256 and ZIP CRC.
Its result identifies Windows Server 2022 Datacenter, build 20348, installation
type `Server`. Native MSI install, forced repair after removal of the owned
technician executable, and uninstall all returned 0. Installed/repaired executable
and agent hashes matched the inputs; technician registry registration existed
after installation/repair and was absent after uninstall. Uninstall also removed
both payload executables, and the actual update-cancellation custom action passed.

Earlier CI fixture attempts stopped at a missing agent checksum or timed out
without installer logs. Named array binding and quoting only file arguments
corrected the command invocation; the final run retained invocation records and
all three native MSI logs. Those failed fixtures are not installation passes.
No host installation or existing lab-service change occurred. This is unsigned
MSI payload lifecycle evidence under the CI account, not standard-user company
setup, enrollment, signature acceptance, a graphical session, or the full Server
2022 compatibility matrix. The separate Server 2016 standard-user error 1601
remains unresolved.

### Published native build and corresponding source integrity

Native Windows run `37110881152`, source
`c7d118c4b655d401c14fd95672ae2aff3b5ed679`, passed all four jobs. The downloaded
24-file artifact matched GitHub's authenticated SHA-256
`c08f556d0ba316754886559e03dae33980f66cc832503e00b14c25b553336cc4`;
ZIP CRC, bounded extraction and published checksums passed. Its corresponding
source archive independently matched 1,101 public Git files, including the pinned
submodule, and six verified generated bridge files. All source manifest hashes
passed. The server setup's exact resource recipe, eleven payload hashes, embedded
source receipt, six packaging source inputs and component pins also passed.
The receipt truthfully records a dirty build working tree. This verifies supplied
build inputs and artifact integrity; it does not establish bit-for-bit binary
reproducibility, production signing or the outstanding Windows acceptance tests.

### Technician MSI lifecycle on Server 2025

Run `37115402384`, fixture source `908b447f7117c08f63315a8fbdff924b1edd4cd0`,
passed on a disposable Windows Server 2025 Datacenter runner, build 26100,
installation type `Server`. It consumed the same pinned unsigned build `11faae3`
as the Server 2022 lifecycle fixture. Actual installation, forced repair after
deleting the owned executable, and uninstall each returned zero. Payload hashes
and registry presence/removal checks passed. The retained invocation records,
result and native MSI logs were downloaded only after checking the successful
run/source; the evidence artifact matched its authenticated GitHub digest and ZIP
CRC. This covers MSI payload lifecycle under the CI account, not company setup,
standard-user installation, graphical sessions, signing or full Server 2025
acceptance. The local Server 2025 VM remains unbooted because available host
memory is below its configured safety reserve; existing services were preserved.

The fixture was then advanced to independently verified native build `c7d118c`
(run `37110881152`), including its current agent hash. Run `37115549339`, fixture
`8ad9b17c75405da0d0363635f7ba19a901c29c3a`, passed the same actual MSI install,
repair and uninstall checks on Server 2025 Datacenter build 26100. Its retained
result and logs passed authenticated artifact digest and CRC verification. This
provides current-package lifecycle evidence with the same limited scope above.

### Standard-user technician offline launcher failure and correction

After authorized manual-equivalent lab sign-in, an isolated interactive task
ran the installed launcher as `SwanLabTech`, session 1, administrator false.
With management unreachable, Windows PowerShell 5 converted the agent's sync
stderr into terminating `NativeCommandError`. The launcher stopped before its
documented cached-branding warning and installed-package verification. Native
process invocation now observes the actual sync exit code independently of
PowerShell's stderr handling. A Windows PowerShell fixture demonstrates offline
startup ordering and denies application launch when installed verification fails;
its mocked signature gate is control-flow evidence only.

The existing signed lab agent/application independently passed real signature,
hash and pinned-publisher verification. With only the launcher development fix
applied, the same owned task reached the offline warning, passed real installed
verification and started the technician executable under the standard account.
Its original launcher and failed diagnostic logs remain preserved privately.
The executable subsequently exited without a visible technician window. This
fixes the observed launcher failure; graphical startup and native sessions remain
unverified and require further investigation. No host production service changed.

The subsequent Application Error event identifies the portable technician EXE
itself, exception `0xc0000409`, offset `0x2ab55`. An owned interactive diagnostic
launched the unchanged signed EXE with explicit standard input/output/error pipes.
It returned zero and spawned `rustdesk.exe` in the same standard-user session;
the actual window displayed the cached company technician name, teal swan/gear
mark, company HTTPS domain, support link, username/password/TOTP fields and
product/license attribution. No technician login or native connection passed.

The launcher now supplies and asynchronously drains the portable wrapper's
standard handles, keeping the readers alive until inherited child pipes close.
The Windows PowerShell fixture additionally spawns a child that writes 128 KiB
to each output stream after its parent exits, and verifies child completion
before launcher return. All three launcher control-flow cases pass. The final
combined launcher still needs a newly generated package and native lifecycle
verification; the observed GUI used the isolated handle diagnostic with the
existing signed lab package. Original failure evidence and existing services
remain preserved.

### Combined launcher and deployment-check follow-up (2026-10-03)

The final combined launcher ran in the owned Server 2016 interactive task as
SwanLabTech, session 1, administrator false. Real installed-publisher verification
passed; the company-branded technician frontend remained open without a recorded
launcher failure. The task remains running while it drains inherited GUI output
pipes. This used the existing signed lab binaries with the development launcher,
not a newly generated release. GUI MFA login and native connections remain
unverified: the computer-use helper could not activate the TigerVNC options modal,
and its documented selection/activation recovery failed. No credentials were
entered into an uncertain field and no alternative UI injection was used.

The existing guest management service started after its signed executable and
loopback-only proxy configuration were verified. Normal guest HTTPS validation
returned 200 for the company profile; the customer process and installer worker
were preserved. No host service, DNS, firewall or trust settings changed.

Native Windows build 37120044435 was dispatched once from 01d24ab to embed both
launcher fixes. It is pending; build completion, artifact integrity and generated
package verification must be recorded separately.

Component run 37120224142 at 6b23cca passed Ubuntu and Windows workspace tests
and builds, including all three Windows PowerShell technician-launcher cases.
The Docker context check, image build, company setup and persistence passed after
adding the three required public inputs to both deny-by-default allowlists.
The later stack probe failed with a reset on the NAT-test socket. The probe now
uses the owned container's private bridge address because upstream hbbs treats
loopback peers on that port as administrative text connections. Published host
sockets retain separate connection checks. The KeyExchange protobuf tag was also
corrected to field 25. Actual stack verification of this change is pending; this
is not public NAT traversal or remote-session evidence.

### Native launcher packaging and Linux export follow-up (2026-10-03)

Native build [37120044435](https://github.com/rijff24/SwanRemoteSupport/actions/runs/37120044435)
passed all four jobs from `01d24ab97cef1c7d9d29da252c55ea33a747c18f`,
including both MSIs and company server setup. Its technician lifecycle evidence
artifact `11273971224`, SHA-256
`ba8e4a0d4967cb0f0dfdaf068578cc75b89736c622e2f3687f0928bbb825ba42`,
passed authenticated digest verification and ZIP CRC. The actual retained native
logs show zero returns for install, repair and uninstall and no `Return value 3`.
The fixture's payload and registration checks passed on Windows Server 2022
Datacenter build 20348. This is unsigned MSI payload lifecycle under the CI
account; standard-user company enrollment, GUI authentication, signed upgrades
and native remote sessions remain separate requirements. Main package download
and independent payload/source verification are still pending.

The new run's six-file generated bridge artifact `11273340217`, SHA-256
`79f58d06a556f10da21d83d064e13c7de81a160c236299f70b5f61bfbfa46264`,
passed source-revision, digest, recipe and CRC verification. This is generated
build-input integrity, not binary reproducibility.

Both component runs at `d724321` passed: push `37121916090` and pull request
`37121918050`. Their Linux deployment checks include the corrected private-bridge
NAT protocol probe, bounded readiness of the same owned container, three distinct
complete exports from stopped read-only transport/TLS volumes, authentication
failure without restore writes, actual restore, and retained company, transport,
TLS, MFA, device and consent identity. Host-published sockets retain separate
checks. This remains an isolated deployment rehearsal, not external NAT traversal
or an authorized remote desktop session.

Earlier parallel Linux runs intermittently failed complete export with SQLite
error code 14 (`SQLITE_CANTOPEN`). They remain failed evidence despite subsequent
passing exports. Distinct management/transport open and snapshot error contexts
and redacted error-category reporting were added; no captured backup output or
credentials are printed. The precise failed operation has not yet been reproduced
with those contexts. Backup reliability remains an open concern.

### Read-only transport WAL defect reproduced and corrected (2026-10-03)

The Unix regression `readonly_transport_wal_export_preserves_committed_peers`
failed in run `37123006043` at source `8fc7ed0`: transport snapshot returned
SQLite code 14 when committed WAL frames existed, the source directory was
read-only, and no shared-memory file existed. This reproduces a concrete defect
with the same error code as the earlier intermittent exports; those earlier
logs did not identify the precise failed operation.

Transport export now copies the stopped database and its WAL/rollback journal
into private writable staging, lets SQLite recover and snapshot that copy, and
removes fixed temporary files and the owned directory afterward. Copies are
bounded to the declared file length plus one byte and a combined 512 MiB limit;
regular-file/reparse checks reject unsafe sources. The original transport
storage is untouched. Transport must remain stopped for a consistent multi-file
copy, as documented in `SERVER_BACKUP.md`.

The regression passed at `f71773bd54190d14ad0332ff1ac6e5213f3f879b`, verifying
restored committed peer data, unchanged source database/WAL bytes and cleanup of
plaintext snapshots/staging. Both full component runs passed: push `37123392820`
and pull request `37123395298`, including Windows tests/builds and the complete
Linux deployment/export/restore rehearsal. Local Windows GNU type checking also
passed; local test execution was unavailable because that toolchain lacked
`libgcc_eh`, and is not counted as a test pass.

A new full native build `37123744363` was dispatched once from the verified
`f71773b` source to package this runtime fix. It is still pending. The earlier
`01d24ab` installer archive remains in its original live download process;
no partial download was replaced or treated as verified.

### Native package provenance and failed-export cleanup (2026-10-03)

The original download of native run `37120044435` completed successfully.
Its 24-file archive passed authenticated artifact SHA-256
`61a7618abd11fa1a60138f7ec944499e7d18460f1a86ff1fc6153c884d77f452`,
ZIP CRC and published payload checksums. Independent checks verified all 11
embedded server setup resources, embedded/external source receipt agreement,
standalone management payload agreement, six public setup source inputs and
pinned transport/proxy hashes. The published source archive matched 1,103
public Git files at `01d24ab` plus six independently verified generated bridge
files. Its working-tree-dirty flag is retained; matching archived inputs is
source provenance, not proof of a clean checkout or binary reproducibility.

The packaged technician launcher matched its exact public source after line
ending normalization. The technician MSI, portable executable and agent hashes
matched their build manifest. The retained CI lifecycle receipt does not include
an independent hash of the tested MSI; its successful install/repair/uninstall
evidence remains scoped to that CI job and must not imply fresh-company or
standard-user acceptance. These artifacts predate the `f71773b` backup fix.

At `5ae76f3`, `failed_transport_snapshot_removes_private_staging_without_changing_source`
passed in actual Windows MSVC and Linux test logs. The fixture verifies that
an invalid transport database fails export, leaves no archive or private plaintext
staging, and preserves the source bytes. Both complete component runs passed:
pull request `37124053449` and push `37124051532`, including the isolated Linux
deployment rehearsal and Windows launcher checks. This test-only commit does
not change the runtime currently being built in native run `37123744363`.

### MSI evidence bound to tested artifacts (2026-10-03)

The lifecycle fixture at `940c64f` now records the package SHA-256, expected
installed payload hashes and fixture revision in its retained receipt. Each
native invocation records the package hash, and the fixture checks that the
package remains unchanged before and after each operation. Fixture revision
identifies the test code; it does not imply that older downloaded binaries were
built from that revision.

Disposable Server 2022 run `37124891111` passed installation, deliberate payload
removal followed by repair, and uninstall against the pinned `c7d118c` package.
Evidence artifact `11275180118` passed authenticated SHA-256
`9940004bb6893f8cade24d18667bc0ebe5cd3140c84aa72ec37d23b3e8c93def`
and CRC verification. The receipt and all three invocation hashes matched the
independently retained MSI SHA-256
`5083a718a1b7f367fe680ada622ca8ce6a55c99f55bf80224c7841d3c257979b`;
expected executable/agent hashes matched the retained native build. All three
native logs contained successful zero returns and no `Return value 3`.
This strengthens artifact-specific lifecycle evidence for that older package;
it does not cover the new backup runtime, standard-user enrollment, production
signature acceptance, GUI login or remote desktop sessions.

### Authorization expiry boundary (2026-10-03)

Review identified that an in-flight renewal response could extend an already
expired device-side lease. At `391541e`, renewal rejects an expired lease before
the request and again before applying the response. The request timeout is
bounded by the remaining lease time and the normal 15-second limit. The receiving
native app also closes an authorized connection on an incoming packet when its
lease has expired, instead of relying only on the next periodic timer tick.

`delayed_renewal_cannot_resurrect_expired_authorization` passed in independently
read Linux and Windows MSVC logs from pull-request run `37125172883`. Both full
component runs passed, including push `37125170016` and the isolated Linux
deployment rehearsal. Local Windows GNU type checking passed. These prove the
renewal validation regression, not a live native session or its termination.
Native build `37125194672` is running from this exact source; build `37123744363`
contains the earlier backup fix and cannot validate the new expiry behavior.

### Native backup runtime verification (2026-10-04)

Native runs `37123744363` (`f71773b`, backup fix) and `37125194672`
(`391541e`, authorization expiry fix) completed successfully. The latter's
package download remains pending independent verification. The former's archive
`11275251202` passed SHA-256
`bb99ad350b2f633cf738fc91028e2e80794b77575229e688a3a84322ee841125`,
CRC and payload checksum verification. Independent checks matched 11 embedded
server resources, six setup source inputs, component pins, 1,103 public Git
files and six generated bridge files. The dirty-checkout flag remains retained;
this is source provenance, not binary reproducibility.

The verified native management executable SHA-256
`f193918d1c089172c5de88ca62fd7d56323498f36050b53619ccc4693aa5fa55`
passed a synthetic Windows CLI export/restore rehearsal. Its stopped transport
fixture contained committed WAL frames and no SHM, with a read-only directory
ACL verified by a denied write probe. Restore retained the committed peer row
and management trust files. Source bytes stayed unchanged, no SHM appeared in
the source, and plaintext staging was removed. Invalid transport export left no
archive or staging; tampered restore created no destination. Fixture setup first
failed because Windows PowerShell inherited incompatible module paths; correcting
only the child process environment allowed the actual runtime test to execute.
No existing service, company database or host security setting was changed.
This is native CLI behavior with synthetic data, not service deployment or a
real-company disaster recovery test.

The build's retained MSI lifecycle artifact `11274617033` independently passed
SHA-256 `659aa1ee0a18f18dcb30494e4a13937aeeeeb4eb4cec5c0f859d74aa37cdead2`,
CRC and native successful install/repair/uninstall log checks on Server 2022
build 20348. Its older fixture lacks package hashes in the receipt; standard-user
enrollment, GUI login and native remote sessions remain unverified.

The authorization-expiry native build's MSI evidence artifact `11274947759`
also passed authenticated digest
`4dc584d2a4cbdc62f1e247ff76395173ead5364b4ffc0e12555e2091e533ab60`,
CRC and successful native install/repair/uninstall log checks. Its receipt and
all invocation records identify fixture `391541e` and MSI SHA-256
`91b63c8cb2f164d25955c10ae82b8e5bc7dc2d10ac9ae3b99680a42f4132d33f`.
Comparison of that recorded package/payload identity with the main downloaded
archive remains pending; the archive's original process is still running.
This evidence remains limited to unsigned Server 2022 CI payload lifecycle.

### Expiry-fix artifact and native backup checks (2026-10-21)

The interrupted `391541e` download was found stopped with 111,149,056 bytes
retained and no live downloader. Resume requested only the remaining byte range
of the authenticated artifact; the complete 24-file archive passed SHA-256
`b627e5369fff25f0555159577b14956e06f983facd963e3c059a1cc3fe14eb37`,
ZIP CRC and published payload checksums. Independent verification matched the
11 server setup resources, six source inputs, pinned components, 1,103 public
Git files and six generated bridge files. The MSI, technician executable and
agent matched the hashes recorded by the CI lifecycle test. This resolves the
pending artifact comparison above; dirty-checkout and binary reproducibility
limits remain unchanged.

Native management SHA-256
`c2b3cc3c20ab5199777d1bacddedf2d31deff0bef4a6ea311ac77975afd9b0c1`
passed the same synthetic Windows read-only WAL export/restore, unchanged-source,
staging cleanup, invalid-transport failure and tampered-restore checks. This
verifies backup CLI behavior in the current packaged runtime; it does not verify
native session expiry, company disaster recovery or deployment lifecycle.

The existing disposable Server 2016 VM was restarted through its verified
launcher without changing its memory guard or network settings. An encrypted
management probe confirmed `SWAN-LAB-16V2`, Server 2016 Standard Evaluation,
64-bit build 14393. Boot and guest identity are readiness evidence only; no new
GUI login or remote-session acceptance is claimed.

### Native technician login and failed rendezvous attempt (2026-10-21)

The existing signed lab technician installation launched as unelevated
`SwanLabTech` in session 1 after configuration sync and two successful package
signature checks. File preparation was slow but showed changing read/CPU
counters; the original process was retained until its frontend opened. The
native company-branded form accepted the restricted `lab-technician` account
and a fresh TOTP, then displayed the approved lab device in its authorized
inventory. Credentials and the authenticator secret were kept private. This
uses the existing older signed lab payload, not the newly downloaded release.

Requesting attended support opened the native connection window for the
enrolled device, then failed with `Failed to connect via rendezvous server`.
No customer-consent prompt or desktop session was demonstrated. Client logs
recorded three TCP punch attempts; the lab hbbs log recorded Windows UDP error
10014 and UDP-loop restarts at the same three times. Both endpoint state files
had accepted profile revision 15, whose transport addresses resolve to the
loopback-only lab server. This is failed transport acceptance, with diagnosis
pending; it is not evidence of working NAT traversal or relay fallback.

After the long lab downtime, the TLS front end initially served an expired
internal leaf certificate. A normal restart of the verified guest company
service restored validated HTTPS; customer and worker services stayed running.
No invalid certificate was accepted. The viewer's relative mouse mapping also
misplaced clicks; full-screen keyboard capture allowed verified keyboard-only
form navigation. Adding a USB controller live was rejected by the VM's PCI bus,
so no live USB hardware change succeeded.

### Candidate Windows UDP correction (2026-10-21)

An isolated Winsock probe on Server 2016 build 14393 reproduced error 10014
when an IPv4 destination was passed to a dual-stack IPv6 UDP socket. The same
probe delivered its datagram when the destination used an IPv4-mapped IPv6
address. No existing service or network configuration changed for this probe.

Commit `e7953c9` provides pinned public transport inputs, two reviewed patches,
and workflow run `39596805961`. The Windows 2022 baseline reproduced error
10014 in the actual shared Rust UDP message path while the IPv4 and IPv6
controls passed. After destination normalization, all three real socket tests
passed on Windows and Ubuntu 24.04. The tests cover protobuf message delivery,
raw datagram delivery, IPv4 sockets, and native IPv6 destinations. An earlier
test fixture used the wrong protobuf field; its correction exposed interleaved
parallel test output. The final fixture runs sequentially and retains strict
failure and control checks; a compilation failure does not count as reproduction.

Both candidate executable builds succeeded. The four downloaded candidate and
regression artifacts passed their authenticated archive digests, ZIP integrity,
receipt, patch, lock, and public recipe checks. The Windows candidate artifact
`12344418248` has SHA-256
`be388753b8b3edf3530dceea59137b1477300cb67eb46d6cc1a12a584c4e3049`.
The source receipt identifies PR merge `c09c4bd8ac85f86e1f0fd10d2853a09b266545a4`;
its parent includes `e7953c9`, and archived recipe files match that reviewed
revision. This is input provenance, not binary reproducibility.

On the existing Server 2016 build 14393 guest, candidate `hbbs.exe` SHA-256
`83d402b48cec61513b6a0e3a6277c648b31a6ccbce0b87eee022b96ca746a443`
and `hbbr.exe` SHA-256
`8ba65379f7a025cad6ec0520d481af62c135e117ac58ac7b460c7b1afbf751cd`
both started and reported version 1.1.14, as specified by the pinned upstream
Cargo package. The candidate rendezvous server then answered real IPv4 and
IPv6 `RegisterPeer` datagrams with a request for a public key. It used separate
ports and a fresh database/key directory, logged no UDP failure, and was stopped
after the test. The temporary loopback artifact transfer server was also stopped.
Existing company, worker, customer and technician processes remained running.

Existing distribution pins and lab services have not been replaced. These
unsigned candidate checks establish startup and basic registration on Server
2016; they do not establish a native authorized session, relay operation,
external NAT traversal, complete Windows compatibility, or release readiness.

### Native attended session through parallel patched transport (2026-10-21)

The candidate ran alongside the original lab company services on rendezvous
32116 and relay 32117, using a separate database and a protected copy of the
lab transport keys. A normal authenticated profile change published revision
16 with those addresses and paused lab updates. Both endpoint state files
accepted the signed revision; customer identity, release sequence and unattended
consent (`false`) were preserved. Original component pins and services stayed
intact. No host DNS, firewall, trust store or Tailscale setting changed.

This exposed two runtime configuration defects. The running technician accepted
the new profile but continued selecting port 21116. Commit `74c3449` applies
transport defaults after synchronization and before a new outgoing request.
The customer, booted after its cached profile expired, kept resolving the
fail-closed `unconfigured.invalid` address: DNS failed before either transport's
refresh timer could run. Commit `bd13500` also reloads signed settings before
each outer rendezvous retry. Native build `39600988305` is verifying both fixes;
neither fix's runtime acceptance is claimed yet. The initial suspicion of a
transport-ID mismatch was disproved: native and enrolled IDs matched.

Restarting the existing signed lab technician and customer service consumed
revision 16 without changing identity or consent. The customer registered its
public key with the parallel server. The technician's actual branded GUI login
with the restricted account and fresh TOTP obtained a grant, punched a secure
local TCP connection, and reached the customer's Accept/Cancel prompt. Explicit
customer acceptance at 14:43:23 lab time opened a VP8 desktop stream and the
clipboard loop. The desktop was visibly rendered; the recursive image is
expected because both editions ran in the same VM. The management service
recorded the grant as claimed and renewed its lease.

Device-authenticated closure of that specific grant stopped clipboard, video
and audio at 14:45:21. The client requested a fresh grant but returned to the
customer approval prompt. Closing that pending reconnect also closed its grant;
no continuing desktop access was observed. Viewer relative-pointer offset made
consent input difficult; measured pointer correction eventually selected Accept.
The existing signed endpoint payloads were used, not the pending native build.
This establishes an attended local session and enforced grant closure with the
candidate transport. It does not establish natural lease-expiry behavior, data
transfer or clipboard-content correctness, unattended access, separate-machine
input, external NAT traversal, relay fallback, or the full Windows test matrix.

### Reboot and relay-test follow-up (2026-10-21)

The disposable Server 2016 guest was shut down and relaunched with a USB tablet.
The company server, installer worker and customer service started automatically;
HTTPS worked, and device identity, consent and release sequence were preserved.
The manual parallel transport task was restarted separately. The viewer's
connection-refused dialog during shutdown cleared after reconnecting. Absolute
pointer input subsequently selected the actual customer Accept button without
the previous relative-pointer compensation. Windows displayed Shutdown Event
Tracker, so clean shutdown-event logging is not claimed.

The existing customer payload again remained on `unconfigured.invalid` after
startup despite accepting profile revision 16. Restarting only that lab service
recovered registration. This reproduces the pending configuration-retry fix;
it is not evidence that the new native build has passed. Build `39600988305`
reached the branded-client compilation step after its prerequisite jobs passed.

The parallel server reported `ALWAYS_USE_RELAY=Y`, but the subsequent accepted
session still logged a secure local TCP connection (`is_local: true`). The
server preference therefore did not make this same-machine test a relay test.
Desktop video appeared after customer approval at 15:04 lab time. Later the
guest display stopped changing, both WinRM probes timed out, and SSH timed out
during banner exchange while QEMU reported the VM running. Attempts to close
the native connection through its window were not confirmed. The guest state
was preserved for diagnosis; the cause of the unresponsiveness is unproven.
Relay operation, session closure for this attempt, and post-failure recovery
remain unverified.

### Private signed packages and standard-user attempt, 21 October

Source `bd1350012227f226db168be51bc823d3f086e493` now has private-lab
customer and technician 1.5.7 MSI wrappers, plus signed agent and worker inputs.
Authenticode publisher, pinned certificate and RFC3161 timestamp checks passed
in the signing guest. This is company lab signing, not project signing approval.
Signed release metadata for test channel sequence 8 was prepared and locally
verified; it has not been imported, approved or rolled out.

On the disposable Server 2025 build 26100 guest, a new non-administrator account
verified the signed technician MSI's expected hash, publisher and certificate.
Its background scheduled-task installation then failed with MSI exit 1601:
the raw log reports access denied (0x80070005) connecting to Windows Installer.
Repair and uninstall were not attempted. The account needed an explicit batch
logon right to start the task; it was never added to Administrators. This failed
background test does not establish interactive standard-user install behavior.
The result, wrapper and raw MSI log were exported and retained privately.
The VM console subsequently showed a black display while administrative remote
access remained available; an interactive rerun is still outstanding.

The second guest cannot yet reach the company HTTPS service because that
service binds only to the first guest's loopback address. Automatic approval
review rejected the proposed private-NIC listener addition and service restart
with only "blocked by policy." No listener change was applied. A reviewable
private draft and an explicit approval request remain pending. The initial peer
HTTPS request timed out; this is not a passing negative certificate test.
Cross-machine native sessions and the runtime configuration fixes remain
unverified. Host RustDesk and its transport servers were not stopped.

### Matched worker and interactive MSI follow-up, 21 October

The Server 2025 console recovered to its normal logon screen. The same signed
technician MSI was then tested as the non-administrator account in interactive
session 1. Publisher verification passed, but installation returned 1625. The
raw MSI log says `DisableMsi` is 1 and rejects non-assigned applications for
non-admin users; repair and uninstall were not attempted. This independently
reproduces the Server 2016 policy limitation documented in INSTALLER_REPAIR.md.
No Windows Installer policy was changed. Both failed attempts remain evidence;
neither is a passing standard-user MSI lifecycle test.

The `bd1350012227f226db168be51bc823d3f086e493` worker was privately signed and
installed over the isolated lab worker with a rollback binary/receipt retained.
The service runs with its existing protected configuration. After administrator
MFA login, both sequence-8 releases were imported and approved with rollout
still paused. Unapproved build requests did not change the queue, but returned
500. The source now returns 404 for unknown releases and 409 for unapproved
releases, with regression assertions covering anonymous/technician denial and
an unchanged build count. CI verification of this correction is pending.

The new worker verified the signed packages and generated both ZIP bundles,
but upload to the older installed management service returned 400. The worker's
embedded launcher differs from the older server's expected launcher. Neither
job reached completed state; generated files alone do not demonstrate server
publication. The matching management binary was signed and timestamp-verified
(private lab hash `cf7b5077a633f8ff9ad830d2e2a9847d16b35494bbeaf061645abbe8c37c07d4`),
but has not been installed.

Automatic approval review rejected the proposed management-backup preparation
with only "blocked by policy" before any command ran. No release withdrawal,
backup, restore or management-service change occurred. A separate reviewable
private draft and approval request remain pending. This is distinct from the
previously rejected private-NIC HTTPS listener change. Host services were not
changed; latest native runtime, cross-machine and update acceptance remain open.

### Latest component checks and local customer upgrade, 21 October

Component workflow `39629396018` passed for exact source
`8c321705e0b3450f440e3c1a7ade6a328aad47ec`: locked Rust tests and release builds
on Linux and Windows, Windows setup/launcher/removal checks, and the Linux
HTTPS/transport deployment rehearsal. The new release-request assertions passed
on both operating systems. The first attempt failed because the test expected
403 instead of the existing administrator-only authentication response 401;
the test was corrected without changing authentication policy.

Both `bd13500` worker ZIP files passed independent path/count/CRC, exact source
script/license, signed profile/release and package/agent hash checks. Bootstrap
company identity, endpoint and both public trust keys matched the independently
installed customer state. These checks do not turn the rejected server uploads
into completed publication jobs.

Using that verified local customer bundle, an explicit Server 2016 upgrade from
1.5.6 sequence 7 to 1.5.7 sequence 8 passed. Device identity/token, unattended
consent and its revision were preserved, expected native/agent hashes matched,
recovery markers cleared, and the support service and configuration watcher
ran afterward. This was local explicit setup, not automatic rollout or a public
download test. The separate transport candidate was restarted after preserving
its previous logs; the upgraded native receiver contacted the configured
32116 endpoint and received a public-key registration request and NAT response.
This is not a separate-machine or relay desktop-session test.

Runtime logs also showed legacy RustDesk Pro HTTP heartbeat/sysinfo attempts.
The source now disables that endpoint and ignores unsigned legacy strategy
options for `swan_custom` editions. The native Windows workflow includes an exact
regression-test gate requiring one executed passing test. Native compilation
and runtime verification of this guard remain pending; no unauthorized desktop
session through legacy strategy was demonstrated. The installed `bd13500`
candidate predates this guard and is not approved for production.

A new isolated Server 2019 evaluation VM was created from the verified official
ISO, with a unique NIC address and private credentials/seed outside Git. The idle
Server 2025 guest shut down normally before launch. Server 2019 installation is
in progress, not yet a passing product compatibility test. Host RustDesk and
its server remain running. The rejected management backup and HTTPS listener
operations remain unchanged pending explicit approvals.

### Clean signed Server 2019 MSI lifecycle, 21 October

The new VM installed Windows Server 2019 Standard Evaluation Desktop Experience
x64, exact build `17763.3650`, from the verified Microsoft evaluation ISO. Its
unattended lab bootstrap did not run; authenticated remote administration verified
the guest before the hash-pinned private seed bootstrap was run manually. SSH
provisioning completed, automatic logon was disabled and readiness was recorded.
This was lab OS preparation, not a product installer failure.

After verifying staged source/package hashes, the unique NIC address and absence
of Swan product services, Windows shut down normally. The stopped QCOW2 disk
passed its integrity check and received clean checkpoint
`swan-clean-server2019-bd13500`. It then booted with installation media detached.

Both privately signed 1.5.7 MSI packages from exact native source
`bd1350012227f226db168be51bc823d3f086e493` passed install, repair and uninstall
as `SWAN-LAB-19\Administrator` (administrator true, session 0). Each package
passed its pinned publisher/certificate/hash verifier before its lifecycle phases.
The public lifecycle runner verified Windows Installer registration, installed
payload identities, customer service ownership/state, technician shortcut and
registry entries, damaged customer license repair, and uninstall cleanup.
All six MSI exit codes were 0, with no reboot required. Exported reports matched
the pinned runner and package inputs; independent raw-log checks found successful
engine returns and no `Return value 3`. The machine's trust changes were limited
to the public private-lab publisher certificate in the disposable guest.

Private receipt `windows-server2019-results-bd13500/verified-lifecycle.json`
retains hashes for all six reports and raw logs. This is signed administrator MSI
lifecycle evidence, not standard-user installation, company enrollment, native
sessions, approved automatic rollout or complete Server 2019 acceptance.

The native Windows build for the legacy-sync guard is workflow `39631663952`,
exact source `c0bf4b40ff8257c9fb69e4c5f6aceb2394f1adc0`. Bridge/helper jobs passed;
its native build and exact regression gate are still running. Do not substitute
the older `bd13500` installed candidate for runtime verification of that guard.
The lab management-backup and private-NIC HTTPS operations remain unapplied
pending the explicit approvals requested after automatic review rejected them.

### Company bundle entry point implementation, 21 October

The company bundle recipe now includes `Install-SwanRemoteSupport.cmd` as an
exact server-validated file. It starts 64-bit Windows PowerShell from the
extracted bundle, including from a 32-bit calling environment. Customer setup
can request elevation before enrollment or installation state is created;
technician setup retains the current user's context. Explicit repair, failed
setup replacement and consent flags survive a customer elevation handoff, while
the elevated child still performs the existing company and package checks.
A cancelled or failed handoff returns failure. The default entry point does not
opt into unattended access.

Local Windows PowerShell 5.1 checks passed: malformed/HTTP bootstrap rejection,
no technician elevation or state writes on signature rejection, argument/error
routing fixtures for a non-admin customer caller, and a real CMD-to-PowerShell
run with LF bundle text and spaces, ampersand, apostrophe and exclamation mark in
the folder name. That real entry-point run rejected an unsigned inert agent and
returned failure without installation or enrollment. The elevation broker was
replaced only in the routing fixture; these checks do not demonstrate a real
successful UAC handoff or a complete installed company bundle.

Windows CI now runs the entry-point checks, and bundle validation tests reject
missing or changed entry-point bytes. Both Docker context allowlists include
only this new required public script. CI builds and actual company-generated
bundle acceptance for this recipe remain pending. Server and worker must use
the matching updated recipe. The customer installation guide now describes the
configurable company flow and links the Tailscale instructions to the preserved
baseline rather than presenting them as current setup.

### Real setup elevation handoff and native guard build, 21 October

The actual Server 2016 UAC broker was exercised from the existing standard lab
account in interactive session 1 using setup source
`0d17a30cd2e323f87e798eabab5aa53918096d3f`. The expanded prompt showed the exact
pinned setup path, bootstrap path and confirmed test domain. Authentication used
only the disposable VM's administrator. The parent received elevated exit code 1
from the unsigned inert-agent fixture and correctly reported failure. Its task
completed successfully as a test, with the parent still non-administrator.
An independent fingerprint comparison preserved installed customer identity,
credential, unattended consent/revision and installed release. This verifies
actual elevation and failed-child propagation; it is not successful installation
or remote-session control of a UAC secure desktop.

The native guard workflow `39631663952` completed successfully. Its native log
records exactly one executed passing
`hbbs_http::sync::managed_sync_tests::managed_sync_rejects_legacy_endpoint_and_unsigned_options`
test (0 failures, 71 filtered out). Artifact `12365255815` is 141,969,478 bytes,
with GitHub SHA-256 `8de1a2b18ffffa86106d24dbbc2f641dec225c802b9f7d18afa5417282a3c391`.
The existing download is live and progressing; archive/source validation,
private signing and installed runtime rejection of legacy traffic remain open.
This build predates the new company-bundle CMD entry point.

Hosted Windows CI exposed a setup-test input bug: .NET Framework flushed a UTF-8
byte-order mark into redirected stdin at process creation, before a replacement
writer could be attached. Setup safely rejected the resulting hostname mismatch.
That failure was reproduced locally with a BOM-emitting console input encoding.
The fixture now selects a BOM-free encoding during process creation and restores
the previous encoding immediately; both the reproduced environment and normal
Windows PowerShell run pass. Production hostname confirmation was not relaxed.
Hosted Windows CI subsequently passed in push run `39642194595` and PR run `39642203316` for source `43e5b37dc09fa0ed04bb5e602ca07dc19d705592`. Both runs also passed Linux component and Linux deployment verification.

The component workflow now preserves corresponding component/deployment source
before compilation and exports unsigned test binaries with source/compiler
metadata and hashes after successful checks. The source archiver's component
mode omits unrelated client bridges/submodules and private untracked files while
preserving actual file bytes; all five source archive tests pass locally.
The successful push run exported Windows and Linux component artifacts, each with corresponding source and hashes. Independent downloaded artifact/source verification and runtime acceptance remain pending. Production
signing, matching company server/worker runtime, public connectivity, complete
Windows coverage and the remaining session/update/restore gates remain open.

### Downloaded component provenance and fresh management runtime, 21 October

Windows component artifact `12366376059` from successful workflow `39642194595`
was downloaded and checked against GitHub's SHA-256
`6cb1a4f73f8fd601b26b2530b7cb393ead02ac7b17f79955895ea0ce21a60963`.
All five entries passed CRC and declared hash verification. An independent check
compared all 114 component source files, modes and manifest hashes with public
Git revision `43e5b37dc09fa0ed04bb5e602ca07dc19d705592`, accounting for checkout
line endings. These checks prove corresponding build inputs, not bit-for-bit
binary reproducibility or production signing.

That exact management executable ran against a fresh, protected company directory
and an ephemeral host loopback HTTP listener. Seventy-four assertions passed:
one-time setup, setup/MFA replay rejection, signed branded profile revision,
technician role and group restrictions, pending enrollment denial, target-bound
signed grants, tampered and repeated claim rejection, permission reduction,
unattended opt-in/revocation and stale consent rejection, build rejection without
queue mutation, worker authentication/revocation, audit history, user disable and
logout. A process restart preserved company key, branding revision, device
enrollment, closed grant history and revoked access. The isolated test process
was stopped afterward; existing company and transport services were untouched.
This is executable/SQLite/HTTP API evidence, not HTTPS deployment, native desktop
authorization, installer generation, migration, backup/restore or update evidence.

The earlier worker/server bundle upload rejection motivates a compatibility
handshake before claiming jobs: the worker checks an authenticated recipe endpoint
and the server checks the matching versioned digest on every claim before queue
mutation. Old or different recipes are rejected explicitly. Source regression
checks cover preserved queued/expired claims on mismatch and preflight responses
from old/incompatible servers. Push workflow `39644529161` and PR workflow
`39644541313` passed for revision
`481439d62af5189706a1b83263af53d603f7de19`. Windows and Linux logs confirm the
recipe normalization/content regression, management claim-compatibility regression
and real loopback HTTP worker preflight regression executed and passed. Linux
deployment checks also passed. Independently downloaded executable and installed
worker verification of this change remain pending.

### Verified compatibility runtime and next signed lab candidate, 21 October

Windows component artifact `12366189203` from workflow `39644529161` passed
independent archive verification against SHA-256
`155e077843dedf212be655d122c57d00bbea034d100af4c4b064f6418f1b2100`, all five
payload entries and all 114 corresponding source files at revision `481439d`.
Its actual management executable passed 80 isolated loopback HTTP assertions,
including a recipe digest independently recomputed from its verified source and
rejection of missing/wrong claim digests. A second fresh-company run passed 85
assertions, adding naturally elapsed grant expiry and authorization-lease expiry:
an unclaimed ticket was rejected after its 60-second lifetime, and an existing
lease could not be renewed after more than 300 seconds. No host clock changes,
mock timers or database expiry edits were used. Both test processes stopped.
These remain management API results; active native desktop disconnection on
lease expiry still requires separate evidence.

Native artifact `12365255815` finished downloading and matched its GitHub digest.
Independent validation compared 1,111 tracked source files and six generated
bridge inputs with pinned public Git/CI inputs. All 95 extracted customer payload
files matched the published installed-file manifest. The technician MSI and both
payload hashes also matched the separately verified successful Server 2022 CI
install/repair/uninstall evidence. This does not prove enrolled native sessions.

A private lab candidate combines native source `c0bf4b4` with component source
`481439d`; the separate origins are retained in its input/signing records. The
native applications and three component executables received verified timestamped
signatures from the existing nonexportable disposable-VM publisher. New customer
and technician MSI version `1.5.8` packages were built with the pinned WiX recipe
and separately signed/verified using that same private lab publisher. No private
key was exported, host trust imported, installed service replaced or production
publication performed. These packages still require installation, enrollment,
upgrade and native-session verification.

### New candidate uninstall regression, 21 October

The signed composite `1.5.8` candidate was exercised on the same Server 2019
guest after the prior `1.5.7` lifecycle. Customer installation and deliberate
license-damage repair passed. Customer uninstall failed with MSI 1603 / custom
action 1722 at `RemoveCompanyConfigurationTask`; technician cases were not
reached. The full failing report and raw log were preserved. This is a failed
acceptance gate, not a six-case success. Windows Installer rollback retained
the installed customer executable/agent and running service.

The existing cancellation helper rejects a retained uninstall rollback record.
Successful prior bare MSI uninstall left that record until explicit verified
company setup, making a later bare reinstall/removal encounter stale state.
The fix adds a commit-only cleanup action using an embedded copy of the release
agent, since its installed copy has already been deleted at commit. Cleanup is
restricted to the canonical customer directory, validates both rollback records
before removal, rejects reparse points and an agent still present, and preserves
the uninstall cancellation marker, identity and consent. Rollback data remains
available until transaction commit. Local Windows PowerShell 5 regression checks
and all three MSI-authoring tests pass; CI, rebuilt signed packages and actual
repeated uninstall/rollback verification remain pending. The failed candidate
has not been published or approved for production.

An independent technician-only `1.5.8` lifecycle attempt was subsequently stopped
by the fresh-install guard before installation: the earlier successfully
uninstalled technician product had left its cancellation marker, cancellation
rollback JSON and helper script in the per-user directory. No technician MSI was
registered. This is a blocked test precondition, not a passed or failed technician
MSI execution. The commit cleanup now covers both editions: SYSTEM/canonical
ProgramData for customer, impersonated user/canonical LocalApplicationData for
technician. Both use an embedded agent so removal does not delete the executable
needed for commit. Local fixture checks reject arbitrary paths for either edition;
MSI-authoring tests also assert the technician action retains user context.
Subsequent verified package results are recorded below; interrupted-transaction
rollback still needs separate verification.

### Verified two-edition uninstall fix on Server 2019, 21 October

Push workflow `39649347086` and PR workflow `39649354472` passed for source
`053a52e3ac29cb0f7e9686da9c626f94bf153005`, including Windows PowerShell 5
commit/rollback-state fixtures and all four MSI-authoring checks. Windows artifact
`12368667260` matched GitHub SHA-256
`b3bdf363098173d49efb4665ef6204f47d5e4ed0ac20512f1932ee5ef3fc4193`.
All five archive entries and 116 corresponding source files passed independent
hash/CRC/public-Git verification. These are build-input checks, not bit-for-bit
binary reproducibility.

The fixed component agent was privately signed and combined with the previously
verified native `c0bf4b4` payload. Both rebuilt MSI `1.5.9` packages received
verified timestamped signatures from the existing nonexportable disposable-VM
publisher; no host trust or production signing was added. The signed customer
MSI SHA-256 is `8a64ca8a90573af75cfcc8e247e6c4d5999b7d488c1a5a9b2202f988b595f678`;
technician is `8caf16d8fcfa5e47e2ed153cc2d094154c3de8d098109863572c68d70a2bbbcc`.

All 12 operations passed as Administrator on Server 2019 Standard Evaluation
Desktop Experience `17763.3650`:

| Edition | Verified operations |
| --- | --- |
| Customer | Upgrade failed `1.5.8` candidate to `1.5.9`, repair, uninstall, reinstall, repair, uninstall |
| Technician | Install, repair, uninstall, reinstall, repair, uninstall |

Every raw MSI log independently showed exit 0 with no `Return value 3`. The
guarded repair checks verified the installed agent, full customer payload and
license repair, or technician executable/shortcut/registration before and after
repair. Reinstalls retained their existing cancellation marker. Both editions'
uninstall commit actions executed, clearing obsolete rollback records after
removal. The earlier technician directory contained exactly its three known
legacy cancellation files and no registered product; those files were preserved
under the protected lab test directory before the first new technician install.
They were not discarded to make the repeated-install cases pass: subsequent
reinstalls used the new retained state in place.

A separate read-only guest check confirmed no customer or technician product
registration, no installed configuration agents, no customer service or watcher,
both uninstall cancellation markers retained, both cancellation rollback records
absent, and unchanged hashes for all preserved legacy files. The native and
component source origins remain separate in the evidence. These tests used the
existing guest after prior package tests, not a new clean snapshot. Company
enrollment/identity preservation, standard-user technician installation, automatic
updates, interrupted-removal rollback and native desktop sessions remain separate
acceptance gates.

### Actual MSI rollback and customer state permissions, 21 October

Two private `1.5.10` fault-injection MSIs were built from native `c0bf4b4` and
component `053a52e` inputs. Their only injected behavior was an opt-in deferred
failure after `RemoveFiles`; the fixtures were lab-signed, not approved or
published as releases. Both editions passed installation and baseline repair,
the deliberately failing uninstall returned the expected 1603, full installed
payload/registration were restored, subsequent repair passed, and normal
uninstall succeeded. Independent raw-log checks verified actual deferred file
removal before `LabFailRemoval` returned 1, followed by rollback. Ten operations
matched their expected outcomes, including exactly two intentional failures.

The customer test created an owned, disabled SYSTEM configuration watcher before
fault injection. Rollback restored its exact agent path, `watch` argument,
SYSTEM principal and disabled state. Both editions retained the prior uninstall
cancellation marker and cleared temporary rollback records after restoration;
the final normal removal committed. These are installer rollback results on
Server 2019 `17763.3650`, not management database backup/restore, migration,
enrollment or native-session results.

A read-only post-uninstall ACL inspection found that the bare-MSI customer state
directory retained inherited standard-user write access. Earlier company setup
applied an ACL, but bare installation/removal also needs an independent trust
boundary. No unauthorized session or privilege escalation was demonstrated.
This is nevertheless a release-blocking permissions finding: cleanup must not
execute mutable scripts from a directory that another user can populate, and
setup must not adopt an untrusted existing state directory merely by changing
its ACL.

The fix adds atomic private-directory creation and owner/ACL/reparse inspection
before MSI folder creation/service startup and before explicit setup reads saved
trust. It rejects unsafe existing objects unchanged and checks for existing
writable directory handles. Both MSIs use embedded agent copies for cleanup;
the agent executes its embedded PowerShell helpers directly instead of writing
and executing mutable helper files beside configuration data. All four MSI
authoring checks, five source-archive checks and entry-point/commit fixtures pass
locally. The directory helper's real administrator filesystem cases passed on
Server 2019, including private creation, trusted reuse, rejection of a foreign
file owner and rejection of a writable existing directory without changing it.
The first harness run correctly rejected unsafe state but Windows PowerShell
treated its redirected stderr as a terminating native error; the harness now
observes bounded child exit codes directly. Compilation/CI and new signed MSI
positive and standard-user negative tests were pending at that checkpoint.
Existing company services and their enrollment were not modified by these changes.

### Hardened MSI clean Server 2022 lifecycle, 21 October

Component source `650361413543baf7ef150f5960544154ec77ddeb` passed Windows/Linux
component CI and Linux deployment CI. The downloaded Windows artifact's SHA-256
was `33e13a86ad51ae5a1d094ad31e0256f7d1a3644df97c2efb9298a2c14fdf340a`;
independent inspection matched all 118 component source files against public Git.
This verifies source provenance, not bit-for-bit binary reproducibility.

Private test version `1.5.11` combined that agent with the previously verified
native `c0bf4b40ff8257c9fb69e4c5f6aceb2394f1adc0` inputs. Both MSIs were signed
and timestamp-verified with the disposable lab publisher. Signed package hashes:

- Customer: `ef55865a3c6db2a8411aa686e77b863588f7cb7419f4494282a8d7cfbfb0362c`.
- Technician: `eab4e4e6f872d0bc986e0feb0a28e6bb3fe98b78b669403dc726cdbbdba9d3ae`.
- Agent: `2fd0bcc199741a6ce2ffad4c5249ed47705a521ced32ddc157f22378f4db06e1`.

A fresh Windows Server 2022 Standard Evaluation Desktop Experience guest,
`20348.587`, was provisioned from Microsoft evaluation media and saved as an
offline clean OS checkpoint before product installation. The lab publisher was
trusted only inside that guest. Root-store import succeeded; the additional
Trusted Publisher import returned access denied. Both Authenticode signatures
were independently observed as valid without that additional store entry; the
unchanged package verifier still required valid signatures and the exact
publisher, certificate and package hashes. No Windows trust policy was relaxed.

Customer and technician installation, repair and uninstall all passed as the
guest administrator: six operations returned 0 without requiring reboot. The
pinned harness verified installed payload hashes, registration, customer service
identity/running state and removal. Independent inspection of all six raw MSI
logs found successful completion and no `Return value 3`. Private evidence is
retained in `windows-server2022-results-c0bf-6503614/verified-lifecycle.json`
alongside the phase reports and raw logs; machine credentials remain outside Git.

The proposed standard-account read/write/create denial test was blocked by
automatic approval review before execution, with reason `blocked by policy`;
specific user approval is pending. Actual compiled-MSI negative tests, repeated
uninstall rollback with this hardened agent, enrollment, approved automatic
updates and native remote sessions remain unverified. These six lifecycle passes
do not complete the Server 2022 acceptance matrix or qualify a production release.

### Hardened MSI rollback on Server 2022, 21 October

The same native `c0bf4b4` and hardened agent `6503614` were used in private
`1.5.12` uninstall fault fixtures, explicitly named `LAB ROLLBACK FIXTURE`.
They add an opt-in deferred failure after `RemoveFiles`; no distributable
installer or production behavior was changed. The signed fixture hashes were
`f614883f22eecc70055a351308eb3e688af58af6508b68372a16d45974ba5d30`
(customer) and
`024fbf1ab80b778af8ad098c1e214817752d234e4d7968be96e08310a9c99eb3`
(technician), using the disposable lab publisher only.

On the Server 2022 `20348.587` guest after the preceding lifecycle tests, both
editions installed and repaired, the injected uninstall returned the expected
1603, subsequent repair succeeded, and normal uninstall committed. Ten operations
matched their expected results, including exactly two deliberate failures.
Independent raw-log checks proved actual file removal preceded each injected
error. Full payload hashes and installed registration were restored; cancellation
markers survived; temporary rollback records cleared. The customer's owned,
disabled SYSTEM watcher was restored with its exact action and disabled state,
then removed during successful final uninstall. Evidence is retained privately
in `windows-server2022-rollback-6503614/verified-rollback.json` with all phase
reports and logs. This verifies MSI transaction recovery, not interrupted
automatic updates, company database recovery, native sessions or standard-user
file-access denial.

### Maintained Linux transport integration, 21 October

Source `143cd645f50de787edde0e3eb67027b84015fe35`, workflow run `39664079574`,
passed Windows/Linux components and the Linux deployment job `119444360250`.
The new candidate image used the pinned public transport and shared-submodule
sources, reviewed patches and Rust toolchain. Its independent public recipe
revision was `6503614`; the build archived 140 exact source/recipe files inside
the image. Source archive SHA-256 was
`a0d56be100851d183eab986533fe9028c2c8a52c8bd31691f92facca14d2cb68`.

The real Docker rehearsal passed with the maintained image's immutable ID:
HTTPS trust validation/untrusted-CA rejection, profile signatures, MFA,
enrollment/logout, local NAT-test protocol responses and complete encrypted
management/transport/TLS recovery into replacement containers retaining original
trust. Both the previous upstream stack and maintained stack passed. These
checks do not prove native desktop sessions, UDP NAT traversal, public ACME,
Internet direct/relay access or production readiness. The explicit candidate
Compose override supports fresh isolated deployments; normal distribution pins
remain unchanged pending those acceptance checks. A subsequent change exports
candidate binaries and corresponding source as a retained CI artifact; that
new export step subsequently passed CI, but independent source inspection found
the archive discrepancy described below.

### Windows server state creation hardening, 21 October

Inspection found that fresh Windows server setup created its ProgramData folder
before restricting access. A source change replaces that interval with atomic
private creation through the signed, reverified management copy in Program Files.
The shared embedded directory helper now has a canonical server scope and an
explicit reject-existing mode, including a directory that appears after the
initial precheck. Existing state is not adopted by rewriting permissions. Server
preparation has a 30-second deadline; a matching management build is required.
Current management also rejects unknown commands before opening data or starting
a listener. No exploit or unauthorized session was demonstrated by this review.

Four MSI-authoring checks, five source-archive checks and PowerShell parsing
passed locally. Actual administrator filesystem tests on the Server 2022 lab
passed private creation, trusted reuse, reject-existing without changing its
ACL, rejection of foreign-owned content and writable existing directories.
The first run exposed a PowerShell case-insensitive variable-name collision in
the new helper parameter; that was corrected before the successful rerun. These
tests used temporary private fixture paths, not the live company server state.
Source `8f4c4ef51710c0749dd7b86d02e89b4738fe7734`, Windows CI job `119454688771`,
subsequently passed the compiled management command test: canonical private
creation succeeded, a noncanonical path was rejected, and a second invocation
refused existing state without changing its ACL or starting a server. The helper
administrator filesystem cases also passed there. Full clean-VM server setup
remains unverified. Existing company services were not upgraded or changed.

### Independent maintained transport source discrepancy, 21 October

Source `2e704f2` passed component/deployment CI, including artifact export and
the maintained stack rehearsal. Artifact `12378445188` (6,338,877 bytes) was
downloaded and its GitHub SHA-256
`823e68c9189266a03b8e3ddc8fdfbcf2a0c8c1baad8556ce085be51a56164d5b`
verified. All six files matched the export manifest, and both executables had
Linux x86-64 ELF headers. This did not prove every source file matched Git.

Independent comparison against immutable upstream/project Git objects rejected
the tracked `db_v2.sqlite3`: its archive SHA-256 was
`a269b1bcb604d32ef120ce8d8e4fd3ec2b58ddb1c9fa24445f256826a1dc7dbd`,
whereas the public Git blob was
`071be773254bb18a6ec9b60bac0d000d78a75f2b2c783d13aeff9e81462f37f0`.
Both were 24,576-byte SQLite files. Source capture had run after compilation,
which uses compile-time SQLx queries against the schema database. The artifact
is therefore not accepted as an exact copy of the pinned source inputs.

The Docker and standalone transport workflows now capture source before the
rendezvous/relay build. A regenerated artifact must pass the independent
whole-archive Git comparison before its source-provenance gate can close. The
existing integration results remain limited to their recorded runtime scope;
they do not override this failed source check or qualify production publication.

### Corrected transport source verification and Linux reproduction, 21 October

Source `adf437ee181bd3f1455df14e07d06b4a2b8b50a1`, workflow `39668437572`,
passed the component/deployment checks with source captured before compilation.
Artifact `12378089575` (6,338,856 bytes) was downloaded and verified against
GitHub's SHA-256
`516bdc2236271ebdcc440a9e2da5e5c14b162097070c80a4631519b8122bbcf2`.
Independent whole-archive comparison now passes all 140 files against immutable
upstream/shared/project Git objects, except the UDP file whose complete content
matches the explicitly reviewed patch hash. The previously mismatched SQLite
file now matches its public Git blob. This closes the recorded source-input
discrepancy for this artifact; it does not validate the earlier archive.

Actual unsigned Linux executables from the separate `39665690071` and
`39668437572` builds also match byte-for-byte:

- `hbbs`: `6bacfa973533248aa24c2917469149d3e08246140f5423160a6da521c3dd018a`.
- `hbbr`: `132ab0c989a110a3335220d1afbbdf022afa559cf9bec23b8419f473d04388a3`.

This is reproduction evidence for those two executables under the common
recorded transport recipe and base-image/toolchain pins. It does not establish
bit-identical OCI images, Windows payloads or the entire product. Independent
receipts remain in `maintained-transport-adf437e/published-source-verification.json`
and `verified-binary-reproduction.json` in the private lab evidence.

Source `f955874c4b777e58fa708401cc71abf52f868f28`, run `39670081266`, subsequently
passed Windows/Linux components and the maintained deployment rehearsal with
additional input guards. Source capture rejects unexpected tracked changes and
accepts only the hash-verified UDP modification. A fresh local clone test
archived all 140 expected files successfully, then deliberately changed a byte
in the public schema fixture and confirmed rejection before any negative archive
was written. No company database was used in this guard test.

### Actual insecure-existing-state MSI rejection, 21 October

The private lab-signed `1.5.11` customer MSI (native `c0bf4b4`, agent `6503614`)
was run on Server 2019 `17763.3650` against the original post-uninstall state
folder with inherited Users-write permissions. No replacement folder or new
standard account was created. Installation correctly returned 1603. Independent
raw-log inspection confirmed `PrepareCustomerState` failed before any payload
copy; the existing cancellation marker, old helper bytes and all ACLs remained
unchanged. No product registration, application, agent or service remained.
Evidence is retained in `windows-server2019-unsafe-state-6503614/verified-rejection.json`
with the original metadata and MSI log. This proves refusal to adopt insecure
existing state, not the separately blocked standard-user read/write/create test.

### Clean Windows 10 LTSC installer lifecycle, 21 October

Microsoft's Windows 10 Enterprise LTSC 2021 Evaluation x64 media was verified
against its published SHA-256 before installation. The installed guest is
`19044.1288`, edition `EnterpriseSEval`. A guarded preflight confirmed no Swan
product registrations, services or customer state and disabled provisioning
autologon. After normal shutdown, an offline internal clean OS checkpoint passed
disk-integrity checks before both editions were tested.

The same private lab-signed `1.5.11` packages used on Server 2022 (native
`c0bf4b4`, agent `6503614`) passed customer and technician install, repair and
uninstall: six operations, all returning 0 without a requested reboot. Independent
verification checked each raw MSI log, package and installed payload hashes,
product registration, the customer's SYSTEM service and removal assertions.
Evidence is retained in `windows10-results-c0bf-6503614/verified-lifecycle.json`
alongside the original reports and logs.

The initial test-driver launch was refused by Windows 10's default PowerShell
execution policy before any product installation. The reviewed, hash-pinned
driver then ran in a separate process with an explicit process-only policy.
MachinePolicy and UserPolicy were undefined; all five parent policy scopes were
identical before and after the successful run. The initial failure is preserved
in the private lab diagnostics. No persistent execution-policy setting changed.

This proves the administrator MSI lifecycle on this Windows 10 LTSC build only.
It does not establish other Windows 10 editions/builds, standard-user technician
operation, enrollment, native sessions, installed-application reboot behavior,
upgrades or automatic updates. The VM's clock was independently checked against
the host before the installer run; the future RTC boot-setting correction still
requires its own reboot verification. Production signing remains unqualified.

### Native standard-user technician GUI and live branding, 22 October

The private lab-signed technician portable from native source
`c0bf4b40ff8257c9fb69e4c5f6aceb2394f1adc0` ran as the existing non-administrator
lab user on Server 2016 Standard Evaluation Desktop Experience `14393.693`.
Its outer executable matched SHA-256
`6aa3c048827e2ad8e4414cc0c09476be9dff9bc8ad02fc46bcfc207d843eda40`.
All 95 extracted native/Flutter payload files matched the pinned CI manifest,
including the actual running executable, libraries and `app.so`. The portable
uses the user's shared extraction cache; its managed state was isolated from
the installed technician state, which remained byte-for-byte unchanged.

Live console observations confirmed company name, signed SVG logo, support link
and product/license attribution, then successful graphical password/TOTP login,
the authorized approved-device inventory and session history. Requesting attended
support reached the existing customer's approval prompt. The technician waited
without a desktop view. Cancelling on the customer side produced "Closed manually
by the peer" and an ended history entry. Independent server history confirmed
the corresponding target-bound grant was claimed, attended and closed. This
checks refusal, not an accepted desktop or separate-machine session. The existing
customer remained the older `bd13500` installation.

An administrator published only a temporary technician display-name change at
profile revision 17. The open app displayed it through its normal periodic
refresh, without a manual refresh click, process restart or reinstall. Restoring
the original name published revision 18, which appeared automatically in the
same process. Independent Ed25519 verification checked revisions 16, 17 and 18
under the original pinned public key; other policy, transport and trust fields
matched the baseline. Software sequence and local consent fields were preserved.
Graphical logout cleared inventory/history, and its server audit event matched
the technician account. Password and TOTP secret were absent from managed state.

The app then closed cleanly and restarted with an unreachable proxy confined to
that test process's environment. An agent probe first synchronized successfully
online, then returned a proxy-tunnel connection refusal with that environment.
After restart the native GUI retained cached company branding, but a fresh
credential/TOTP login attempt failed and cleared the password/code fields. The
company HTTPS service remained reachable outside the isolated process, and its
audit contained no successful technician login during the offline run. All 95
running payload files were checked again. Both app runs closed cleanly; the
company and customer services remained running. No firewall, listener, service
installation or transport process was changed.

Private evidence, raw audit/history, signed state snapshots, proxy diagnostics
and independent verification are retained in
`technician-c0bf-gui-20261021/verified-gui-acceptance.json` and its `evidence/`
directory. UI observations were made through the live VM console and are
corroborated by those runtime records; they are not a substitute for the remaining
protocol and Windows-matrix tests. This isolated portable fixture used copied
public technician configuration and a lab publisher. Complete company bundle
installation, accepted remote control, cross-machine/Internet/relay operation,
secure-desktop behavior, key rotation and approved automatic updates remain open.

### First-run branding and policy collection, 22 October

The shared web setup page now collects customer/technician names, colors, SVG
logos, support contacts, website shortcuts and consent wording, plus unattended
support eligibility, update pause/channel/cohort and UTC maintenance settings.
Previously these controls were available only after company creation, so initial
setup silently used product defaults. Initial setup and subsequent profile edits
now use the same field renderer and collector. This shared page is used after
both Windows installation and Linux first-run deployment. Initial device approval
remains mandatory; allowing unattended support does not grant customer consent.

For PR head `066c6d91b0f35f304f1a546c6f3ebeceb79cfc05`, component workflow
`39686752599` passed Windows, Linux and both Linux deployment rehearsals.
The tested merge checkout was `272f9da7287f9428d5db86366f6f74c44c0c808a`.
Windows job `119522655784` and Linux job `119522655449` explicitly passed
`first_run_preserves_company_branding_and_update_policy_atomically` and the new
front-end form contracts. The real router/SQLite test verifies that invalid
initial policy creates no company, account or setup audit entry; a valid request
persists every supplied branding/policy field and returns a verifiable signed
profile with a server-assigned company identity. The form contract also checks
subsequent editing and rejection of incomplete shortcuts before submission. A
negative control using the preceding public source correctly detects the missing
first-run branding fields.

A static browser preview verified the new controls and exposed a collapsed color
swatch caused by generic input padding. The swatch layout was corrected and
visually rechecked. This preview used public sample data, could not submit setup,
and was closed along with its temporary loopback static-file process. No existing
company deployment was changed. These checks establish the new form/API behavior,
not a completed fresh Windows service installation, generated company bundles,
production signing or the remaining full-product acceptance gates.

### Stock client legacy-password rejection, 22 October

A separate, Authenticode-valid PURSLANE RustDesk 1.5.0+68 control
(`7f6d1e09e5a9924f567962aa57a6b126b409bed4775d98074c727420438768dc`)
ran as the existing non-administrator lab technician on Server 2016
14393.693. Its independently probed ID was 880000042, absent from company
enrollment. With only public transport configuration and a deliberately invalid
legacy password, it connected to approved customer 1193046 and displayed
“Company session authorization denied.” No desktop or customer consent prompt
appeared; management audit history contained zero session requests since the
attempt began. This is an observed native protocol denial, not a simulated UI.

The receiving customer was the older `bd13500` build
(`60d15651c583537bcaaf25df2b838ca594f5855b2d2b0f416b690645ef107cfc`).
This same-VM lab result does not validate the newer receiver, direct-IP paths,
all permissions, relay selection, or public Internet access. Port 21118 belonged
to hbbs rather than the customer; it was not used as a customer-listener proof.
No separate receiver-log denial was established. The control exited normally
with code 0, its scheduled task returned Ready/result 0, and the process was
absent. Customer and company services remained running and HTTPS returned 200.
Local raw evidence is retained under the private lab `legacy-control-1.5.0-build68`
directory; credentials and generated client private keys are excluded from this
repository. Host RustDesk services were not stopped or reconfigured.

The subsequent receiver change adds a fixed local warning for rejected managed
login attempts, identified only by local connection number. Credentials and
underlying error details are deliberately excluded. Compilation and runtime log
verification of this new change remain pending; the stock-client observation
above predates it and does not prove the new warning is emitted.

### Receiver system-control permission correction, 22 October

Source inspection found that the virtual-monitor handler could reach the
monitor driver without checking keyboard/control permission. Elevation and
resolution changes already had handler checks. Managed receiving connections
now filter virtual-monitor on/off, elevation, resolution and Windows desktop
session-switch requests before dispatch whenever either the company grant or
customer's current keyboard/control permission denies control. Existing monitor
selection, capture and refresh remain available to view-only sessions.

A native regression test serializes and parses the control packets, checking
rejection with control disabled and acceptance with control enabled, and checks
that ordinary viewing packets remain allowed. The Windows build workflow runs
this test explicitly and requires one executed passing test. Compilation,
test results and real driver/session behavior remain pending. This is a source
correction, not a claim of completed packet-injection acceptance or an updated
installed customer app.

### Native tampered-cache check and legacy-install UI correction, 22 October

The existing lab-signed technician `c0bf4b4` portable ran as standard user
SwanLabTech on Server 2016 14393.693 with a separate cached-state fixture.
Only the decoded technician display name was changed, retaining the original
signature. Independent Ed25519 verification accepted the original envelope and
rejected the modified one. The GUI showed “Company setup is not complete,” no
company branding, no technician login or device inventory. The untrusted name
did not appear. It closed normally (exit/task result 0); the native process was
absent, both working services stayed running, and the source fixture hash was
unchanged. Local evidence/verifier are under `technician-c0bf-gui-20261021` and
`Verify-Lab16C0bfTamper20261022.py`. Network isolation used only the test process's
proxy environment; no firewall or server settings changed. This verifies local
cached-profile rejection, not hostile-HTTPS sync or the other profile negatives.

The test also exposed a legacy “Click to upgrade” card because another version
was installed in the guest. The button was not clicked. Source now hides legacy
installation/download cards for both stable managed application identities,
including an invalid company configuration. Managed UI installation entry points
and the legacy `--update` command refuse the operation; the company agent and
explicit bundle's silent-installer path remain available. Compilation and native
verification of this correction remain pending. The previously dispatched
`3c4fb57` build predates this correction and cannot validate it.

### Verified current native build and corresponding source, 22 October

Windows workflow `39690742314` passed at source
`becf3cd861275e5100e2b86c3c26ad53be18cd00`. Raw job logs confirm both exact
native regressions executed with one pass and zero failures each:
`managed_sync_rejects_legacy_endpoint_and_unsigned_options` and
`managed_view_only_packets_cannot_change_system_control`. Their scope is the
legacy-policy guard and parsed packet permission classification, not a live
malicious-peer or monitor-driver test.

Artifact `12390656897` is 146,651,249 bytes, SHA-256
`cf0109e9b9ccc7366fc6995228855e66a198fc8c932d998b9d09d1d4ce05a6bd`.
The first transfer failed with ChunkedEncodingError. After that process exited,
127,926,272 retained bytes were resumed using an exact validated HTTP 206 range.
The entire final archive passed the authoritative size/digest and all 24 entry
CRC/path/payload-checksum checks. The corresponding source archive matches
1,122 public tracked files/submodule objects and all six separately verified
CI-generated bridge files. All 95 extracted native/Flutter payload files match
the installed manifest; the native EXE hash is
`5bdfcbf7d464511c51470e4a773ea583a853c2124cb4d83ab644acd0782d58e8`.
No downloaded executable was launched during these checks.

The independently checked MSI evidence artifact `12390213470` is 34,872 bytes,
SHA-256 `84ffe55bfa20fea974c6c3f0aad662fa103076007a298ffb2ed5c18011721531`.
All three original installer logs report successful completion without a failed
action, and their source/package invocations agree. The downloaded technician
MSI hash `a6ad6b59af1e953e84239fdc5eb1ad6d07fdcdc5b819adc6c85b2cb1c60479b3`
matches the package actually installed, repaired and uninstalled under the CI
account on Windows Server 2022 Datacenter build 20348. This is an unsigned
CI-account lifecycle, not clean-machine, standard-user, enrollment, signature
acceptance or remote-session verification. Private lab signing and latest-native
GUI/receiver verification remain next; production signing, full reproducibility
and the complete acceptance gates remain open.

### Current lab signing and tampered-cache GUI regression, 22 October

Five verified `becf3cd` inputs were copied to a separate private directory in
Server 2016 and signed using its existing nonexportable disposable lab key.
Each Authenticode signature and RFC3161 timestamp passed `/pa /all /tw`, pinned
to certificate DER SHA-256
`d5d4567bf67622ae8bda75e22db7b72b278c0eaa58450427bf8bf0e0bba3f94e`.
The signed technician portable hash is
`db903307fde776513ea1830a97843c23414b88e58239e213d11830a042135931`;
the signed agent hash is
`f31d217bacdc6a3234c417d8acb9e96abe7ac75541e106b5999df2d54d72b4b8`.
All original input hashes were preserved. This is private lab signing only;
none of the company server, worker or installed customer binaries was replaced.

The new portable ran as the existing non-administrator SwanLabTech in desktop
session 1, with a new public-only technician state fixture. Independent signature
verification accepted the untouched original profile and rejected the fixture
whose only decoded change was the technician display name. All 95 actual
running native/Flutter files matched the verified CI manifest, including native
EXE `5bdfcbf7d464511c51470e4a773ea583a853c2124cb4d83ab644acd0782d58e8`.
A strict fixture path comparison initially rejected the executable's `\.\`
segment; canonicalizing that observed path resolved the harness mismatch without
changing the app or relaxing the expected extraction root.

The GUI showed “Company setup is not complete,” with no forged branding,
technician login, or device inventory. The legacy install/upgrade card exposed
by the older build was absent. Closing this fallback incoming window kept the
native process alive in the background. After independently checking its PID,
path, hash, owner and desktop, only that test process was force-stopped. The
first cleanup command stalled at cross-user confirmation; its exact pending
PowerShell process was cancelled before explicit forced cleanup. The launcher
then returned 0 and the task became Ready, with no native test process left.
This is **not** a normal native-exit acceptance result. No VM reboot or existing
service stop was used. The installed customer hash and both running services,
original public profile, and unsigned build inputs were checked again afterward
and preserved.

Local evidence is in `candidate-becf3cd-inputs/signing-evidence` and
`technician-becf-tamper-evidence-20261022/verified-runtime.json`, with independent
verifiers retained outside the repository. This establishes the current local
corrupted-cache/UI regression only. Ordinary technician login, legacy command
refusal, hostile-HTTPS profile sync, corrected receiving-device permissions,
automatic updates, clean-machine installers and the full release gates remain
open.

### Live legacy update command refusal, 22 October

The actual `becf3cd` native EXE ran with `--update` as the existing standard-user
SwanLabTech in Server 2016 desktop session 1. A separate fixture contained the
unaltered signed public technician profile. All 95 native/Flutter files were
verified against the CI manifest before launch, and the reviewed launcher hash
was checked before its limited interactive task was registered.

The command exited normally with code 0. A newly changed original native log
contained the exact warning “Managed updates require the approved company update
agent.” The collected raw log hash matched its receipt. The installed customer
EXE hash remained
`60d15651c583537bcaaf25df2b838ca594f5855b2d2b0f416b690645ef107cfc`, and
the public test profile hash remained
`1fc8663de5b64b350de940b15067d6c5bc2ae9ab8b3f07d52111462e5c19776e`.
A post-test check found the recorded native process absent, task Ready/result 0,
original source profile unchanged and customer/company services Running. No
forced cleanup, UAC acceptance, service stop or package installation was used.

The exact refusal marker, raw log, source/payload identity and before/after hashes
are independently checked in the private
`technician-becf-legacy-update-evidence-20261022/verified-command.json` evidence.
This is runtime evidence that the legacy native command refuses the managed
update path, including its observed zero exit code. It does not verify approved
automatic updates, GUI installation callbacks, publisher checks, interruption
recovery, or the full release criteria.

### Current technician MSI standard-user diagnostic, 22 October

The exact public `becf3cd861275e5100e2b86c3c26ad53be18cd00` packaging
recipe was extracted from its verified source archive and used with WiX 5.0.2.
Its unchanged project binaries were signed with the existing disposable company
lab publisher. The resulting technician MSI version 1.5.0 hash was
`61be6e524adbfbe164a47281cad04fe033debcf3e033d334c2e657946cc191f6`.
This was a manually prepared private test package, not a live company worker job
or a production-signed release. Host certificate trust was not changed.

On Windows 10 Enterprise LTSC Evaluation 19044.1288, a newly created standard
user logged on to the desktop. A limited interactive task installed the MSI,
moved only its newly installed and hash-checked native executable into the
owned evidence directory, repaired the missing executable, and uninstalled.
All three Windows Installer operations returned 0. Independent checks verified
the original verbose log hashes, the repair log's missing-file entry, restored
payload signatures/hashes, and removal of payload files and per-user
registration. No administrator token, UAC approval or persistent execution-policy
change was used. The reviewed runner used process-scoped Bypass only; all other
execution-policy scopes remained Undefined.

The OS displayed an expired evaluation watermark. Its licensing provider
reported status 5 and zero grace. One normal Microsoft activation attempt failed
with a general activation error and left that state unchanged. No product key,
KMS configuration, evaluation rearm or clock change was used to bypass expiry.
The results therefore establish a functional installer diagnostic only, not a
qualified Windows 10 release. A valid Windows 10 test environment remains required.
This run does not establish company enrollment, technician GUI authentication,
native sessions, upgrades or automatic updates.

The guest's final check completed normally over SSH at 00:57:09 UTC: task
Ready/result 0, payload and user registration removed, OS restart pending.
Its stdout was preserved as an explicitly labeled observation; the separate
original JSON transfer subsequently failed with a closed connection. The VM
stopped answering management connections and showed a black screen after an OS
update restart had been scheduled. A single normal ACPI shutdown request was
sent at 01:21:19 UTC; completed shutdown is not established. Passive QEMU disk
counters at 01:24:04 and 01:25:01 UTC recorded 610,982,912 additional written bytes
and 10,217 additional write operations. This establishes ongoing disk activity,
not successful Windows update/reboot or the cause of the black screen. The guest
has not been forcibly powered off.

Private evidence is in
`windows10-standard-becf3cd-results/verified-diagnostic.json`, including the
three original MSI logs and their receipts, runner/task identity, licensing
result, activation failure, and labeled final-check stdout observation.
The report explicitly records `full_windows_qualification: false`. Separate
receipts retain the first final-check transfer failure and subsequent recovery;
the labeled stdout observation is not presented as an original downloaded file.

After Windows update processing, the guest returned to its sign-in screen and
SSH recovered. The original final-check file was then downloaded successfully:
SHA-256 `ddec9a7cfa33749ca3df61f1904390147b8c9a2a264d615c0fcb8d4f12728390`
matched the guest's hash and the previously preserved stdout contents. The
independent diagnostic verifier now uses that original file. A separate
post-update check recorded build `19044.7725`, no OS restart pending, task
Ready/result 0, and the technician payload still removed. Windows licensing
remained status 5/zero grace. These later observations do not establish MSI
qualification on the updated OS; the three installer operations ran on
`19044.1288`. The guest acknowledged a normal shutdown request, QEMU exited 0,
and only its disconnected viewer was closed. No forced power-off was used.

A separate boot of this updated guest checked normal Microsoft activation once
more after the OS update. The existing evaluation's activation attempt at
03:20:52 UTC failed again, leaving license status 5, zero grace and reason
`0xC004F009`. No keys, KMS configuration, clock or rearm state were changed.
The collected local activation receipt has SHA-256
`06bf83d0307684e8cb27d2ed5c37a5a1005a68c8e770c3ecd57ef298bd637bc9`.
The guest acknowledged normal shutdown and its QEMU process exited 0. No
installer operations were repeated during this licensing check. A valid
Windows 10 test installation remains required for final qualification.

### View-only Windows session acknowledgement correction, 22 October

Source inspection after the `becf3cd` diagnostic found that the managed control
filter rejected every `SelectedSid` packet when keyboard/control permission was
absent. The client sends this packet to acknowledge the current Windows session;
the receiver then starts monitor/camera subscriptions or completes delayed
file-directory reads. Rejecting that acknowledgement could prevent legitimate
view-only or file-only sessions from initializing on hosts with session selection.

The receiver now compares the requested session with its own OS-reported process
session ID. Acknowledging that same session is allowed; selecting a different
session still requires both receiver control permission and the signed grant's
keyboard permission. An unavailable OS session ID fails closed. Other system
control packets retain their existing permission checks. The wire-message
regression test now covers same-session acknowledgement, another-session denial
and failed OS lookup, alongside elevation, resolution and virtual-display checks.
Native workflow `39704802586` is building exact correction source
`ebef5a610cd2b020a02363aa218a401f771fdad7`. Both bridge-generation jobs passed;
the downloaded six-file bridge artifact `12394472703` passed provenance,
size, CRC and SHA-256 checks (archive
`c3629a2fcf5984eb414fddd6806211affc6da1341313e9fe42ca61f364df799b`).
The completed native build passed compilation, both exact managed regression
tests and the unsigned technician MSI lifecycle under the CI account. Raw job
`119583130305` logs show each exact regression test ran once and passed with
zero failures. In particular, the control test covers acknowledging the current
Windows session without keyboard permission while denying another session or
an unknown current session. Receiving-device runtime acceptance remains pending.

The downloaded native artifact `12396855453` was 146,667,282 bytes and matched
SHA-256 `479ee43e8bfa6ccbb78b212f567b009776e8427f5e3a8083ff13d585d11b1bc8`.
All 24 entries passed path/CRC/published-checksum verification. The published
source archive matched 1,122 tracked public Git files and all six separately
verified generated bridge files. All 95 extracted native payload files matched
the CI installed-file manifest; the inner unsigned native EXE hash was
`33925b581e6b118a87c095dc89778767d0dd1e5dc9738abc4aa87ab64c109ec7`.
No executable was launched during these static checks.

MSI evidence artifact `12396935044` matched archive SHA-256
`bcdf87b1ef308a7c6495ab228f1f5cd018db6f184b7a2ca85c2287c41a72244c`.
Its three original Windows Installer logs report success and no Return value 3.
The tested package SHA-256
`7c31ea534bec14519be7c30ffb31eb86b1b0878d23e1353f1432de7da1559595`
matches the downloaded technician MSI. This is unsigned lifecycle evidence under
the Windows Server 2022 CI account, not standard-user enrollment, clean-machine
qualification, signature verification or a new native receiving session.
Private verification receipts are in `native-ebef5a6`,
`customer-payload-ebef5a6.verified.json` and `technician-msi-ci-ebef5a6`.

### Independent patched engine source build, 22 October

The engine reconstruction recipe added in `b79d57f` pins engine source
`db49896cf25ceabc44096d5f088d86414e05a7aa`, depot tools
`580b4ff3f5cd0dcaa2eacda28cefe0f45320e8f7`, the original DEPS hash and
original/patched hashes for exactly two source files. A real sparse checkout of
that public engine commit passed the patch check. Changed DEPS, either changed
source file, a wrong revision and attempting to patch the already modified
checkout were all rejected. These checks establish source/patch validation,
not engine compilation, binary equivalence or runtime acceptance.

Workflow `39707585793` failed before checkout because the pinned depot tools'
normal Windows bootstrap had been skipped when automatic updates were disabled.
The explicit pinned bootstrap fixed that failure. Workflow `39707908369` then
reached dependency checkout but failed fetching tinygltf commit
`9bb5806df4055ac973b970ba5b3e27ce27d98148` from the old Flutter mirror.
That mirror's commit endpoint returned 404; the declared canonical upstream
repository exposes the exact commit. Source `670e2ac` adds a single explicit
repository override retaining that original DEPS revision, leaves DEPS bytes
unchanged, and requires both the actual dependency revision and repository URL
to match before packaging. Workflow `39708643178` is testing that source.

That run subsequently passed dependency checkout and the reviewed source patch,
generated its GN build, and reached C++ compilation. It failed because the
runner's Microsoft C++ 14.44.35207 library requires Clang 19 or newer while the
engine dependency recipe pins an older Clang. A compatible pinned Microsoft
toolset still needs to be selected and tested; the compiler compatibility check
was not disabled. There is no successful source-built engine artifact yet.

The next recipe pins MSVC tools `14.38.33130` and SDK `10.0.22621.0`, explicitly
selects the older toolset through a hash-checked patch to pinned buildroot
`e265c359126b24351f534080fb22edaa159f2215`, and verifies the generated compile/link
commands before compilation and packaging. Three local guard tests passed,
including mixed/wrong toolsets, SDK drift, changed patch/dependency pins and
compiler/STL compatibility suppression. A real sparse checkout of the public
buildroot commit accepted the reviewed patch and rejected changed source,
a wrong revision and repeated patching. These are provenance/guard checks;
successful engine compilation and runtime qualification are still required.

Source `5a5e4f2` ran in workflow `39711188710`. Its three provenance guard
tests, side-by-side Microsoft toolset provisioning and generated-command checks
passed on the Windows runner. All 4,900 compile/build steps reached the final
DLL link, which failed with LNK1104 because `atls.lib` was missing. The compiler
compatibility failure is resolved, but this is not a successful engine build.
Diagnostic artifact `12398238710` retains the actual toolchain, dependency
revisions and generated commands (archive SHA-256
`230ced36cdfddb7be630dc84d45709c50c63affd301fbb59e4b09e98b500a637`).
Source `9f97ef0` adds the matching MSVC 14.38 ATL component, checks its headers
and library before building, and records its library hash. Workflow
`39716739857` passed the provenance tests and pinned toolchain/ATL installation
and entered the source-build step. This is an in-progress observation, not a
successful build result. Engine linking,
artifact verification, independent rebuild comparison and runtime acceptance
remain pending.

The earlier failed build's downloaded diagnostics passed their published archive
hash and CRC checks. Inspection found no `/Brepro` in its generated compiler
commands. The final link command delegates flags to a response file, so its
command listing alone does not establish the effective linker flags. Following
the LLVM guidance linked in the source document, the next recipe adds release
COFF/PE timestamp controls, relative debug paths and a stable PDB basename. It
retains the final linker response file and rejects missing timestamp/path
controls or enabled incremental linking. Four local guard tests pass; a separate
checkout of the exact public buildroot accepted the expanded hash-checked patch
and rejected both a repeated patch and changed compiler source. These are source
and guard checks only. The patched engine has not compiled or demonstrated
reproducible bytes yet; the existing `9f97ef0` build is left running unchanged.

The local source-verification fixture also matched complete public Git blob/mode
listings against pinned Git tree objects: 5,604 engine files under tree
`934b9064b5bc6814984734eb223acc23dfc42ef8` and 145 buildroot files under tree
`22cdad9ffd0a6d8d91256d89d19f4ed338aa4786`. The artifact collector is prepared
to compare its original-source archives against every one of these files after
a successful build. Preparing this check does not count as a passing artifact.

Engine results remain separate candidates with explicit unverified binary
rebuild/runtime/production flags. The application workflow still uses its
previously reviewed engine archive hash. No source-built engine has been
promoted or counted as a reproducibility pass.

### Current technician GUI and approved desktop smoke test, 22 October

Five verified `ebef5a6` build inputs were signed and timestamped with the existing
non-exportable disposable lab publisher. Original inputs, installed customer
hash and running customer/company services were preserved. Signing receipts
identify each public build input and signed output; this is private test signing,
not production signing or a live company worker publication.

The technician portable and agent were placed in a new fixture with only the
unchanged signed public technician profile at revision 18. A limited interactive
task launched it as existing standard user `SWAN-LAB-16V2\SwanLabTech` in session
1. The running native PID `1216` and all 95 extracted payload files matched the
verified CI manifest, including native EXE SHA-256
`33925b581e6b118a87c095dc89778767d0dd1e5dc9738abc4aa87ab64c109ec7`.
The portable's actual extraction cache was the user's default
`AppData\Local\rustdesk`; the managed state was in the new fixture. This does
not establish complete isolation of every native per-user configuration/cache.

The actual GUI displayed company branding and its HTTPS domain, accepted the
lab technician's password/TOTP login, and showed the authorized approved device
and session history. A support request produced the preserved customer's consent
prompt. Clicking Accept displayed an actual desktop capture stream. The session
window was then closed through its titlebar; a refreshed history showed Ended.
Sign out removed inventory/history and left password/authenticator fields empty.
The native process was absent after closing the signed-out app, with no forced
termination. The launcher exited 0; an independent native exit code was not captured.

Backend audit records scoped to this task confirm technician login, one session
request and logout. The matching server history row is claimed, closed and
customer-approved (`unattended: false`). Independent checks verified the public
profile's signature against the earlier trusted bootstrap, correlated the audit
target and GUI request timestamp, and matched original evidence hashes. The
provided lab password and MFA seed were absent from the 35 examined runtime text
files and the collected evidence. Original installed/source profiles and both
services remained unchanged/running.

Private evidence is in
`technician-ebef-gui-20261022/verified-gui-smoke.json` and
`candidate-ebef5a6-inputs/signing-evidence`. This test used the existing reviewed
engine, the older `bd13500` receiver and the same VM for both peers. It does not
qualify the source-built engine, new receiving guards, separate-machine input,
clipboard/transfer, public direct/relay, automatic updates, fresh company setup
or the complete Windows installation matrix.

### Independent company component binary comparisons, 22 October

Component build jobs in workflows `39716739878` and `39716743778` were compared
using the downloaded binary bytes, not just their declared hash lists. All four
platform-specific jobs completed successfully; the second workflow's separate
Linux transport job was cancelled, so the entire second workflow is not counted
as successful. The reviewed branch revision is `9f97ef0`; the pull-request job
recorded merge revision `01a344a99bb488d131714767b372f3ce65cdeab1`. Each artifact's
130 corresponding source files matched that reviewed public source, allowing
checkout line endings, and were byte-identical between the two builds on each
platform. Both builds recorded Rust `1.90.0 (1159e78c4 2025-09-14)`.

The Ubuntu 24.04 jobs `119620529061` and `119620541102` produced identical
unsigned component binaries:

| Component | SHA-256 from both independent jobs |
| --- | --- |
| Management | `d6dc94f8558b9bfd962bca8a98d84ce9e2cb1785cebc22f57305863eb7eaa393` |
| Agent | `1391e389a7e6cf3329fa42bd866be2b263df66fd05a6594c02ceac45fafe6f22` |
| Worker | `476c451519f240dec68eb5edbc105289df8d3f7af7772d5837676d91f8346c7c` |

Their GitHub artifacts `12399761385` and `12399437062` passed archive size,
SHA-256, CRC, declared payload hashes and complete corresponding-source checks.
The archive digests are respectively
`b6f4e4fce0bb32d031587b816ee9c63213fa136bf79c84748ef4abc290ce3ebf`
and `fe47e9f103a9d49693077c5ab2c3242ba447382e42311ca7d9c0e36c49f48261`.
This establishes these three component binaries on this CI platform only. It
neither qualifies Linux endpoint GUIs nor proves arbitrary-host, native Windows
app, installer, signing, container-image or complete-product reproducibility.

The Windows jobs `119620529016` and `119620540922` produced different bytes for
all three components. Their artifacts `12399696985` and `12399173780` passed the
same integrity/source checks, with archive digests
`963935d51ee58d10f680d8f22fdac2fda3d3cfc6422b50238c8dc183ab7ffac8`
and `a779c0e1b6f075a4a67e1d7a26eb2d6dc3eb9e8614ca1fdfef461873b6aa8a1e`.
Read-only PE inspection found different COFF/debug timestamps and CodeView
identifiers. Masking those fields in memory for diagnosis made each pair match;
the original artifacts were not modified, and this is not a reproducibility
pass. The Windows x64 Rust linker configuration now adds `/Brepro` while keeping
the static C runtime. The workflow also watches this configuration so a change
triggers verification. Corrected builds and an actual binary comparison remain
required; parsing the updated configuration is not a build result.

Local original archives and detailed evidence are retained under
`component-rebuild-comparison-9f97ef0`, including `comparison.json`, each
`verified-provenance.json`, and `windows-pe-differences.json`. No downloaded
component executable was run during this comparison, and no installed lab
service was replaced.

### Corrected Windows and Linux component comparisons, 22 October

The `/Brepro` correction at reviewed branch source `a413ab5` passed a new
independent comparison. Workflows `39719130453` and `39719134640` completed their
Windows/Linux verification and Linux deployment jobs successfully. The second
run used public PR merge commit `aab258b3db45ec5cfe19b5e7f4bb61b218612ae2`.
Its public parentage includes `a413ab5`, and its component workflow bytes match
the reviewed branch workflow (SHA-256
`3ff0b0c6c36d0855a09d157193d083715434c93653f889e8e6060367b45e1983`).
All 130 source files in each artifact matched the reviewed public source and
were byte-identical within each platform's pair. Both jobs on each platform
recorded identical Rust compiler identities.

Windows jobs `119628269995` and `119628283105` produced exactly matching original
unsigned EXE bytes, without masking or rewriting any timestamp/debug field:

| Component | SHA-256 from both independent Windows jobs |
| --- | --- |
| Management | `7a982a93819b164ca4e7d5fb7887a97726b4d0facb39475e00d4de1d243db905` |
| Agent | `a1bc8f814cd28d0cbcaaf0cab28ff7d0f0dc157badf3d77fc04f44ec1a86758f` |
| Worker | `775b3dcaa3ab79cef39a6f70199189d49423736f8d55f627e577089ac32cddd0` |

Their verified artifacts are `12400462478` (archive SHA-256
`b4582a927bbe53c5187857b586dcee0330bde5019a29edbf640a110dfd1c73f1`)
and `12400666985` (archive SHA-256
`ac69874f6b456e930040ee22a663b2e915a970e563e1dc09814249ffd5735749`).

Linux jobs `119628269626` and `119628283243` again matched all three component
binaries, with the same binary hashes as the earlier Linux comparison above.
Their verified artifacts are `12400402279` (archive SHA-256
`6e9ed5381a37fe7331a11cb09edd2cbdf34482553b87ce6dd4b2c42bd7aa81a7`)
and `12400068589` (archive SHA-256
`4187cc747b2afac5056db579fad2bd8644ca4c1c8afca683b3d31474a66f94e7`).

The original archives, per-build provenance receipts and merge-recipe correlation
are retained locally under `component-rebuild-comparison-a413ab5`. Its complete
`comparison.json` has SHA-256
`8a5789ecf664c462495c5042ea2f625340cebf5ec533707d1a0584f422480c0e`.
These are successful component comparisons on the recorded hosted CI platforms,
not universal host/toolchain reproducibility or complete product qualification.
The native graphical customer/technician apps, Flutter engine and installers
still need their own comparisons and runtime checks. No downloaded executable
was run by the artifact verifier and no installed lab service was upgraded.

### Portable package cache identity correction, 22 October

Inspection during the independent native builds found that the portable packer
embedded the current wall clock into `app_metadata.toml`, making otherwise
identical package builds differ. Its existing runtime uses this integer to
refresh the extraction cache. The generator now derives a nonzero, TOML-safe
cache identity from the complete `data.bin` payload, preserving the metadata
format and changing the identity whenever the packaged payload or startup
executable changes. Directory/file entries are sorted and the existing Python
Brotli dependency is pinned to the locally tested `1.1.0` version.

Four local tests pass for identical/changed content, dictionary insertion order,
real compression and executable selection, and directory restoration on failure.
The first real-compression test exposed an existing generator bug: it retained
`os.curdir` rather than the caller's absolute current directory and left input
files open, causing Windows temporary-directory cleanup to fail. The generator
now closes inputs and restores the caller's directory in a `finally` block. The
corrected test run also passed with resource warnings treated as errors. The
Windows packaging workflow runs these tests before creating the portable EXE.

These changes are not present in the already-running native builds
`39720429487` and `39720432071`, both pinned to earlier source `50c08fd`. Those
builds are intentionally retained for comparison of the original artifacts and
inner payloads. A subsequent native build, full binary comparison and actual
extraction-cache/restart/upgrade checks remain required for the corrected packer.
This marker does not authorize sessions or replace signed package verification.

### Native bridge and helper input comparisons, 22 October

The consumed `bridge-artifact` inputs from native workflows `39720429487` and
`39720432071` passed published archive hash/size/CRC checks. All six generated
Rust/Dart/header files were byte-identical. Their producer jobs were
`119632380801` and `119632388574`; artifacts were `12400298300` and
`12401035923`. This validates those generated source inputs, not compiled apps.

Their helper producer jobs `119632380451` and `119632388297` also completed
successfully, but the original `WindowInjection.dll` bytes differed: SHA-256
`b11015224b6d4d5964042b085447e76b835373d84b5bcccdf88355cead1292de`
versus `628bf38d93d43f9e2ec5562b45c812a96aa2651371833921a167ee0042842f99`.
Both 56,320-byte DLLs passed artifact integrity checks. Read-only PE diagnosis
found different COFF/debug timestamps and CodeView identifiers; masking those
fields in memory made the remaining bytes match. No artifact was modified or
counted as reproducible. Local evidence is in `native-dependencies-50c08fd`.

The helper x64 release workflow now requests reproducible compiler/linker
metadata and retains exact original upstream source and build/verification
scripts separately from installed payload files. Its PE guard requires x64/DLL
metadata, a REPRO marker and a PDB basename. Two parser guard tests pass locally,
including malformed/truncated metadata and build-machine path rejection; the
verifier also accepts the already independently reproduced Windows management
binary. This does not yet establish a corrected helper build, independent
helper binary comparison, code injection/session behavior or runtime acceptance.
The ongoing native builds remain pinned to the earlier helper workflow.

### Corrected helper binary and source comparison, 22 October

Independent helper workflows `39722213586` and `39722215742` at source
`f887ddc` completed successfully. Producer jobs `119638032242` and
`119638038048` produced identical original `WindowInjection.dll` bytes:
SHA-256 `0a739c4e0bb61b979103b0b2ca864f129c7ba51f5706ea8faccf1b8ef53a57c3`.
Both DLLs passed x64/DLL, REPRO marker and PDB-basename checks; no timestamp or
identifier was masked for this successful comparison. The recorded hosted image
was `win22`, version `20261012.333.1` in both jobs.

Binary artifacts `12401362042` and `12401129411` both had archive SHA-256
`d7c92fab6677938e4453551e2ad6e87c57d3b15ead79b06b3d7bd04f513dcc8c`.
Their separate source artifacts `12400968332` and `12401164054` passed published
hash/size/CRC checks and contained the exact public build workflow and verifier
sources, allowing checkout line endings. All 25 original upstream source files
mapped to public Git blobs at `ecd8d6a139eee76845ea66423fb739af450fda90`.
The original source ZIPs were byte-identical with SHA-256
`21a4a67590a1a2128b019794f9da4d1e2a5130bd411f63a6450361893bf2a732`.

The initial independent verifier rejected seven text files because Windows Git
exported CRLF while their public Git blobs used LF. Inspection proved these
were line-ending conversions only; the resumed verifier revalidated the retained
original archives against their published hashes and checked every source blob
with this conversion explicitly allowed. No artifact was modified or live
transfer restarted. Local evidence, including the failed check's investigation,
is retained under `helper-rebuild-comparison-f887ddc`.

This establishes the helper DLL comparison on these two recorded CI jobs.
It does not establish complete native payload/installer reproducibility or
runtime injection, UAC or support-session behavior with the corrected helper.

### First successful source-engine build, 22 October

Workflow `39716739857`, source `9f97ef0`, completed successfully. Its raw job log
records reviewed engine/buildroot patch verification, pinned generated-command
checks and final build step `[4900/4900] LINK(DLL) flutter_windows.dll` at
04:13:39 UTC. This confirms the matching ATL correction resolved the previous
link failure. Source reconstruction artifact `12401286693` was published at
109,172,797 bytes with archive SHA-256
`e2f2e4c312fdbc47afb1de5871a117b1dffdebde40cc0fa97bfcc0db1d7dcb11`.

The existing download/verification process is still running; neither downloaded
artifact integrity nor source-archive equivalence is counted as passed yet.
The later timestamp-controlled recipe `b1d62b2` is building separately in
workflow `39718232889`. The successful first build did not contain that recipe's
new deterministic flags, so it is not an independent reproducibility pass.
The application's reviewed upstream engine pin remains unchanged and no
source-built engine has been run in a native app or approved for production.

### Engine verifier line-ending failure and evidence preservation, 22 October

The first engine artifact transfer completed, and its declared size, archive
hash, CRC and recorded evidence-file hashes passed before the local verifier
stopped on a build-script byte comparison. The original archive had not yet been
saved by that verifier, so its in-memory bytes were lost on failure. This was
an evidence-handling defect in the local verifier, not a successful complete
artifact/source verification.

Comparison of the completed CI build log's recorded hashes against public source
`9f97ef0` established that both PowerShell build scripts and the workflow matched
public source after CRLF checkout conversion; the lock, patches and Python
verifier matched exact public Git bytes. No other source change was established
by that check. The repository now explicitly keeps these engine recipe files in
LF form, supplementing the existing engine patch/lock/Python attributes.

The replacement local collector saves and hashes the original ZIP before recipe
checks, retains resumable partial bytes with an immutable artifact identity,
and records byte progress without saving credentials or signed download URLs.
Its recipe comparison explicitly permits the established script/workflow
line-ending conversion. The replacement transfer remains active; complete engine
source-archive and binary verification are still not counted as passed. The
previous failed verifier and CI-log hash investigation remain available locally.

The native artifact upload list also explicitly includes the separate helper
source capsule. Merely downloading it into the build's artifacts directory did
not include it in the workflow's previous explicit upload list. This correction
keeps source material outside installed payloads while retaining it for release
review. Its inclusion still needs verification in a subsequent native build.

### Native build outcomes and independent artifact retention, 22 October

Native workflow `39720429487` at source `50c08fd` completed successfully.
Its independent companion `39720432071` built both apps and MSIs and passed the
managed-policy/control regression tests and technician install/repair/uninstall
checks, but the overall run failed during server packaging. Raw job
`119633063832` reports a remote connection reset while downloading the pinned
Caddy archive. This is a failed complete build, not an application/runtime pass
or an independent complete-package comparison. Its completed technician MSI
lifecycle artifact was retained; its final combined app artifact was skipped by
the then-current success-only upload step.

The workflow now retains explicitly named, completed client inputs before the
independent server-packaging step. The earlier upload is gated on successful MSI
creation, is clearly marked unsigned test input, and does not imply later
lifecycle or server packaging passed. The complete combined artifact remains
success-only. Source ZIP checksums are recorded before the early upload, without
duplicating checksum entries during server packaging.

Pinned server archive downloads now use at most three transfer attempts with
bounded delays, HTTPS/TLS 1.2 support and SHA-256 validation before atomic file
publication. A verified existing archive is reused; a mismatched existing file
is preserved and rejected. Hash-mismatched downloaded bytes are retained for
inspection and are not retried or published. Fault-injected tests for interrupted
transfers, retry bounds, existing evidence, corruption, HTTPS enforcement and
atomic publication passed locally under PowerShell 7 and Windows PowerShell
5.1. These are transfer/integrity tests with simulated network failures; actual
supplier download and early artifact retention still need a subsequent native
build. No live download was restarted to make these changes.

### Source-engine integrity and failed isolated runtime test, 22 October

The preserved `9f97ef0` engine artifact passed its outer hash, ZIP integrity,
all recorded evidence-file hashes and public recipe comparison. The original
engine and buildroot archives mapped to all 5,604 and 145 public Git blobs.
Fifty-five engine `third_party/txt/` files required explicitly verified Windows
line-ending conversion; this did not modify the original artifact or binaries.

An isolated fixture combined that engine with the previously verified `ebef5a6`
native app and app bundle. All 95 payload files were checked before launch;
the executable used the existing disposable lab signature. The actual native
process ran as the existing non-administrator `SwanLabTech` in session 1 and
exited with code 255 before displaying its interface. Its fatal message reports
snapshot version `80a49c7111088100a233b2ae788e1f48` expected by the runtime,
versus `ccfc17e3575a1ecc23c0d7bf8ff6f6cc` in the app bundle. Task completion
code zero is only the wrapper result and does not turn this into a passed test.

The public Dart `88c9758ef131d430d8ce595c6bfb4c90574d3ddd` snapshot-version
algorithm and its 15 original VM files independently produce the runtime's
`80a49...` value. A CRLF diagnostic produces a different value, so the observed
mismatch has not been attributed to checkout line endings. Flutter 3.24.5's
public engine revision is `a18df97ca57a249df5d8d68cd0820600223ce262`; its DEPS
still specifies this same Dart revision. Merely changing the engine tag is
therefore not established as a fix. The existing application build replaces
the SDK's whole Windows release-engine directory with the reviewed upstream
archive; its compiler/runtime inputs need to be treated together.

The Windows build now offers a separate explicit `source-candidate-9f97ef0`
experiment. It downloads the exact prior engine build, checks the reviewed
inner archive and each binary hash, requires the matching SDK snapshot-version
marker, and replaces only the DLL/import library/symbols. The SDK snapshot
compiler remains unchanged and its hash is recorded. Three local tests cover
compiler preservation and rejection of incompatible compilers or tampered
archives before modifying the SDK cache. Actual candidate CI compilation and
native runtime qualification remain pending. The default upstream selection
has not changed.

Original runtime receipts and logs are retained locally under
`engine-9f97-gui-ebef-inputs/runtime-evidence`. Both installed company/customer
services remained running, the installed customer executable and fixture public
profile hashes remained unchanged, and the experimental native process exited.
Host RustDesk and rendezvous/relay processes were left running.

The timestamp-controlled engine workflow `39718232889` at `b1d62b2` also
completed successfully. Its original artifact is being preserved by the
existing streaming download; complete verification and an independent
same-recipe binary comparison remain pending. It was not substituted into the
incompatible older bundle or counted as a runtime pass.

### Native application date input, 22 October

Inspection found that the pinned upstream `hbb_common::gen_version()` writes
the machine's current local minute into the native application's `BUILD_DATE`.
This is a separate source of nondeterministic executable data, beyond linker
timestamps and the portable packer's cache marker.

Managed application builds now render version metadata from Cargo's package
version and a required `SOURCE_DATE_EPOCH` in UTC. The Windows workflow uses the
source commit timestamp, and the application source capsule records that epoch
so exported sources can reproduce the input without a Git checkout. Ordinary
upstream builds retain their existing path. No submodule revision was changed.
The five source-capsule tests passed locally, including repeatable archives,
private-file exclusion and the recorded epoch's equality to its source commit.
Three native Rust regressions cover fixed UTC output, invalid date rejection
and Rust string-literal escaping. They passed in workflow `39730660383` at
`e6b87b9`; actual independent native binary comparison remains pending in
workflow `39741804606`. The earlier `5777ee1` engine experiment predates this
correction.

### Timestamp-controlled engine artifact verified, 22 October

The original artifact from engine workflow `39718232889` completed its existing
transfer and passed verification without a restart. Artifact `12403616067` is
109,003,646 bytes with SHA-256
`73a49daf109caff8180c21b31cedd83b438c4c368ae7ee095c54fdcac976ca9c`.
The original ZIP was saved before checking its contents. All 19 evidence files,
public recipe bytes, all 5,604 engine and 145 buildroot source mappings,
generated compiler commands and final linker response passed their checks.
The already investigated text checkout conversions remain explicitly declared.

The source-built DLL SHA-256 is
`ef120f74ffc05de1397bcbbf01c601af3a5452aa6a9d6939e410c41a774199ba`.
Read-only inspection confirms Windows x64 DLL format, the REPRO debug marker,
the basename `flutter_windows.dll.pdb` rather than an absolute build path, and
the expected Dart snapshot-version marker. No DLL was executed by these checks.
The original inner binary ZIP has SHA-256
`6da1e7d46774fbb1164351e85e9fa439ee84a380b5a45c80d4720d1c97d6e67e`.
Original artifacts and verification receipts remain in the private lab folder
`engine-source-b1d62b2`.

Independent workflow `39730890327` uses reviewed source `e6b87b9`. All eight
public engine recipe/workflow/test files are byte-identical in Git to `b1d62b2`;
this correlation is recorded locally. Its actual exported inputs and binaries
still require verification and comparison after completion. Different source
commits elsewhere in the app do not establish or disprove engine reproducibility.
Neither runtime qualification nor production approval is claimed.

### Licensed Server 2022 standard-user fixture prepared, 22 October

The preserved Server 2022 Desktop Experience guest booted normally at build
`20348.587`. Read-only licensing inspection reports the evaluation licensed,
with approximately 180 days remaining. A new disposable local standard account
completed an actual graphical Windows logon. An owned limited interactive task
then verified its identity, non-administrator token and session 1. Its new user
profile has no technician installation or registration. Execution policy stayed
at machine `RemoteSigned`, with no process/user override; no explicit
`DisableMSI` policy value was present. No customer or company service was started.

The public two-field lab identity marker remains System-owned and unchanged;
only that marker gained read access for the test account. Credential storage
remains protected, and product private-state permissions were not changed or
tested. This fixture is for the positive current-source technician MSI lifecycle,
separate from the previously blocked private-state access test. No new MSI phase
or full Windows qualification is counted as passed yet.

The disposable MSI runner now makes technician repair remove a previously
verified installed executable by moving it into the new owned evidence directory,
then requires the MSI to restore it. This distinguishes missing-file recovery
from repair of an intact installation and preserves the original test bytes.
Windows PowerShell parsing passed; execution against the pending current
package and independent result/log verification remain required.

### Current matching-engine runtime and installer policy, 22 October

The original `e6b87b9` native archive is 256,143,913 bytes, SHA-256
`7fc4840057d82c94fa49d3dfb2a97542247e1f398b6d26016cf2aa21bef5fb7e`.
After complete source and payload verification, three copied executables were
signed and timestamp-verified with the disposable lab publisher. No signing
key was exported. The isolated standard-user native process matched all 95
original payload files and the experimental source-built engine. Its branded
login, MFA, authorized inventory/history, customer approval and actual desktop
capture were observed. After session close, history showed Ended; sign-out
cleared inventory, password and authenticator fields. Server audit/history
confirmed one claimed, closed, customer-approved session and login/logout.
The native process exited, both preserved services stayed running, and original
customer/technician profile hashes stayed unchanged. This was the same Server
2016 VM and the older `bd13500` receiver: separate-machine control, relay,
Internet connectivity and corrected-receiver acceptance remain unproven.

The current lab-signed technician MSI has SHA-256
`cf886b50bbe91344e05bfa863a78cee49163a13c8ad37773b4f52603363238e1`.
An actual limited interactive user on licensed Server 2022 evaluation build
`20348.587` ran the public MSI harness from `eecf985`. Signature/publisher checks
passed, but installation returned 1625. The original MSI log records effective
`DisableMsi=1` and blocks unmanaged per-user installation, despite the absent
explicit registry value. No installed payload, shortcut or per-user registration
remained. Repair and uninstall did not run. Policy was not weakened. This is
a failed standard-user test, not a full Windows compatibility pass. Company
setup now explains error 1625 and directs users to an administrator; parsing
passed, but this new message has not been exercised through a company bundle.

Independent timestamp-controlled engine build `39730890327` retained and
verified all 19 evidence files, 5,604 original engine files and 145 buildroot
files. Its DLL SHA-256 equals the first build's
`ef120f74ffc05de1397bcbbf01c601af3a5452aa6a9d6939e410c41a774199ba`,
and the import libraries also match. The PDBs differ: the second has SHA-256
`ebd4d4341d313cc2e353beaad9fc5ef72d21439ffbc96aa3256a7d677d7a621e`.
Read-only MSF parsing found differences in 4,108 of 4,124 streams, so this is
not established as a timestamp-only difference. Three exported text recipes
differ only by independently checked CRLF/LF checkout conversions; original
PDB and binary bytes were never normalized. Complete engine-output and native
application/installer reproducibility remain open.

The original engine/buildroot source archives, dependency revision records,
generated commands, link response, applied patches and GN arguments match
between the two timestamp-controlled builds; recorded toolchain fields differ
only in the source commit and workflow identifiers. The next public recipe
tests the pinned DEPS LLVM linker with single-threaded integration. Patch/hash
validation, four provenance/flag regression tests and PowerShell parsing pass
locally. Its compilation, independent PDB comparison and native runtime behavior
remain unverified. The previous successful GUI smoke test does not qualify this
new linker experiment.

The first LLVM-linker experiment (`39747929292`, source `57e7f0a`) passed the
recipe guards and toolchain setup, then failed GN generation before compilation:
the clang toolchain no longer consumed the unconditionally declared `expname`.
The patch now constructs that Microsoft-only output name inside its conditional
branch. Patch reconstruction and four local guard tests pass after correction;
actual GN generation, linking, reproducibility and runtime tests remain required.

The corrected-variable build (`39750471521`, source `b72c61f`) advanced to GN's
generated-input validation and failed because Flutter's Windows archive target
still listed `flutter_windows.dll.exp`. The engine patch now removes that
obsolete intermediate from the archive list as well as from the LLD outputs.
The three-file engine patch is hash-pinned and its reconstruction and local
guard tests pass. Compilation and independent output/runtime verification
remain pending; neither failed attempt produced a qualified engine.

### Rebuilt MSI product identity, 22 October

Two local technician builds using the same verified `e6b87b9` executable/agent
and original WiX 5.0.2 recipe generated different ProductCodes and MSI hashes.
Both builders now derive ProductCode from their existing edition UpgradeCode
namespace and `windows-x64|<version>` using UUIDv5. Five local tests pass,
including cross-checking PowerShell against Python's standard UUIDv5 for both
editions and two versions. Two actual technician builds then retained identical
ProductCode `{A6857EBE-9FD4-52DE-B899-454C2F066E52}` for version 1.5.0.
Their package codes/timestamps and raw MSI hashes still differ; complete MSI
reproducibility and actual stable-identity upgrade/recovery tests remain open.

### Prepared MSI metadata, timestamps and administrative lifecycle, 22 October

Subsequent changes `d7375b5` and `67786dd` prepare deterministic unsigned MSI
metadata and stage byte-verified input copies with consistent cabinet timestamps.
Two full builds per edition produced identical MSI bytes even when identical
input payloads had different 2020/2025 timestamps. Technician SHA-256 was
`96d33dd08f5f30a49d9d318017e7ff343f4e20f87f837ad0499b5ae4c4c709e5`;
customer SHA-256 was
`839b41f2d17645bfc69eb6f44901f5ce97d6433c59cc8addf49598b8ea3e51c3`.
The actual technician builder also produced the same bytes under PowerShell
5.1 and 7. Six authoring tests and native staging/metadata guards pass locally.
This is local repeatability evidence, not independent CI reproducibility.

A separate disposable administrative account on Server 2022 Standard Evaluation
build `20348.587` then ran the public lifecycle harness against a newly built
version 1.5.0 technician MSI using installer recipe `67786dd` and verified
`e6b87b9` lab-signed executable/agent inputs. The MSI itself was unsigned:
SHA-256 `f3cb6e56229488b9d6d4dec0e07318a5520b82ee66177435ae358c98faf348e7`,
ProductCode `{A6857EBE-9FD4-52DE-B899-454C2F066E52}`. Install, missing-executable
repair and uninstall each returned zero and passed payload/registration checks.
The task completed with result zero; execution policy was unchanged. Nine
original records, including all three installer logs, were transferred and
hash-verified outside Git under the private lab's
`technician-stable-admin-20261022/lifecycle-evidence` directory.

This qualifies only those administrative bare-MSI operations. It does not
supersede the standard-user error 1625 result, demonstrate a signed MSI or a
company-worker bundle, exercise an interactive desktop, establish upgrades or
automatic updates, or qualify the full Windows matrix. Independent builds,
cross-time-zone repeatability, signed publication and the remaining end-to-end
acceptance gates are still required.

The same isolated administrative profile subsequently passed reinstall after
that uninstall, an MSI major upgrade from 1.5.0 to 1.5.1, and uninstall of 1.5.1.
All three operations returned zero. The old ProductCode became unregistered;
new ProductCode `{D32FA420-4F84-50A6-BAB2-CBFDA73F87C9}` became installed, the
shortcut remained available, and an explicitly unowned fixture file survived
the upgrade unchanged. The new unsigned MSI SHA-256 was
`8d80925e0c05717d062d4e868b31258cfa58a55449c664c8d10c6a9bed68ca8b`.
Seven original records, including reinstall/upgrade/uninstall logs, were
transferred and hash-verified under the private lab's
`technician-stable-upgrade-20261022/upgrade-evidence` directory. Execution policy
was unchanged. Both MSI versions contained the **same** verified `e6b87b9`
application payload: this proves that installer-version transition only, not
an application update, signed automatic rollout, interrupted-upgrade recovery,
or preservation of real device identity, consent or company configuration.

### Independent native rebuild comparison, 22 October

The second exact-source `e6b87b9` native build, run `39741804606`, was downloaded
once and verified against artifact digest
`274f2ce79b5210ea6cb401cb1b6c9a0113653b8036d5c933cae2bfe0802b1fb8`.
Its 1,138 public Git/submodule source files, six generated bridge files, retained
engine evidence and source epoch `1792648409` verified before comparison with
run `39730660383`. Both original archives and decoded payloads remain outside
Git; no comparison rewrote a binary.

The original unsigned `swan-agent.exe`, `swan-management.exe` and
`swan-worker.exe` matched byte-for-byte. Of 95 decoded customer payload files,
83 matched, including `librustdesk.dll` and the AOT application data. The desktop
launcher and 11 Flutter plugin DLLs differed by two or four bytes each. PE
inspection confirmed every differing byte lies in the COFF or debug-directory
timestamp fields. All 12 lacked `IMAGE_DEBUG_TYPE_REPRO`. Consequently the
portable packages, both old-recipe MSIs and server setup package also failed
the complete-byte comparison. Full native reproducibility is **not achieved**.

The Windows CMake configuration now requests `/Brepro`, non-incremental linking
and PDB basenames for non-Debug runner/plugin targets before their creation.
The Windows build workflow checks those actual outputs using the existing PE
metadata verifier and retains its receipt. All 12 actual failing old binaries
were rejected by that verifier; its two parser tests pass, and the original
Rust native DLL passes its metadata check. Corrected C++ compilation, independent
raw-byte comparison, runtime qualification and server-setup reproducibility
remain unverified. A REPRO marker by itself will not satisfy those gates.

### Server launcher compiler repeatability, 22 October

The two original `e6b87b9` server setup executables had matching resource/source
manifests but different generated .NET module IDs. The legacy Framework compiler
does not provide a deterministic-compilation option. The recipe now uses the
hash-pinned, Microsoft-signed Roslyn 4.12.0 compiler package, ordered compact source
metadata, deterministic compilation and explicit, hash-recorded references.
It verifies all embedded resources, including `SOURCE.json`, after compilation.

Two actual local builds with the final revised recipe produced identical unsigned
server setup SHA-256
`3a7d1b17c8ed4028a73c1fc03a2669fa6b8e6445e086085934a0f6c4fbaabf4d`.
One build used Windows PowerShell 5.1 and the other PowerShell 7. A wrong compiler
package was rejected before extraction. The receipt and original outputs remain
under the private lab's `server-setup-roslyn-20261022/final-validation` directory.
This was a dirty working-tree experiment over revision `49ced68`, using the
verified `e6b87b9` management executable and retained pinned transport/HTTPS
resources; it is not an independent CI build of the complete current server.
No setup executable was installed or run as an application during these checks.
The updated CI tests, reference-assembly pinning across images, production signing,
actual server deployment and the remaining acceptance gates are still unverified.

The next server recipe revision also pins all five .NET/PowerShell reference
assemblies through two hash-verified Microsoft SDK packages and checks each
selected DLL hash before compiling. It no longer reads the host's runtime/GAC
assemblies as compiler references. Two actual local builds of that revision,
again using PowerShell 5.1 and 7, produced identical unsigned server setup
SHA-256 `0489bfd78caae185edab646f72a74f84c063056e82d23c2e523cb929774774ab`.
Both wrong-reference-package cases were rejected before extraction; the receipt
verified every compiler reference against the published pins. Original outputs
and the receipt remain under the private lab's
`server-pinned-references-validation-20261022` directory. This was a dirty-tree
experiment over `e3721a7`, using the retained `e6b87b9` management executable.
Independent CI, cross-image comparison, native runtime/deployment and signing
qualification remain pending; the local result does not satisfy those gates.

The pinned-reference executable also returned zero from its packaged
`--render-preview` path on Windows 11 Home build `26200`, and the rendered form
was visually inspected. This exercises launcher loading, resource extraction,
Windows PowerShell runtime binding and preview rendering. Installation and
browser actions were disabled; no company services were installed. The CI
repeat-build test now exercises this preview mode and retains the PNG. This is
preview-only runtime evidence, not actual server setup or deployment acceptance.

The same pinned-reference launcher was exercised in preview-only mode on the
existing Server 2016 Standard Evaluation VM, build `14393.693`, with .NET
Framework release `394802`. It rendered a PNG and returned zero when observed
through an owned `System.Diagnostics.Process` handle. The company server,
installer worker and customer services remained running before and after.
The PNG and original report were transferred and hash-verified under the private
lab's `server-preview-pinned-refs-16-20261022/repeat-observation` directory.
The headless SSH render omits some native window/border decoration; interactive
appearance is not qualified by this image. No management process was started
and no server installation was attempted.

The first preview observation rendered an image but did not expose an exit code
through Windows PowerShell's redirected `Start-Process` result. Its inconclusive
report remains preserved; it was not counted as a pass. The original PID was
confirmed absent before the repeat. The public preview test now owns the child
process handle directly; its full repeated-compilation, package-rejection and
preview checks also passed under Windows PowerShell 5.1 on the host, with records
under `server-preview-process-ps5-validation-20261022`. This addresses test-result
capture only and does not qualify deployment, upgrades or the full Windows matrix.

### LLVM-linked engine artifact and startup smoke, 22 October

The corrected engine recipe at `05d5369` completed in run `39753153773`.
Artifact `12416969754` (107,382,139 bytes) was preserved and verified against
SHA-256 `5629a63868421d17ac00a6cdd4215cda852c59b8bcf9efb802482d6248751d53`.
All 19 evidence files, 5,604 original public engine files, 145 original public
buildroot files, pinned recipe inputs and generated compile/link commands
verified. The DLL SHA-256 was
`c87ba5a71afa7b7100c1c9ccc1289009dab812f9de2acc38f9d5103e48c1f204`.
An independent same-source rebuild later completed; its original-byte comparison
is recorded in the independent LLVM engine result below.

An isolated Server 2016 `14393.693` standard-user experiment combined this engine
with the matching `e6b87b9` application payload and its existing lab-signed native
main executable. Live process-module inspection confirmed the candidate engine
and expected Rust DLL were loaded from that isolated payload. Computer-use
inspection showed the company-branded technician login, username/password/MFA
fields and product/license attribution. No credentials were entered. The window
was closed through its title bar and the process exited zero. All 95 payload
files verified afterward with the two declared engine/signed-main substitutions;
the original bootstrap and installed receiver hash stayed unchanged, and all
three existing Swan services remained running. Six original run/log/input
records were transferred and hash-verified under the private lab's
`engine-05d5369-gui-20261022/evidence` directory.

This is engine startup/rendering/clean-exit evidence only. It does not prove
authentication, remote-session behavior, public connectivity, independent PDB
reproducibility, installer/updater compatibility, production signing or the full
Windows matrix. The default application engine remains unchanged.

### Fresh standard Windows 10 Pro MSI lifecycle, 22 October

A new disposable VM was installed from Microsoft's Media Creation Tool standard
Windows 10 x64 ISO, selecting Pro. The ISO SHA-256 is
`ac6f8bd452c476f07f2ca14d85622e7c8723af6517aa27a014ddc4831e38b693`.
First interactive logon and the guarded provisioning report completed. Tests
recorded Windows 10 Pro x64 `19045.3803`. This is a standard unactivated
installation: the read-only licensing observation recorded status 5 and zero
grace. No activation bypass, rearm, clock change or licensed-trial claim applies.

The previously verified unsigned technician MSI 1.5.0, SHA-256
`f3cb6e56229488b9d6d4dec0e07318a5520b82ee66177435ae358c98faf348e7`,
contains lab-signed application `e6b87b9` and uses installer recipe `67786dd`.
Under the VM's Administrator account, installation, repair of a deliberately
missing technician executable, and uninstall each returned 0. The public harness
checked installed executable hashes, registration, shortcut and removal. Nine
original result/driver/log records were collected and hash-verified under the
private lab's `technician-win10-admin-20261022/lifecycle-evidence` directory.

The first unsigned test-script invocation was rejected by the default PowerShell
policy before installation; that failed observation remains retained. The
verified test driver then ran in a child process with `-ExecutionPolicy Bypass`.
Its reported process policy is explicit. No machine/user PowerShell or Windows
Installer policy was configured for this test; machine/user PowerShell policy
snapshots matched. The child process exited 0 and the
original phase reports independently recorded each MSI result. This is an
administrator bare-MSI functional result for this exact package and OS build,
not standard-user, customer, company-worker, enrollment, update, licensed-Windows
or complete Windows 10 qualification.

### Complete component/deployment CI observation, 22 October

The manual company-product run
[39766139184](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39766139184)
at exact source `fa188bcfe78152bef9504fca41a161a0679587bc` completed successfully.
Windows 2022 and Ubuntu 24.04 component verification jobs passed, including the
pinned server compiler/references reproducibility and packaged preview checks on
Windows. The Linux deployment job passed its fresh setup/persistence, isolated
HTTPS/transport rehearsal, maintained transport source build, and backup recovery
checks. The optional technician MSI lifecycle job was skipped; this run supplies
no evidence for that job. These CI checks do not replace the outstanding real
Windows server deployment, public-network, interactive session, signing and full
compatibility acceptance requirements.

### Independent LLVM engine original-byte comparison, 22 October

The same public engine recipe `05d5369c04804b9a9b2dc06df0cb85b0efe6f274`
completed independently in runs
[39753153773](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39753153773)
and [39761977920](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39761977920).
The second artifact `12420939008` (107,382,134 bytes) verified against SHA-256
`f1e6df169a68c3cc470d68522d7ea997ade549f2c07b621566f76bc07192b258`.
Both retained archives independently passed the 19-file evidence verification,
5,604 public engine source mappings, 145 public buildroot source mappings,
pinned recipe checks, generated commands and final link-response checks.

All three uncompressed outputs matched original bytes: `flutter_windows.dll`
(18,156,544 bytes), `flutter_windows.dll.lib` (30,436 bytes), and
`flutter_windows.dll.pdb` (239,464,448 bytes). No timestamps, debug records or PDB
streams were normalized. This establishes independent reproducibility for these
exact engine outputs and recorded inputs. It does not establish equality to the
historical upstream binary, complete application reproducibility or runtime
acceptance. Both original artifacts remain preserved privately; the public
[comparison record](../deployment/windows/engine-05d5369-reproducibility.json)
contains both artifact identities and all output SHA-256 hashes.

The native workflow now offers this explicitly pinned engine as the additional
`source-candidate-05d5369` lab choice. Five selector regression tests passed,
including unknown-choice and wrong-generation rejection. A separate local
selection check accepted the actual first archive and all three binary pins
while preserving a non-executed snapshot-compiler fixture; that fixture check is
not a real SDK/application runtime test. Runtime and production approval remain
false, and the default engine selection remains unchanged. The previously
recorded standard-user login-screen startup smoke remains the only runtime scope
qualified for this engine so far; full remote-session checks remain outstanding.

### Standard-user technician MSI on standard Windows 10 Pro, 22 October

The same unsigned 1.5.0 technician MSI recorded in the administrator result
above also passed installation, missing-executable repair and uninstall as
`SwanTechMsi10` on the standard Pro `19045.3803` VM. The original phase reports
each recorded `administrator: false`, `standard_user_required: true` and exit 0.
The task ran with a limited interactive token in session 3 and completed with
last result 0. Ten original task, safety-marker, result and MSI log files were
transferred and hash-verified under the private lab's
`technician-win10-standard-marker-20261022/lifecycle-evidence` directory.

The initial password task was registered but never ran and was disabled before
an alternate launch. A direct credentialed process launch was denied before it
created a driver PID. After normal interactive login, the first driver attempt
failed before MSI results because the independently provisioned safety marker
allowed only SYSTEM and Administrators to read it. Its failed records remain
preserved. The corrected fixture grants this owned standard account read-only
access to that single lab marker, preserving its contents and SYSTEM ownership;
it grants no product-state permissions. A fresh fixture and separate task then
produced the passing result. None of the failed driver starts is counted as an
MSI execution or pass.

The verified driver used process-only PowerShell Bypass, with the original
machine/user policies remaining undefined. No Windows Installer policy was
configured. The OS remains unactivated with the licensing state recorded above.
This is standard-user bare-MSI evidence for the exact old application/installer
inputs, not current-source, company enrollment, customer installation, approved
updates, remote-session behavior or complete Windows 10 qualification.

### Corrected desktop artifact verification and comparison tooling, 22 October

The complete `49ced68` native artifact from run `39760022829` was preserved:
256,166,945 bytes, SHA-256
`c1ed67f599910d9d94e58caa1635761521c1f91d140fe3c435ea1716646710d8`.
Its 1,143 tracked public source files, six generated bridge files, source epoch
and retained pinned engine evidence verified. All 95 decoded customer payload
files matched the installed-file manifest. Independent inspection of the actual
main executable and 11 plugins matched their retained PE guard receipts: each
has a REPRO marker and no absolute PDB path. These markers do not establish
independent reproducibility; that comparison and this build's runtime acceptance
remain pending.

A manual read-only native comparison workflow now verifies original archive
digests, exact public source and all eight expected shipping executable/MSI
outputs from two different successful runs. Eight trust-boundary regression
tests passed, covering altered source with a self-consistent manifest, invalid
file/download hashes, unsafe storage origin, incomplete output sets and binary
differences that must not be hidden by source line-ending handling. The new
source/hash verifier also accepted the actual preserved `49ced68` archive;
that single-archive check was not an independent-build comparison. No completed
pair-workflow result is claimed here. Runtime and production approval stay false.

### Fresh activated Windows 11 administrator MSI lifecycles, 22 October

Official Microsoft Windows 11 Enterprise 26H2 evaluation media completed and
matched its published SHA-256
`bc3f24086ebadc94489066b5ad78089e2cf5c3491e90e790bb81a2b199c10e38`
(8,225,329,152 bytes). A new 64 GiB GPT guest with 4 GiB RAM and two virtual
CPUs installed build **26300.9457 x64**. The isolated Alpine 3.24.2 WSL2 lab uses
QEMU 11.0.3/KVM, secure-capable EDK2 firmware with Microsoft Windows boot
certificates, and a swtpm 0.10.2 TPM 2.0 backend. The installed Windows OS
reported Secure Boot enabled and TPM 2.0 present and ready. No Windows hardware
requirement checks were bypassed. Its first logon completed, and the temporary
automatic-logon password was removed. Normal Microsoft online activation then
reported LicenseStatus 1 and 129,600 evaluation minutes remaining; no key
replacement, rearm or clock change was used.

Native run `39777271372`, exact source
`9e4ea6dfce7dc1e66eccf6e1068780a23ae8fd59`, supplied the unsigned MSI inputs
with the explicitly selected source-built `05d5369` engine. Original archive,
1,149 tracked public files, six generated bridges and retained engine evidence
verified. The 95 portable application files matched their manifest, and the
main executable plus 11 plugins matched their PE metadata receipts. These
checks do not establish complete native output reproducibility; see the
[original build comparisons](SOURCE_AND_LICENSE.md#source-built-engine-application-comparison-22-october-2026).

Under the guest Administrator, both editions passed actual install, repair and
uninstall with exit code 0 in all six operations, without a required reboot:

| Edition | Original unsigned MSI SHA-256 | Repair evidence |
| --- | --- | --- |
| Technician | `297265c102b0c94b5361c76b9873bc2b09fe3e76b2f98975a06b840452babeba` | A previously verified main executable was retained outside the install path; MSI repair restored the missing executable with its expected hash. |
| Customer | `9ebb1d8e56248b9b6005ed1a884ff29ec5edd8421b60b00253ec743e875cac48` | The packaged license was deliberately damaged; repair restored it and all 96 manifested files. The service's executable path, LocalSystem identity and running state were checked. |

MSI registration, installed executable/agent hashes, technician shortcut and
component registrations, complete customer payload and uninstall removal were
checked. Each edition's nine original phase/driver/MSI records were transferred
and hash-verified in the private lab. Original machine/user execution policies
were preserved; only the verified test driver used process-scoped Bypass.

The initial customer driver used the portable manifest (95 files) instead of
the MSI-specific manifest (96 files including `LICENSE.txt`). The harness
rejected this fixture before invoking MSI; its five original failure records
were preserved and hash-verified. A separate corrected fixture used the actual
MSI build receipt's manifest hash
`3453afe7fd075f1739bbbfa04980f21f5bcf6ba7dceda1d47333a94af7be485c`
and produced the passing customer results. No MSI failure or pass is claimed
for that first preflight-only attempt.

The technician's original harness recorded the legacy registry ProductName
“Windows 10 Enterprise Evaluation”; its build/platform checks and independent
Win32_OperatingSystem observation identify Windows 11. Original records remain
unchanged. The public harness now obtains caption, version, architecture and
build from Win32_OperatingSystem while retaining the exact registry UBR; the
customer run verified the corrected Windows 11 caption.

This is **administrator bare-MSI evidence for those exact inputs**, not
standard-user installation, company-generated bundles, enrollment, remote
sessions, approved updates, interruption recovery, trusted production signing
or complete Windows 11 qualification. Those release gates remain open.

### Activated Windows 11 standard-user technician MSI, 22 October

The same original unsigned technician MSI from source `9e4ea6d`, SHA-256
`297265c102b0c94b5361c76b9873bc2b09fe3e76b2f98975a06b840452babeba`,
passed actual install, missing-executable repair and uninstall under an owned
local standard user on the activated Windows 11 Enterprise evaluation
**26300.9457 x64** above. All three MSI exit codes were 0, with no required
reboot. The account belongs to Users and not Administrators. A normal graphical
login created its active console session 3; the manually started test task used
that account's Interactive/Limited principal. The driver rejected a different
SID, an administrator token or a non-interactive session, and each public
harness phase required `RequireStandardUser`. Original phase records report
`administrator=false` and `standard_user_required=true`.

The fixture grants that account read access only to the independent lab safety
marker and access to its own test directory. Marker bytes/owner were preserved;
no customer product-state permissions or Windows Installer policies were
changed. The verified driver used process-scoped PowerShell Bypass and preserved
its original machine/user execution policies. The corrected public harness
records the actual Windows 11 caption. Ten original task, marker, lifecycle and
MSI records were transferred and hash-verified; the copied executable, agent,
shortcut, component registration, missing-file restoration and uninstall
removal checks passed. The first administrative technician records retain
their original legacy registry caption as described above.

These three standard-user operations plus the six administrative operations
are **nine passing source-specific bare-MSI operations**, not complete Windows
11 compatibility. Company bootstrap/enrollment, accepted remote sessions,
approved updates, upgrade/interruption recovery, production signatures and the
remaining feature/Windows matrix still require their own evidence.

### Unpublished 1.5.1 inputs for real update tests, 22 October

The application, Flutter build number, portable packer, workspace lock entries
and native installer workflow now consistently select **1.5.1** (Flutter build
69). This prepares a genuinely different application version for approved
automatic-upgrade tests over retained 1.5.0 inputs. The baseline tag, existing
1.5.0 build references, installed lab identities, company profile revisions,
installer edition namespaces and publisher trust are unchanged. This is not a
production release, an applied update, or evidence that any upgrade passed.
Native build/source verification, signing policy, admin approval and actual
interruption/session/identity-preservation tests remain required.

### Complete unsigned native reproducibility, 22 October

The original outputs from successful independent native builds `39807079997`
and `39819129959` at source `e9d89ca97319590905a39663b76718ee53bcdd78` passed
[comparison 39829786141](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39829786141).
The [unchanged original report](../deployment/windows/native-e9d89ca-rebuild-comparison.json)
verifies both original archive digests, public source and generated bridges,
and all eight expected executable/MSI outputs are byte-identical. No shipping
file was normalized or executed. This uses the explicitly selected independently
reproduced `05d5369` engine; it does not establish the default engine's source
reproducibility or extend this result to a different application revision.

The first archive was also retained locally and passed source/artifact checks;
all 95 extracted application files match the installed-file manifest (accounting
for the documented portable `rustdesk.exe` to installed `Swan Remote Support.exe`
name). The 12 native desktop metadata hashes and pinned engine DLL hash match.
These are build-integrity checks, not runtime tests. Earlier failing original
comparisons remain retained as historical evidence.

The real 1.5.1 native build `39820151128` at `699c9e0` has now succeeded.
Its subsequent independent comparison and local integrity checks are recorded
below. Installation and applied automatic-upgrade evidence remain separate.
Production signing and the full installation/session/update/Windows acceptance
requirements remain open.

### Unpublished 1.5.1 independent comparison, 22 October

Successful native runs `39820151128` and `39832811268`, both at exact source
`699c9e022ad71e4a409be52b56c9fe9697074bdf` with the selected `05d5369` engine,
passed [comparison 39844492046](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39844492046).
The [original report](../deployment/windows/native-699c9e0-rebuild-comparison.json)
records eight byte-identical unsigned shipping outputs, both original archive
digests and verified public source/generated bridges. Shipping files were not
normalized or executed.

Local collection also verified the first original archive, complete public
source, six bridges, selected engine and all 95 decoded payload-file hashes.
The application EXE and Rust DLL have different bytes from the preceding 1.5.0
inputs. Actual major upgrade, approved automatic rollout, interruption recovery,
identity/consent preservation and the remaining runtime/release gates still
require explicit test results; build reproducibility does not prove them.

### Actual technician 1.5.0 to 1.5.1 MSI upgrade, 22 October

On the retained Windows Server 2016 Standard evaluation Desktop Experience
**14393.693 x64**, an isolated new local administrator profile passed four real
MSI operations: install 1.5.0, major-upgrade to 1.5.1, repair after retaining and
removing the installed technician executable, and uninstall 1.5.1. All exit
codes were 0; no reboot was required. These are original unsigned inputs from
the independently reproduced builds above:

| Input | Application/recipe source | Original MSI SHA-256 |
| --- | --- | --- |
| Technician 1.5.0 | `e9d89ca97319590905a39663b76718ee53bcdd78` | `f9bf6db54580945e4f588a84e95fffc77632f2a43d7c80881485dc8342d3ac33` |
| Technician 1.5.1 | `699c9e022ad71e4a409be52b56c9fe9697074bdf` | `f09b59d42287b595548e6a894f6fb498f46106218d66fc6742e5f17eae8b8b4d` |

The upgrade removed old ProductCode
`{A6857EBE-9FD4-52DE-B899-454C2F066E52}` (ProductState -1) and installed
`{D32FA420-4F84-50A6-BAB2-CBFDA73F87C9}` (ProductState 5), retaining technician
UpgradeCode `{A4374699-436F-4917-9A4E-223F62A9E634}`. The public lifecycle
harness then verified the new executable/agent hashes, shortcut and component
registration before inducing and repairing a missing executable. Uninstall
verified removal of the new payload, shortcut and registration.

The existing administrator profile contained technician software; its preflight
stopped without modification. A separate owned administrator account was used
instead. A manually started Password/Highest scheduled task ran under that
account's actual SID; no SSH access rule or Windows Installer policy was changed.
Task-principal name normalization and null-trigger preflight issues were
corrected before its first execution; the task was confirmed never previously
run. The final task, wrapper and original phase reports all record success.
Fourteen original driver, phase, invocation and MSI records were transferred
and hash-verified in the private lab. Machine/user execution policies were
preserved; the verified test process alone used PowerShell Bypass.

The existing customer's device ID, device-token hash, unattended consent,
consent revision and accepted release sequence were unchanged, and its customer,
management and worker services remained running. No technician app was launched.
This proves an **administrative bare-MSI major upgrade in that profile**; it is
not standard-user/interactive acceptance, an enrolled technician identity test,
an approved automatic update, interruption recovery, publisher validation,
remote-session acceptance or complete Server 2016 qualification. The retained
VM is not represented as a fresh clean-machine test.

### Server 2016 standard-user MSI policy rejection, 22 October

The same original unsigned technician 1.5.0 MSI at `e9d89ca` was tested under a
separate owned standard account on Server 2016 Desktop Experience **14393.693**.
A normal graphical login loaded that account's profile in active console
session 1. An Interactive/Limited task ran the verified driver with the exact
account SID; the public harness required `RequireStandardUser` and recorded
`administrator=false` and `standard_user_required=true`.

Initial installation returned **1625**, with the original MSI log explicitly
reporting effective machine policy `DisableMsi=1` and installation forbidden by
system policy. Read-only inspection found no explicit machine `DisableMSI`
registry value; absence did not mean installation was allowed. The technician
installation directory remained absent. The attempt stopped at installation:
upgrade, repair and uninstall were not run, and no pass is claimed for them.
Seven original wrapper, driver, phase and MSI records were transferred and
hash-verified. This does not invalidate the separate administrator-profile
upgrade result or establish that every Server 2016 deployment has this policy.
Microsoft documents [DisableMSI's managed-application restriction](https://learn.microsoft.com/en-us/windows/win32/msi/disablemsi).

The initial Password/Limited task never executed: original Security 4625 events
reported status `0xc000015b`, logon type 4 (batch logon denied), and its task
retained never-run result 267011 with no wrapper/MSI output. That setup failure
was retained separately. No batch-logon right or Windows Installer policy was
changed; the later independent Interactive/Limited task produced the actual MSI
rejection above. Only the independent lab safety marker received a read grant
and the account's own fixture directory received Modify access. Marker bytes
and owner were preserved; customer product-state permissions were untouched,
and the standard-user driver did not access customer private ProgramData.

Standard-user Server 2016 MSI/upgrade acceptance remains unproven. An approved
administrator/IT deployment is required where Windows policy rejects unmanaged
per-user MSI packages; the software must report that rejection and cannot
silently elevate or weaken policy. Other Windows standard-user, interactive
application and automatic-update requirements retain their separate gates.

### Native 1.5.1 technician GUI and existing-company authorization, 22 October

The verified 95-file application payload and agent from exact source
`699c9e022ad71e4a409be52b56c9fe9697074bdf` ran under the owned standard user's
active console session 1 on Server 2016 **14393.693 x64**. This was an explicitly
unsigned runtime fixture with separate company state, not a company installer
or a workaround recorded as a passing MSI test. The agent validated the existing
lab company's HTTPS endpoint and signed profile; native UI rendered the updated
technician display name, logo, support contact and product/license attribution
from revision 18. The retained cached profile signature was independently
verified after collection.

A new company account with role `technician` completed actual GUI password/TOTP
login. With no group access, its UI showed no authorized devices. A separate
normal authenticated API session also returned an empty inventory and HTTP 403
for a valid-proof, customer-approved grant request targeting the company's one
approved device. A temporary group assignment made that device and its support
button visible after refresh. An API request then returned a signed 60-second
grant; its signature, company, technician, device, ephemeral proof-key binding
and permissions were verified against the pinned company key and actual group
policy. The request was never claimed and no remote session was started.

Removing that temporary group access closed the unclaimed grant, cleared its
lease, removed the device from UI/API inventory after refresh, and made another
valid-proof request return HTTP 403. This proves server-side authorization
alongside UI behavior; it does not test receiving-device enforcement or an
active session's lease. The GUI signed out, cleared inventory/history, and
closed with actual process exit 0. Its task also completed with result 0.
Twelve original wrapper, setup, state and runtime records were transferred and
hash-verified. The collected records contain neither the test password nor its
TOTP secret. The temporary company account was disabled, its grants were closed,
and the existing customer's identity, consent and three services were preserved.

The old deployed application displayed an update-deferred warning in this
fixture; no installed release was recorded and automatic updates are not
qualified by it. The history row also incorrectly displayed `Requested` for the
closed unclaimed grant. Current UI source now prioritizes `closed` and displays
`Closed`; that correction requires its own rebuilt runtime verification.

This evidence covers **native profile/GUI/MFA/inventory and existing-company API
authorization** for those exact inputs. It does not establish fresh company
deployment, standard-user MSI installation, production signatures, an approved
software update, external/direct/relay connectivity, customer approval or
receiving-device session behavior. Those acceptance gates remain open.

The later technician-update diagnostic change at exact source
`54931939887a0d2a376f1535082bd606254d6e9e` passed company-component push
[39848503726](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39848503726)
and PR [39848513299](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39848513299):
Windows 2022 and Ubuntu component/security tests and builds, plus the Linux
deployment job, succeeded. The optional technician MSI CI job was skipped;
these results do not replace the separate native installer and runtime evidence.
They do not include a rebuilt runtime test of the subsequent history-label fix.

### Existing lab worker publication failure and corrected reporting, 22 October

Read-only inspection of the retained Server 2016 deployment found two old build
claims still running in the management database. The worker process had no
active child build process; its protected log repeatedly recorded HTTP 400 after
package signature verification. Its original signed-input ZIPs were preserved.
An exact-hash upload reproduction received the server's JSON rejection
`Unexpected ZIP footer`. The retained archive has a valid 11-entry ZIP32 footer.
Installed-binary inspection found the technician-uninstall bundle helper in the
worker but absent from the management executable. Neither old executable
contains the newer recipe-discovery endpoint, which returned HTTP 404.

This is evidence of an old mixed component deployment, not a passing installer
publication test or a reason to relax ZIP validation. Current server/worker
source already checks matching bundle recipes before claims. Coordinated lab
maintenance remains pending: automatic approval review rejected preparation of
the refreshed release-withdrawal/encrypted management-backup draft as
`blocked by policy`; fresh human approval has been requested. That rejected
preparation created no draft file and changed no services or releases.

Current worker source now turns permanent upload-validation rejection into a
failed completion report, preserving the original local output and publishing
no download link. Auth/state conflicts and temporary network/server/rate-limit
failures retain recovery. Diagnostics accept only fixed public error labels
from a bounded response with a reading deadline. Wire regressions cover failed
completion reporting, identical-byte transient retries, auth/conflict recovery
and exclusion of arbitrary remote text/secrets. All four new regressions passed
on Windows 2022 and Ubuntu at source
`4c5accb7ae73615c799a43fa70dc2a49aeed0822` in
[PR check 39855276940](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39855276940).
The original two job logs were retained and their individual passing test lines
verified. That full component/deployment check passed; the optional MSI job was
skipped. These are HTTP/control-path regression tests, not a matched Windows
worker/server publication test. Runtime publication and download authorization
remain open while maintenance approval is pending.

### Rebuilt native history-label regression, 23 October

Native [run 39851990495](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39851990495)
passed every job at exact source
`3bad1697b0d744421ded6386cbcd091ea0b13e67`. Its original unsigned artifact
`12461520673` was retained: 254,500,267 bytes, SHA-256
`be54aa65b24e0e0ebd4a3bb3bd0cb880e2336d0acba910db9cf1a257fc386ce8`.
Verification matched 1,157 tracked public source files, all six generated bridge
files, and the original selected engine source/evidence. All 95 extracted
portable payload files matched the installed-file manifest without modifying
executable bytes. This particular source has not yet passed an independent
native rebuild comparison; earlier source-specific comparisons do not prove it.

The unchanged extracted app and agent ran with isolated technician state under
the owned standard Windows account on the retained Server 2016 lab guest. HTTPS
setup accepted company profile revision 18 and displayed its branding. Normal
MFA login with the owned company **administrator** account displayed inventory
and existing closed sessions. Those history rows now visibly say `Closed`.
No support request was made and no account, group or permission was changed.
Sign-out removed inventory/history and cleared password/authenticator fields;
the process and interactive task subsequently exited with result 0. Nine
original setup, state, fixture and runtime records were transferred with byte
counts and SHA-256 verification; the test password and TOTP secret were absent
from them. All three existing lab services remained running.

This confirms the native history-label correction, signed-profile startup and
MFA/sign-out for that bounded fixture. Administrator login does not replace the
earlier technician-role authorization tests. The unpacked app displayed an
update-deferred warning and had no installed release; neither automatic updates
nor the Windows Installer policy diagnostic was exercised. Fresh deployment,
MSI acceptance, receiving-device enforcement, separate-machine/public transport,
production signing and update qualification remain open. This native source
also predates the newer `4c5accb` worker rejection fix; that fix's component tests
must not be attributed to these native binaries.

### Windows 10 administrative technician major upgrade, 23 October

The preserved standard Windows 10 Pro x64 guest, build `19045.3803`, passed a
real bare-MSI upgrade from source `e9d89ca` version 1.5.0 to native source
`3bad169` version 1.5.1. The latter original MSI SHA-256 is
`185e37ff134d756c741ec81e68eb03d09a2ef230920357272481ff920f4715e9`.
Initial install, major upgrade, missing-executable repair and uninstall all
returned 0 without requiring reboot. Actual technician application bytes changed;
the old ProductCode became absent, the new ProductCode became installed, and
the technician UpgradeCode remained stable. Repair checked the new payload;
the independent original report confirms missing-file recovery.

The original administrator profile's unused-directory guard first stopped
before installation because an earlier technician directory already existed.
That directory and its original failure evidence were preserved. The passing
run used a fresh owned administrator profile with a password-authenticated,
highest-privilege task. The harness used a process-only execution-policy option;
persistent policies were unchanged. No customer enrollment or company services
were present in this Windows 10 fixture before or after the test. Fourteen
original fixture, invocation, result and MSI-log records were retained and
verified against guest byte counts and SHA-256 values. An initial collector
field-name assertion failed after transferring those records; a separate
metadata-only verification validated the originals without repeating any MSI.

Windows activation remains status 5 with zero grace minutes; no activation
bypass was attempted. These are functional **administrative MSI upgrade**
results on the specified guest, not complete licensed Windows compatibility,
standard-user installation, company-generated distribution, production signing,
enrollment preservation or automatic/interrupted-update qualification.

### Windows 10 standard-user technician major upgrade, 23 October

The same original 1.5.0 `e9d89ca` and 1.5.1 `3bad169` technician MSIs subsequently
passed install, real major upgrade, missing-file repair and uninstall under a
fresh owned **standard** account on Windows 10 Pro `19045.3803`. Normal GUI
sign-in initialized the account; the test used an interactive, limited task.
The driver rejected administrator tokens, and the install/repair/uninstall
reports independently recorded `administrator: false` and
`standard_user_required: true`. All four operations returned 0. The upgrade
removed old product registration and installed the new product with the stable
technician UpgradeCode. Eighteen original fixture, task, marker, invocation,
result and MSI-log records were retained and verified by byte counts and
SHA-256 hashes.

The first limited task failed before any MSI because the independent lab safety
marker granted read access only to the older lab account. Its task record was
preserved and disabled after confirming no instances, upgrade-start record or
MSI result directory. Read-only access to that **lab marker** was added for the
new account, preserving its bytes and owner; a separate guarded limited task
then completed the actual tests. No customer product directory access probe or
customer ACL change was performed. Persistent script policies and the lab
marker remained unchanged during the passing test. A historical helper's
plaintext disposable password was also rotated; its replacement stays in
protected local storage and is not part of published evidence.

This extends the exact-source functional MSI evidence to a standard Windows 10
user. It does not qualify company bootstrap/enrollment, signed distribution,
automatic updates, interruption/reboot recovery or remote sessions. The guest
remains unactivated with zero grace minutes; licensed compatibility qualification
and the remaining Windows matrix are still open.

### Windows 10 customer MSI repair with a real reboot, 23 October

The retained Windows 10 Pro `19045.3803` guest passed bare customer MSI
installation at exact native source `3bad169`; package SHA-256
`73d58b30997618347d19c560541909fbf0af24f00f9e72499e8382ac95d65b49`.
All 96 MSI-packaged files and the agent matched their pinned hashes, and the
customer service ran as LocalSystem from the expected executable path.

Repair restored a deliberately damaged packaged license, passed its payload
checks and returned **3010**, requiring a reboot. The original no-reboot driver
therefore stopped and retained its failed overall result. Its original MSI log
records deferred cached-installer file deletion and a requested restart. The
test did not reinterpret that attempt as a zero-reboot pass or repeat the MSI.
After retaining and hash-verifying its evidence and confirming no active MSI
client, the disposable guest performed a normal requested restart. A changed
operating-system boot timestamp proved the reboot. All 96 files and the agent
still matched; the customer service automatically returned to Running with the
expected LocalSystem account, automatic startup and exact executable path.

A separate guarded continuation then uninstalled the same original package with
exit 0 and no further reboot. The public harness verified removal of packaged
files, agent, product registration and service; an independent final observation
also found the service absent. Twenty-one original pre/post-reboot fixture,
invocation, result and MSI-log records were retained with byte counts and
SHA-256 verification. Persistent script policies were unchanged.

This proves **bare customer MSI install, repair requiring reboot, post-reboot
payload/service recovery and uninstall** in the specified lab. It does not prove
company enrollment, consent/identity preservation, graphical support sessions,
signed distribution or automatic update recovery. The unactivated Windows
limitation and the other release gates remain open.

### Latest worker-fix native reproducibility, 23 October

Both native builds at exact source
`4c5accb7ae73615c799a43fa70dc2a49aeed0822` passed every job:
[first 39867952641](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39867952641)
and [independent 39867956814](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39867956814).
They include the worker permanent-upload-rejection fix as well as the earlier
technician history/update-diagnostic changes. The pinned
[comparison 39876756059](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39876756059)
passed and its original [report](../deployment/windows/native-4c5accb-rebuild-comparison.json)
was retained unchanged. All eight shipping outputs are byte-identical across
the independent builds; no shipping-file normalization or execution occurred.
Both public source proofs verified 1,157 tracked files and six generated bridge
files at the exact source timestamp.

Only the completed independent build's full original archive was downloaded
locally: artifact `12467476411`, 254,512,609 bytes, SHA-256
`58e48e54d33a4ebfe57e37d336c53bb6e8c993773d020fd23449a303855c3dd0`.
The local collector separately verified its complete public source, bridge and
original engine evidence. Its eight shipping file hashes/lengths match the
comparison report's second build. All 95 extracted portable files match their
original installed manifest. The original comparison-report archive was also
retained: artifact `12469100351`, 17,454 bytes, SHA-256
`730b91a56cd0c6999c1ab51ea077a1f7b749103898c958fdf8e37ba9a43b70d6`.

This extends exact-source **unsigned build reproducibility** to the latest
worker fix. It does not deploy that worker into the older mixed lab deployment,
qualify company-generated publication/download permissions, extend earlier
runtime/MSI results automatically to new binaries, or approve production signing
and rollout. Those acceptance gates remain open.

### Latest native MSIs on retained Server 2022, 23 October

Original `4c5accb` native MSIs were tested on the retained Windows Server 2022
Standard Evaluation Desktop Experience x64 guest, `20348.587`. A fresh owned
administrator profile completed technician 1.5.0 `e9d89ca` install, real upgrade
to 1.5.1 `4c5accb`, missing-executable repair and uninstall. All four returned 0.
The new original technician MSI SHA-256 is
`fb13e6f43656164b7d6d4f18601edb186e2f7d511791383e2a62fbd7e158acf8`.
The old product registration was removed, the new registration installed, and
the UpgradeCode remained stable. Fourteen original fixture/task/invocation,
result and MSI-log/payload records were retained and verified by byte counts
and SHA-256 values. The initial broad evidence transfer timed out; metadata-first
SFTP collected the originals without repeating any MSI operation.

The original customer MSI SHA-256 is
`b7f98073f17e32d206e6026392f1f47f540adfc60a6dae5a5f03b75a634bdc5b`.
It separately passed administrative install, damaged-license repair and uninstall,
all with exit 0 and no reboot requirement. All 96 packaged files and the agent
matched their original manifests; the harness checked the running service's
exact executable path and LocalSystem ownership after install/repair. Uninstall
removed packaged files, agent, registration and service; an independent final
observation also confirmed the service absent. Twelve original fixture,
invocation, result and MSI-log records were transferred and hash-verified.

No customer enrollment or company services existed before the technician test.
The test preserved that absence; it is not enrolled identity/consent evidence.
Persistent script policies were unchanged. These are exact-source **bare MSI**
results in retained lab installations and an owned administrator profile, not
fresh company setup, standard-user qualification, signing, company-generated
downloads, automatic updates or support-session acceptance. Server 2022's prior
standard-user policy rejection remains separately documented.

### Latest native MSIs on retained Server 2025, 23 October

The original `4c5accb` technician and customer MSIs also passed on Windows Server
2025 Standard Evaluation Desktop Experience x64, `26100.32230`, with the same
package hashes recorded for Server 2022. A fresh owned administrator profile
completed technician 1.5.0 `e9d89ca` install, real upgrade to 1.5.1 `4c5accb`,
missing-executable repair and uninstall. All four returned 0; old registration
was removed, new registration installed, and the UpgradeCode remained stable.
Fourteen original fixture/task/invocation, result and MSI-log/payload records
were retained and verified by byte counts and SHA-256 values.

Customer administrative install, damaged-license repair and uninstall each
returned 0 without requiring reboot. The harness verified all 96 packaged files,
the agent and the service's exact path, Running state and LocalSystem account.
Uninstall removed packaged files, agent, registration and service; a separate
final observation confirmed the service absent. Twelve original fixture,
invocation, result and MSI-log records were transferred and hash-verified.
A local staging template initially pointed to the technician fixture; the
existing-directory guard stopped it before file transfer or MSI execution. A
separate corrected customer path was used for the actual tests.

The fixture contained no customer enrollment or company services before these
tests. Persistent script policies were unchanged; no company-server setup,
configuration, group or authorization change was made. These are **bare MSI
lifecycle/major-upgrade** results, not standard-user policy qualification,
enrolled-device identity preservation, signed company distribution, automatic
updates or remote-session acceptance. Activation was not collected in this
bounded run. The prior Server 2025 standard-user policy rejection remains open
as separately recorded; this administrator result does not supersede it.

### Latest Server 2019 technician upgrade and unsafe-state rejection, 23 October

On retained Server 2019 Standard Evaluation Desktop Experience x64
`17763.3650`, a fresh owned administrator profile passed technician 1.5.0
`e9d89ca` install, real upgrade to original 1.5.1 `4c5accb`, missing-executable
repair and uninstall. All four returned 0; old registration was removed and the
UpgradeCode remained stable. Fifteen original task, fixture, invocation,
result, state and MSI-log/payload records were retained and hash-verified.
Before/after customer-folder file hashes, owners and DACLs independently match;
the pre-existing unsafe customer fixture was preserved. This is not enrolled
identity/consent evidence or a fresh company-server setup test.

The same original `4c5accb` customer MSI recorded for Server 2022/2025 was then
run as Administrator against that existing inherited broadly writable state.
It returned 1603, and rollback left no product registration, payload or service.
The original cancellation marker/helper bytes and all captured owners/DACLs
remained unchanged. Ten original records, including the raw MSI log, before/after
state and the initially failed test assertion, were transferred and matched
against independent guest hashes. Raw MSI log SHA-256:
`2c026c4c8a66c5612bcbf0117f755a091a21b917978448d81230bb6961c1351a`.

The initial assertion incorrectly treated `Action start/ended` scheduling
entries as actual deferred execution. Independent inspection of the retained
original log instead finds `CustomActionSchedule` for `PrepareCustomerState`
with deferred action type 3074, followed by its **actual error code 1** and
InstallFinalize failure. No actual FileCopy, FileMove, folder creation or service
installation operation occurred. Scheduling an InstallFiles action therefore
did not mean payload was copied. The MSI's embedded preparation agent was
statically extracted and matched original native agent SHA-256
`030c230b9ed43eec3eb4b4b857dd77f626b0ffe3ccf81835f63a4bc932b2606a`.
Four local verifier controls using modified log copies reject missing failure,
injected actual file copy, truncated deferred execution and an incorrect final
return code. No original log or shipping bytes were altered and no MSI was rerun
to obtain this corrected analysis; the initial failed assertion remains intact.

This proves **compiled-MSI rejection of the original unsafe state before payload
copy**, not positive customer installation on a clean current-source machine.
No replacement customer folder, standard-account file-access probe, ACL change
or company-server startup was performed. Positive clean customer lifecycle,
enrollment, signing, automatic updates and remote sessions remain open.

After the Server 2019 results and original evidence were collected, the retained
guest stopped responding to SSH; the existing viewer remained black after wake
and secure-attention input. QEMU still reported Running. The attempted normal
guest shutdown connection failed before authentication, and a normal ACPI
power-off request was sent at 01:20 UTC on 23 October. Power-off/recovery is not
yet verified at that point; no force-stop, reset, disk replacement or fixture
cleanup had been performed. After a 15-minute wait the guest was still
unresponsive. The exact disposable QEMU instance was then closed through its
monitor `quit` command after confirming its existing clean OS snapshot and all
collected test evidence. This was an **unclean guest power-off**, not a successful
Windows shutdown. Its original process completed with exit 0; read-only qcow2
checks reported zero container errors and the clean snapshot remained present.

One controlled restart subsequently restored the Windows lock screen and SSH
on the same `17763.3650` guest. Independent checks found the original customer
fixture's file bytes, owners and DACLs unchanged, the rejected product/services
still absent, and a clear Windows NTFS dirty flag. This proves bounded **basic
lab guest boot and fixture recovery** after that incident. The cause remains
unproven; it does not qualify long-term guest stability, software update recovery,
company backup/restore or complete Server 2019 acceptance. No original disk or
snapshot was deleted or replaced.

On a subsequent observation at approximately 02:10 UTC, SSH again timed out
during banner exchange. The existing console still showed the lock screen at
03:54 local time and did not advance after one viewer-provided guest secure
attention action. The attempted normal shutdown after the earlier recovery had
also failed before SSH authentication. The guest process remained present; no
second forced power-off or restart was performed in this observation. The
earlier bounded recovery result therefore does not establish sustained guest
stability. Original disks, snapshots and collected installer evidence remain
preserved.

### Approved Server 2016 management maintenance, 23 October

Following explicit human approval, the existing disposable Server 2016 company
VM completed guarded management-only maintenance. A fresh MFA session and
preflight found no active claimed support sessions. The installer worker was
stopped, and only `8-Customer-test` and `8-Technician-test` were withdrawn using
the normal `DELETE /api/v1/releases/{id}/approve` endpoint. The earlier review
draft's POST withdrawal URL was incorrect and was not executed. Withdrawal
closed the two obsolete running jobs through the server's existing behavior;
no job status was edited manually. All other release approval flags and release
metadata were verified unchanged.

The previously retained, locally trusted lab-signed management CLI, SHA-256
`cf7b5077a633f8ff9ad830d2e2a9847d16b35494bbeaf061645abbe8c37c07d4`,
created a 125,096-byte encrypted management-only backup. Archive SHA-256:
`00a0883277079052c2875aafd5167a6abf259267ed9d33d184823cde233e389c`.
A newly generated random passphrase was stored in a separate protected guest
directory, outside the archive directory. Both directories allow only SYSTEM
and Administrators. Six original guest records were collected and independently
matched against guest byte counts and hashes.

Independent in-memory validation authenticated AES-GCM, rejected a changed
ciphertext and an incorrect passphrase, checked ZIP CRCs, and matched backed-up
management key/setup/HTTPS-recipe file hashes against their live counterparts.
The database snapshot contained the expected withdrawals and closed jobs.
SQLite `quick_check` returned `ok`. The first verifier could not deserialize a
WAL-mode database into memory; its failed version was retained. The continuation
adapted only the journal-mode header bytes in a disposable in-memory database
copy. The original encrypted archive and collected records were unchanged;
decrypted contents and passphrase were not saved on the host.

The worker was restored in a `finally` block and independently observed Running.
All three original lab services were Running; management and customer service
process IDs were unchanged. Customer device identity, token hash, unattended
consent/revision and accepted release sequence were unchanged. No server/worker
upgrade, restore, transport/TLS-key export or live network change was performed.
This closes the approved maintenance prerequisite, not full Windows deployment
backup/restore, migration recovery or coordinated upgrade acceptance.

### Coordinated Server 2016 management/worker upgrade, 23 October

The same disposable deployment subsequently upgraded both management and worker
to exact reproduced source `4c5accb7ae73615c799a43fa70dc2a49aeed0822`.
Original unsigned input SHA-256 values:

| Input | SHA-256 |
| --- | --- |
| Management | `43a0002c9c17372bf8958eb91ae27092b71434adc9441d9da4e6a19d30618bc3` |
| Worker | `9624a05cae857d48d02270a7128992faa3522903801a7fce3cbbb32996f3ef12` |

Both came from the previously verified independent native build `39867956814`;
the earlier public comparison established all eight outputs identical to the
first build. The existing private lab publisher signed these inputs without
exporting its private key. The first signing attempt failed because the guest
could not reach DigiCert's timestamp endpoint; the failed attempt and logs were
preserved. Retrying with Sectigo's documented RFC3161 endpoint succeeded, with
the unchanged signer requiring `signtool verify /pa /all /tw` and valid
Authenticode. See [Sectigo timestamp documentation](https://www.sectigo.com/resource-library/time-stamping-server).
These remain **private lab signatures**, not approved project production signing.
Installed signed management SHA-256 is
`8808af5c56b4e0c90b63a934f69c4f83f9816a4294b4a76f4a847848e96e08d0`;
signed worker SHA-256 is
`59e3d4354af0b529a10e061ebc39a0573226e44ea06ce356d44c491bd5742664`.

Independent inspection of the encrypted pre-upgrade snapshot found schema
version 2, matching the target, so no schema migration was needed. The guarded
replacement retained both old binaries and the worker receipt, stopped the
worker before management, and checked verified HTTPS health before starting the
new worker. The first attempt falsely rejected a renewed signed profile because
the harness compared the entire envelope: the profile endpoint refreshes issue
and expiry timestamps on every request. That failure exercised binary rollback;
both original hashes and all Running service states were independently observed
after restoration. It was a harness assertion failure, not evidence of changed
company settings or a shipping runtime fix.

A separate retained second attempt compared all configuration fields apart from
the validated renewable timestamps and completed at 09:47 UTC. The authenticated
worker recipe endpoint returned
`6c5691a926cdea242020ca10df46ba2ee4d4e3456a93fa4e14e64cb8831aadf0`,
matching an independent calculation from the pinned source's seven bundled text
files. Fourteen original signing, journal, script, result and profile records
were collected and matched against independent guest hashes. Both original
before/after profile signatures verified against the worker's existing pinned
Ed25519 public key; their complete configuration matched, revision remained 18,
and updates remained paused. The initial collector omitted explicit TLS 1.2
selection for its observation; its failed version was preserved, and a corrected
observation verified the already-collected originals without repeating transfer
or the upgrade.

All three original services were independently observed Running on the installed
hashes, with no active builds. Customer identity/token hash, consent/revision and
accepted release sequence were preserved, and its service process was unchanged.
Release approvals, service environments, Caddy configuration and transport trust
keys were unchanged. The transport/HTTPS component binaries were not replaced;
management's normal component restart used the existing configuration. No host
services, firewall/DNS settings, peer bind change or public endpoint was changed.

This closes the **paired lab binary upgrade and recipe endpoint** prerequisite.
It does not yet prove new company-generated bundles upload/download correctly,
installation of those bundles, automated endpoint updates, full deployment
restore, schema migration rollback or public direct/relay operation.

### Current-source company bundle jobs, 23 October: completion unconfirmed

The original independently reproduced `4c5accb` customer and technician MSIs
and configuration agent were staged and signed using the existing private lab
publisher and the same verified timestamp policy. Signing the outer MSIs did not
alter their embedded payloads or actual MSI ProductVersion `1.5.1`. Signed input
SHA-256 values:

| Input | SHA-256 |
| --- | --- |
| Customer MSI | `e7387695be27942849fe345308dcf32260b439c79a12e00205f4e5173bcb0545` |
| Technician MSI | `1b22ef4726db3cd0d8aad084afd564976c828565804cf49118d4a196086733bd` |
| Configuration agent | `cfab98c90b1ae7ac5ba1ee2111d4381ce54de48fda8204806556d3d92aa973b7` |

The existing owned lab release key signed exact metadata for sequence 9, test
channel, version `1.5.1`, pinned public source `4c5accb`, and the original payload
manifests (96 customer files and one packed technician executable). No private
release key was transferred to the worker. These test declarations are scoped
to Server 2016 for this run and do not qualify that OS or other Windows versions.
Updates remained paused at company revision 18.

Normal server import succeeded for both new releases. Before approval, build
requests returned 409 for each; after approval, exactly one job per edition was
queued: customer `23755146-ed3c-4577-8c80-e6efb16d1395`, technician
`7b271526-963b-4f96-ac43-0ba66c550142`. The obsolete sequence-8 releases remained
withdrawn. No installer was executed or endpoint update enabled.

The first status observation subsequently exceeded its 240-second SSH observer
timeout, and a separate small SSH probe timed out during banner exchange. The
existing VM process remained present and its read-only monitor reported Running.
The console woke to the desktop; a subsequent Task Manager shortcut had not
produced its window by the next observation. This does not establish the cause
or terminal state of either build. No duplicate job, service restart, VM reset,
forced power-off or disk replacement was performed during these observations.
Original jobs and retained outputs must be inspected after the wait before any
recovery decision. Successful bundle upload, public customer download, protected
technician download and installation of either generated bundle remain unproven.

### Bundle-test interruption and same-disk recovery, 23 October

After the explicit fifteen-minute observation wait, SSH still failed and the
company guest display did not wake. A normal ACPI shutdown request at 10:20 UTC
did not stop the existing VM during its grace period. Monitor `quit` was then
requested, followed by termination of that exact owned emulator process when it
also failed to exit. This was an **unclean guest power-off**. The original disk
was retained; offline qcow2 checking reported zero container errors. No build
record was edited, duplicate job submitted, database restored or disk replaced.

The same company disk booted again at 10:42 UTC with a private lab allocation
of 3 GiB RAM and two virtual processors, compared with the former 2 GiB/one
processor. The old private settings and launcher were preserved. SSH returned,
and an independent administrative check at 10:58 UTC matched device ID, token
hash, unattended consent/revision and accepted release sequence against the
pre-maintenance record. This proves bounded identity/consent persistence through
this incident, not complete service recovery or an interrupted endpoint update.

The company service was observed Stopped with Win32 exit code 1. The worker log
repeatedly reported refusal at the HTTPS recipe endpoint. Original Caddy,
rendezvous and relay logs were collected through SFTP. The 1,049,730-byte Caddy
log, SHA-256
`0f4f21d782fb2e55a64e63df9615d20febdfe03ba28dcf784597b0ac88b95a65`,
contains access violations and Go runtime fatal faults in the pinned Caddy
2.10.2 component, including TLS/runtime stack frames. It also contains older
incidents; its contents do not by themselves establish the cause of the latest
guest-wide communication interruption. A subsequent service-start SSH connection
failed during banner exchange, before a verified service-start action. Job
terminal states and generated downloads must still be observed through a healthy
server. Increasing guest capacity has not established sustained stability.

The separately retained, already unresponsive Server 2019 test guest was also
powered off uncleanly after normal shutdown and emulator-exit attempts failed.
Its original disk, clean snapshot and collected installer evidence were retained;
offline qcow2 checking reported zero container errors. It was not restarted or
replaced with another company deployment.

Independent endpoint work prepared private latest-source Windows 11 fixtures for
the actual technician 1.5.0-to-1.5.1 upgrade and customer MSI lifecycle; neither
was executed at this point. A separate fresh Server 2016 endpoint-only seed was
prepared with distinct lab access keys. Its KVM setup stopped at environment
preflights before any Windows installation: the minimal owned Linux runtime
lacked optional helper commands. A shell-only continuation preserves the partial
fixture and equivalent kernel read-only-mount checks. This preparation is not
Windows installation or compatibility evidence.

A supported storage move of the stopped, owned endpoint-lab WSL distribution
from D: to a protected C: directory is being verified separately. All emulators
inside that distribution were checked absent first; other distributions and the
default registration were not selected for modification. VM resume is gated on
the completed move, unchanged registration and matching entire-VHD hashes. The
prepared Windows 11 resume keeps its original 4 GiB guest allocation, 6 GiB host
memory guard and 100 GiB reserve on the actual virtual-disk volume, and requires
its existing TPM state file. A company-VM migration remains pending fresh human
approval because the earlier automatic review blocked company backup/restore/
migration work. No company disk was copied into a new hypervisor environment.

### Approved cold migration preflight, 23 October

The human subsequently explicitly approved migrating the existing disposable
company VM to KVM, retaining the original disk and backup, preserving its company
configuration and HTTPS binding, and never running both copies together. This
approval does not establish that the migration succeeded.

The original company emulator remained live after more than 30 minutes following
the recorded normal shutdown request. A monitor exit request also failed to
terminate it. After a further grace period, only that verified emulator process
was stopped. Its original qcow2 disk remains on D:; a subsequent offline image
check reported no errors. This was an unclean shutdown, not proof of guest
filesystem or service health. The company disk has not yet been copied or booted
under KVM.

The separate endpoint-lab WSL storage move's command observer timed out. Windows
now registers the original distribution identity at its C: destination, and the
original VHD path is absent. The destination has the expected 25,041,043,456-byte
length, but its full-file SHA-256 differs from the pre-move value. The VHDX image
check reported no errors and its dirty flag was false; neither result explains
the hash difference or proves the retained guest contents unchanged. All guests
in that distribution remain stopped, its resume verification gate remains
unsatisfied, and read-only offline investigation is in progress. No move was
repeated, no registration was manually rewritten, and no hash guard was relaxed.
The unrelated working host services remain untouched.

Offline parsing of the relocated endpoint VHDX found ext4 state `1` (clean) and
readable retained Windows 11 disk, firmware-variable and TPM-state files. Their
current hashes were retained privately. No corresponding pre-move hashes of
those individual files were available to prove equality, so the whole-VHD
verification gate remains unsatisfied and that distribution was not booted.

The approved company migration is proceeding separately in a newly imported,
minimal owned Linux runtime on C:, using the already verified Alpine rootfs and
pinned QEMU 11.0.3 packages. Existing distribution identities and the default
registration were verified unchanged. This is storage for the existing company
disk, not initialization of another company backend. The first streaming copy
failed with a host `MemoryError`; its partial disk and diagnostics were retained,
and it was never booted. A subsequent copy uses direct writes to bound Linux file
cache growth. Native company launch scripts are guarded, and a Windows reader
handle denies writable access to the original disk while copying. Independent
source/target hash equality and an offline qcow2 check are required before the
copy can be committed or booted. Copy completion, KVM boot, service recovery and
post-migration identity checks remain unverified at this entry.

### Company disk migration copy verified, 23 October

The live pipe-based direct copy was deliberately stopped after its measured
throughput proved unsuitable; its incomplete disk remains retained. A subsequent
mount preflight correctly refused to copy when drvfs ignored `ro`. The continuation
used a separate bind mount, verified the kernel mount's read-only flag, held the
original disk against Windows writers, and retained all previous partial copies.

The complete 29,104,275,456-byte company disk was copied into the separate protected
Linux runtime on C:. Independent host-source and Linux-target SHA-256 values both
equal `fbe14b2cf543e2c341bac8b87d1bd5cfc5c76f7dde265a7e886716b172381da2`.
The copied qcow2 image check reported no errors before its final name was committed.
The original disk and encrypted backup remain retained. The original native boot
helpers have a migration guard; the KVM boot wrapper also holds the original disk
against writable access for its entire lifetime.

At 13:26 UTC, the verified copy was launched using pinned QEMU 11.0.3 under KVM,
with 3 GiB memory, two virtual processors, the original default e1000 MAC, installed
disk boot, and the existing loopback forwards. No company setup, key replacement,
restore or HTTPS-binding change was performed. Successful guest boot, service
health, preservation of live company state and the two original build-job results
still require post-boot observation; a successful disk copy alone proves none of
those acceptance requirements. The separate endpoint VHDX hash discrepancy remains
unresolved and that distribution remains unused.

### Migrated company API and original installer bundles, 23 October

The same company guest responded over its existing pinned SSH access after KVM
boot. At 13:28 UTC, customer, management and worker services were Running. Checks
against the original pre-maintenance invariant preserved the customer device ID,
token hash, unattended-consent value/revision and accepted release sequence. The
Caddyfile retained its original loopback bind; its live SHA-256 was
`72afc6ad51fffacc41e436de01bb819696c4870129725fa7850631b7b10f748a`.
No company restore, new setup, transport-key replacement or endpoint-binding
change was performed.

A fresh login using the existing local administrator and TOTP created a session
at 13:28:58 UTC. A subsequent bounded API-only observation at 13:33:55 UTC returned
health `ok`, revision 18 with updates paused, all three services Running, and both
original job IDs in `completed` state. No replacement jobs were queued.

At 13:34 UTC, real HTTPS downloads verified:

| Edition | Anonymous request | Authenticated download | Bytes | SHA-256 |
| --- | --- | --- | --- | --- |
| Customer | 200, downloaded | Not required | 32,481,317 | `d03f13ac895f3e1bb424b021628c31cb68357c34010199572887b6d777b73568` |
| Technician | 401, denied | 200, downloaded | 30,276,919 | `c43f529426b7a1829d1a4bbe06ecd67b86851d3d70b299c96ce0ede09f1fd6c6` |

Both downloaded hashes matched the corresponding completed job result. Customer
identity, consent, release sequence and service process remained unchanged during
these checks; revision 18 and paused updates were rechecked afterward. Six original
guest records, including both ZIPs, were collected over pinned SFTP, with the ZIP
sizes and hashes verified again on the host.

Independent host verification of both original ZIPs established exactly 12 unique
flat entries, valid CRCs, no directory or symlink entries, and seven bundled
scripts/license texts byte-identical to normalized pinned source `4c5accb`.
Bootstrap had exactly six public fields and matched the existing company and
release trust pins. Both signed envelopes verified against those pre-existing
pins; changing signed payload bytes failed verification. Profile revision 18,
paused updates and valid lifetimes were checked. The release envelope matched the
original imported envelope, version 1.5.1, sequence 9, test channel and Server 2016
scope. MSI and agent hashes matched that signed metadata. The customer manifest
contained 96 installed-file entries; technician contained one. The initial host
verifier incorrectly assumed the import request itself was an envelope; its
failure was retained, and the corrected check compares its actual `envelope`
member. No package was rebuilt or changed to satisfy that check.

The diagnostic helpers also exposed an observation problem: Windows PowerShell
serialized filesystem metadata attached to log lines into a very large JSON
response. A later compact observer encountered a live-log sharing violation. Both
failed observations were retained. The successful API-only observer omits log
objects and limits output size. These helper failures do not prove that the
earlier Caddy faults or guest freezes share the same cause.

These are company-lab-signed packages, not trusted production project releases.
The generated bundles have not yet been installed on a fresh endpoint or a new
technician profile. Internet reachability, separate-machine sessions, automatic
rollout and the remaining Windows compatibility gates are still unproven.

### Generated technician bundle: installed-signature rejection, 23 October

The original technician ZIP was used unchanged in a newly created owned
administrator profile on the existing Server 2016 guest. Its first scheduled
attempt reached its time limit without a result. An earlier diagnostic SSH
timeout had left the exact read-only MFA observer running inside the guest,
expanding filesystem log metadata during JSON serialization. The orphan consumed
over 2 GiB and approximately 579 CPU seconds. Its executable, PID and encoded
command SHA-256 were checked against the retained helper before stopping only
that observer. No product service was targeted. This establishes a diagnostic
resource problem; it does not resolve the earlier Caddy access violations.

Inspection retained the prepared technician state at profile revision 18 and
release sequence zero. A guarded continuation ran the original bundle setup.
It verified the installer, prepared the durable setup marker, installed the MSI
files and copied the signed agent, then failed final installed-publisher
verification. The MSI and agent both verified as trusted in the original and new
profiles; Microsoft SignTool and the agent's separate `verify-package` command
also passed. The failure was therefore not an MSI trust rejection: the installed
packed executable still had its original unsigned source hash
`cede4bad34a29a78a70d627b751e949b51db02cb95b324aaffa1173f2499fa0a`
and Authenticode status `NotSigned`. The installed agent had the expected signed
hash and status `Valid`. The pending-install marker remained and sequence zero
was not advanced. A later guarded retry refused the changed, pending installation
state before attempting another install. No marker was deleted and no signature
requirement was bypassed.

This exposed a lab-input preparation error: signing the original unsigned CI MSI
wrapper did not sign its embedded application. Both existing MSI builders require
signed payloads by default; the CI `UnsignedTest` artifacts remain unsuitable for
a complete signed release. Seventeen original scripts, result records and logs
were collected over pinned SFTP and independently hash-checked. The account
password was excluded from collection. The original customer identity/consent,
its service and the company/worker services still passed their separate checks.

Corrective staging signed a separate copy of the pinned technician executable
using the existing non-exportable company lab key and RFC3161 timestamp
verification. Its signed SHA-256 is
`12ce1f1cef84eb3c4c121b3b3922faf13fa6cf3aef4573d5f0d841daf67204e5`;
the original failed-install file was unchanged. The planned MSI retains software
version 1.5.1 and its existing stable product identity. Seventy-nine build inputs
from exact source `4c5accb` and the pinned WiX 5.0.2 supplier MSI were transferred
and hash-verified. Build preflight stopped before executing WiX because this VM
has .NET Framework release 394802 (4.6.2), below WiX's 4.7.2 requirement. Microsoft's
[.NET Framework 4.8 offline installer](https://dotnet.microsoft.com/en-us/download/dotnet-framework/thank-you/net48-offline-installer)
is being prepared for this lab build prerequisite; installation and a corrected
MSI build have not yet been verified. No signing key was exported and no
production publication is approved.

### Correctly signed technician bundle and explicit setup recovery, 23 October

Microsoft's official .NET Framework 4.8 offline installer was downloaded once;
its 121,346,568 bytes had SHA-256
`0a3a390c47e639d0f7fc65b21195fee6b7f65b066f80f70c60fab191d14b7e40`.
Its Microsoft Corporation Authenticode signature was verified on the host and
again after transfer to the owned guest. It was not executed on the host. Guest
preflight excluded active support sessions and worker jobs. The installer returned
3010 and the old runtime registry value remained until the required normal guest
reboot. Post-reboot observation verified .NET release 528049, all three Swan
services Running, unchanged HTTPS configuration, and preserved original customer
identity, consent and accepted release sequence. The pre-reboot result was retained
rather than rewritten as a completed runtime verification.

The pinned technician builder then ran with its default signed-input checks,
without `UnsignedTest`, on exact source `4c5accb`, WiX 5.0.2 and release epoch
1792703512. Both executable and agent were already signed by the same lab
publisher. Prepared unsigned MSI SHA-256 was
`208401a37ca44af99d4c1e4c50f35b397dc07bbcfe9b78939796d44acad446d4`.
After signing and timestamp verification, MSI SHA-256 was
`b777a37508aa4aab91c851274f645df9bd6b22cc997696090ebfd2f62b382e3b`.
Software version remained 1.5.1, ProductCode remained
`{D32FA420-4F84-50A6-BAB2-CBFDA73F87C9}`, and UpgradeCode remained
`{A4374699-436F-4917-9A4E-223F62A9E634}`. No fake software-version increase or
company-dependent product identity was introduced.

A separately signed test release, sequence 10, referenced that MSI and its signed
installed executable. The server rejected an unapproved build with 409, then
approval and one build request created job
`1a744e32-8439-45b1-a8f1-0e268efca717`. It completed normally. Anonymous technician
download returned 401; authenticated download returned 200. The 30,296,131-byte ZIP
had SHA-256 `97e397701511465e80c92386c91ee97c80e4115b83aa6ac5d11b069a8c518896`,
matching the job result. Independent host inspection verified the 12-entry layout,
CRCs, seven exact pinned-source texts, six public bootstrap fields, original trust
pins, signed profile/release envelopes and artifact hashes. Profile revision 18
and paused updates remained unchanged.

At 14:50 UTC, the existing owned administrator profile invoked the original
bundle's `Install-Company.ps1` with explicit `ReplaceFailedSetup`. Its preflight
required the exact pending sequence-9 setup, zero recorded release sequence and
existing MSI registration. The normal program archived that original marker
unchanged, performed the same-version MSI repair with signed payloads, verified
installed files and publishers, retained the signed installer cache, wrote the
exact signed sequence-10 release envelope and removed the active setup marker.
The installed agent's separate `verify-installed` command returned zero. MSI
registration and the user shortcut were present. Twelve original records and
installed/cache artifacts were collected over pinned SFTP and hash-checked;
independent verification matched the exact bundle/cache/agent bytes, executable
hash, both signed state envelopes and the byte-identical archived sequence-9
envelope. An initial collection-query syntax error was retained and corrected;
neither the installation nor its result was rerun for collection.

The original customer identity, consent, service and company/worker services were
rechecked afterward. This result proves recovery of this failed setup in an
administrator profile with the lab publisher explicitly trusted. It does not prove
a fresh-machine or standard-user installation, graphical technician login/session,
customer enrollment, automatic update rollout, public reachability or production
signing. A corrected signed customer package and the other release gates remain
outstanding.

### Signed customer bundle and failed downgrade attempt, 23 October

The 95 unsigned customer payload files were verified against the independently
reproduced `4c5accb` artifacts. The application was signed separately using the
existing non-exportable lab key; signed application SHA-256 is
`9ec538bc34b50f1f84fe48561f664f9f572eeaa41c5f2aa48a6ff39bb90138b3`.
Python 3.12.10's official portable build runtime was checked against its published
archive checksum and its core executable/DLL Authenticode publishers, then used
only inside the lab build folder. The original payload was retained.

The pinned customer builder ran without `UnsignedTest`, using the already signed
application and agent. Its manifest covered 96 installed files. Compared with the
original unsigned CI manifest, the application hash changed for signing, and the
license used exact Git LF text where the CI copy used CRLF. The license texts were
compared equal after that explicit conversion; no original shipping artifact was
normalized or changed for a reproducibility comparison. An initial manifest check
that assumed only one changed file was retained and corrected before metadata was
signed or imported.

Prepared unsigned MSI SHA-256 was
`a8e9ba3d45d61d0d990468b5f7d641dc1b2c15356ef440e1b377a29793ddec74`;
signed MSI SHA-256 was
`78613eafa2cc56fed53b2a550b2aec7db10b82733d8cd6f781b26bb34218b47c`.
Version 1.5.1 and stable customer product/upgrade identities were checked unchanged.
The signed test release used sequence 11, unchanged company trust pins and Server
2016 scope. Its unapproved build returned 409. After approval, original job
`425fc1b5-cc46-49a1-9877-dd545cd9f61f` completed. Anonymous HTTPS download returned
200 with 32,492,051 bytes and SHA-256
`ec13dd18d1bb3b33da872966430ffd15c75932a4208e7d52801a85291b3cc64c`.
Independent host verification checked the original ZIP's 12-entry layout, CRCs,
seven pinned-source texts, public-only bootstrap, pre-existing profile/release
pins, signatures and complete signed installed-file manifest. Eight original
download/build records and the archive were retained.

Pre-upgrade checks found Server 2016 Standard Evaluation Desktop Experience
14393.693, approved enrollment, profile revision 18, release sequence 8, consent
false at revision 6, no active support session or recovery marker, and no Tailscale
service. The original generated setup was run as SYSTEM. At 15:29 UTC it failed
with MSI 1603; Windows Installer's event identified its newer-version launch
condition. Read-only inspection established that the old MSI is registered as
1.5.7 with ProductCode `{32063221-0D0B-4749-B725-5E4504E5B2A9}`, despite application
file version `1.5.0+68`. The new 1.5.1 product was not registered and the original
application hash, identity, credential, consent and sequence were unchanged. No
version was inflated and no downgrade condition was bypassed to force a pass.

This failure exposed a separate installer defect: setup had already stopped and
disabled its existing configuration task. The original validated action and signed
agent were restored to Running, while the pending sequence-11 marker was retained
unchanged. Customer, company and worker services were Running. Nine original
failure/restoration records were collected over pinned SFTP and hash-checked;
device credentials were excluded. This is a failed customer-upgrade test, not
fresh enrollment, a successful upgrade or complete rollback.

The source fix now rejects newer related MSI registrations, and incompatible
same-version registrations with different product identities, before preparing a
setup marker or stopping configuration syncing. If a later setup step fails, it
restores the prior enabled/active task state only after validating the current
task's action and SYSTEM principal again. Restoration failure is reported together
with the original setup error; pending installation evidence is retained.
Windows PowerShell 5.1 checks passed ten isolated read-only MSI catalog cases,
seven actual-handler scheduler-substitute cases, and the existing real setup
entry-point/trust-rejection checks. These are bounded regression checks, not
successful product installations. The new preflight was also executed against
the real lab MSI catalog: it rejected registered 1.5.7 versus candidate 1.5.1 with
no MSI actions, unchanged pending-marker bytes and the original watcher Running
before and after. Both Windows CI workflows now include the regression checks.
Matching server/worker/agent builds must be regenerated because bundled source
texts and their recipe changed.

Fresh customer testing needs the previously reviewed private guest HTTPS binding
change. A new explicit approval request is pending; the earlier automatic review
blocked that change, and the separate approved migration preserved the binding.
No replacement listener, proxy, host firewall change or DNS change was used to
work around it. Other full release gates remain open.

### Installer-fix build evidence and clean endpoint preparation, 23 October

Company component run `39983860542` passed for source
`d8fad20408281c5936425da0e37ccb7606f1d2f7`. Its Windows artifact archive was
downloaded without execution and verified against GitHub's SHA-256 digest
`16778d74d7a5b8c1338de45335ddcafaa3bbe002db77dfeeb70f24c5bba4e1d6`.
All component and source-archive hashes matched `SOURCE.json`. The corresponding
source archive contained exactly the expected 156 tracked inputs: 14 matched
public Git blobs byte for byte, and 142 differed solely by Windows checkout CRLF
line endings. Both raw hashes were retained for those differences; archived
bytes were not rewritten. This verifies source correspondence, not independent
component reproducibility, installed behavior or production signing.

Independent Windows application runs `39984444925` and `39984452304` were still
building at the observation. Their bridge jobs passed, but application binaries
and a comparison of their shipping bytes were not yet available. The unrelated
successful comparison-workflow trigger is not proof of these builds matching.

General CI run `39983870640` failed because the installer-wait test spawned the
standalone name `tests::child_fixture` inside the application crate, where the
actual name includes `install_wait::`. The child therefore ran zero tests and
returned success instead of the expected failure code. A test-only fix derives
the module prefix in both contexts. Focused regression run `39988354836` then
passed on Windows 2022 and Ubuntu 24.04: each platform compiled both standalone
and embedded contexts and passed all three tests in each, using actual child
processes and verifying failure status and the minimum wait. Full application
CI and acceptance remain separate gates.

A distinct endpoint-only Server 2016 evaluation VM was provisioned with a new
32 GiB disk, separate lab credentials and pinned SSH keys in the healthy KVM lab
runtime. The existing company VM and frozen Windows 11 storage were not replaced
or booted respectively. Host storage reserved the full new disk plus 100 GiB
before launch. Windows setup visibly reached 69% of file preparation; later
disk activity continued, but the provisioned SSH endpoint and final Windows
installation had not yet been verified. No company management service was
started in this endpoint fixture. Its installation is not a passed product test.

### Clean Server 2016 signed customer lifecycle, 23 October

The separate endpoint reached the Windows desktop. Its original specialize
command returned `0x1` before creating the lab directory; the original cause
was not established. Default Windows management access, forwarded temporarily
through loopback only, allowed the owned endpoint to be identified and its
pinned seed scripts to be verified. A bootstrap recovery installed the existing
pinned OpenSSH payload but stopped at its combined key-permissions command.
The exact existing key bytes were retained, private key files were restricted
to SYSTEM and Administrators, and key-only SSH was verified against the
pre-provisioned host key. Automatic logon was disabled and its saved password
removed. The temporary management forwarding was then removed. This repaired
only endpoint lab access; the company VM HTTPS binding was not changed.

On Server 2016 Standard Evaluation Desktop Experience 14393.693 x64, the
corrected source-`4c5accb` company-lab-signed customer MSI
`78613eafa2cc56fed53b2a550b2aec7db10b82733d8cd6f781b26bb34218b47c`
passed fresh installation, repair of a deliberately damaged packaged license,
and uninstall as SYSTEM. All three MSI exit codes were zero. Each installed
phase verified all 96 manifest files, the exact agent hash, service executable
and SYSTEM ownership, and valid application/agent signatures from the expected
disposable lab publisher. Uninstall removed packaged files, product registration
and the customer service. Thirteen original scripts, reports and MSI logs were
collected through pinned SFTP and independently hash-checked, including hashes
recorded by the phase results. This is a passed signed bare-MSI lifecycle on
this exact Windows build, not company enrollment, interactive UAC, automatic
update, remote desktop or complete Server 2016 qualification. No project signing
approval or production publication is implied.

The corrected signed technician MSI was also attempted from an owned standard
account. Task Scheduler initially denied batch logon (`0x80070569`), so only that
account's batch-logon right was added; before/after exports verified all other
user rights unchanged and the account remained outside Administrators. The
resulting actual standard-user install attempt failed with MSI 1601; its original
log records connection to the installer service denied with `0x80070005`.
Four original failure records were collected and hash-checked. A separate
credential-based interactive-token process launch produced no lifecycle result;
its launcher's null child exit code is not a pass. Installer policy was not
relaxed. Standard-user technician installation on this VM remains unqualified
and requires further diagnosis in a normal logged-on user session.

### Installer-fix independent rebuild verified, 23 October

Native Windows runs `39984444925` and `39984452304` both completed successfully
for source `d8fad20408281c5936425da0e37ccb7606f1d2f7`, using the same pinned
source-built Flutter engine candidate. Comparison run `39995075014`, executed
from frozen comparison-tool source `89039772db027868eecb4de3217fa3dbd9fb8c79`,
verified both corresponding source archives (1,160 tracked source inputs and
six generated bridge files each) and found all eight original shipping files
byte-identical. No shipping-file normalization or executable execution was used.
The original report is retained in
`deployment/windows/native-d8fad20-rebuild-comparison.json`. Its SHA-256 is
`d339ad4e039af68e04f93da38c105bc21697f0fca418b9843ff28e4504caaec1`.
The retained first-build archive additionally matched GitHub's digest
`c67a287094a504220626c7f4d88a12e76fb67a607fdb4de955ffe68c240bc960`
and every first-build output in the comparison report. This is reproducibility
evidence for the exact unsigned candidate, not runtime or release approval.

The same three management/worker/agent input files also matched the independently
compiled Windows company-component run `39983860542` byte for byte. Copies were
signed with the existing non-exportable disposable-lab publisher and verified
using timestamp-required SignTool checks and Windows signature validation.
Eight original signing/provenance records and signed files were retained and
their signed binary hashes checked independently. Company services were not
changed; coordinated installation and end-to-end testing remain outstanding.

The original Linux deployment job `120517833123` in run `39983860542` was retained
and its successful steps inspected. Actual isolated Docker tests passed fresh
setup/persistence, MFA/replay checks, enrollment approval, HTTPS/profile signing,
and complete encrypted backup recovery into replacement containers, with both
pinned upstream and maintained transport components. Their logs explicitly leave
native sessions, UDP traversal, external reachability and public ACME unverified.

To investigate the standard-user MSI failure, the disposable endpoint was also
prepared for loopback-only RDP lab control with NLA retained and the owned test
account outside Administrators. No successful standard-user desktop login was
demonstrated: Windows UI automation could observe owned login/file dialogs but
did not reliably target their controls. Temporary host credential entries were
removed after verifying their exact owned account, comment and session-only
persistence; existing host credentials were not replaced. This lab-control path
is not evidence of Swan connectivity. The standard-user technician test and the
separately pending company HTTPS-listener approval remain open.

### Existing company upgraded and matching bundles generated, 23 October

A fresh authorized management-only backup was created before installing the
`d8fad20` management/worker pair. Its encrypted archive is 160,757 bytes with
SHA-256 `df03fdfe6c8d57e89517e8ff4ab8f717a1747471f030079c52fd2b62e9850876`.
Independent in-memory verification authenticated AES-GCM, rejected tampering and
the wrong passphrase, checked ZIP integrity and SQLite `quick_check`, and matched
backed-up management material and release/job state. The original archive was
unchanged; only a disposable in-memory SQLite copy's journal-mode header was
adapted for deserialize. The passphrase remained separately protected in the VM
and was not saved on the host. No transport/TLS export or Windows restore was
performed, and no additional releases were withdrawn.

The existing Server 2016 company VM then completed a guarded coordinated upgrade
to the matching lab-signed `d8fad20` management service and installer worker.
Independent observation verified installed hashes/signatures, the original
profile trust pin and both before/after profile signatures, unchanged profile
revision 18 with rollout paused, and the new matching bundle recipe
`3c38e520a233282bd1ae06739b4bd6cdc8409439f7ac46c11edc837e1d71c2d5`.
Customer identity/token hashes, consent, release sequence, process ID, running
configuration watcher and the original pending sequence-11 marker were preserved.
HTTPS configuration bytes, transport keys, service environments and release
approvals were unchanged. All three services were Running; the previous signed
management/worker binaries were retained and independently hash-verified for
rollback. This successful upgrade did not exercise rollback or restore.

Matching customer and technician MSIs were built using the exact public source,
reproduced native payloads, and 213 verified inputs. The existing pinned WiX 5.0.2
and Python 3.12.10 inputs were reused without downloading them again. Default
signed-input checks were used, with the application and agent signed before MSI
construction and the resulting MSIs separately signed and timestamp-verified.
All original build inputs remained unchanged and no private key was exported.
Fifteen original build/signing records and outputs were collected and checked.
The customer's 96-file manifest matched all 94 unchanged native payloads, its
new signed main executable, and the exact public-source license text. Read-only
MSI inspection confirmed version 1.5.1, x64, and the stable product/upgrade IDs
for both editions. The lab-signed MSI hashes are:

- Customer: `1b804b8c008cc1e164f88791c2192c4acd1814b4c76446a812a6c995f1ae5d31`.
- Technician: `d0bcede9234a90f272ef34a71f734cf4f1813b2fbcc2fbe376208061a5a84bda`.

The upgraded server rejected build requests before release approval, then its
worker completed customer sequence 12 and technician sequence 13. The customer
bundle downloaded anonymously with HTTP 200; technician access returned 401
without authentication and 200 with it. Both original ZIPs matched their completed
job hashes. Each contained exactly 12 safe flat entries, the seven exact pinned
bundle texts, only six public bootstrap fields, the original signed release
envelope, and the expected signed MSI/agent hashes. Profile and release signatures
were independently verified against preexisting pins; modified signed payloads
were rejected. These are isolated company-server distribution checks with rollout
paused and Server 2016 test-channel metadata, not external reachability, fresh
customer enrollment, successful endpoint upgrade or production publication.

The complete generated sequence-12 customer setup was also invoked once against
the earlier contaminated 1.5.7 MSI fixture, with explicit failed-setup replacement
requested. Its bundled MSI preflight rejected candidate 1.5.1 before preparing or
replacing installation evidence. The initial harness expected the generic outer
error, but Windows PowerShell surfaced the native child's version error first.
That original error was preserved. A read-only continuation and independent
collection confirmed identical before/after snapshots: identity/token hashes,
consent, sequence 8, payload hashes, service process, Running watcher, pending
marker, cancelled archives and MSI registration were unchanged. The candidate
was not registered. Setup was not rerun. This verifies the actual generated
installer's negative version guard; it does not resolve the old interrupted
fixture or establish a successful customer upgrade. Positive endpoint installation,
standard-user technician qualification and the other full release gates remain
open.

The original generated sequence-13 technician bundle subsequently completed an
upgrade in its existing owned administrative Windows profile. Its accepted
sequence advanced from 10 to 13, both installed executables and the cached MSI
matched the signed bundle, and installed-state verification succeeded. Company
bootstrap/trust, profile revision 18 and the earlier cancelled-setup archive
were preserved; no pending marker remained. Thirteen original evidence files
were independently collected and checked. The customer service, watcher and
pending marker were unchanged. This is not standard-user, technician login or
remote-session qualification.

A separate Windows profile exposed another preflight defect: Windows Installer
reported state 2 for the technician product installed under the other account.
Microsoft documents this as INSTALLSTATE_ABSENT, not corrupt registration.
The helper now allows installation in that state only for an ordinary per-user
MSI (missing or empty ALLUSERS); machine-wide and dual-purpose packages still
fail closed. Related-product downgrade and conflicting-version guards remain.
Sixteen isolated catalog cases and seven watcher-recovery checks pass. A read-only
probe against the actual signed MSI/catalog selected installation for the new
profile, still rejected the newer customer fixture, and preserved both existing
installations and customer evidence. No MSI action ran and no distributed bundle
was patched during that probe. A rebuilt matching bundle and positive second-user
installation remain required.

The retained signed d8fad20 customer MSI also completed a bare-MSI reinstall,
damaged-license repair and uninstall on SWAN-END-16, Server 2016 Standard
Evaluation Desktop Experience 14393.693 x64, running as SYSTEM. Each operation
returned 0; installed publishers and all 96 packaged file hashes were checked.
Thirteen original logs/reports were collected and independently hash-verified.
The endpoint had previously uninstalled the 4c5accb customer package: its only
remaining customer state was the intentionally retained zero-byte
pending-uninstall marker. The original clean-state preflight rejected that
directory; a separate guarded preparation allowed exactly that existing marker
without deleting it. This is repeated-install/removal evidence, not a first
installation on a new disk. It does not demonstrate enrollment, cancellation
clearance by complete company setup, GUI, updates, or the later f84ffb0 helper.

The signed d8fad20 technician MSI was then attempted from the actual
SWAN-END-16 standard-user desktop, session 2, using an Interactive/Limited
scheduled task without stored task credentials. Installation returned 1625.
The original MSI log accepted the package signature at the unrestricted
software-restriction level, then reported effective DisableMsi=1 and explicitly
rejected a non-assigned application for a non-administrator. Both
AlwaysInstallElevated policies were 0. Four original failure records were
collected and hash-verified. This establishes an installer policy restriction,
distinct from the previous batch-logon 1601 failure; it is not a successful
standard-user installation. No policy was relaxed and no administrator membership
was granted. See [Microsoft DisableMSI documentation](https://learn.microsoft.com/en-us/windows/win32/msi/disablemsi).
Managed deployment under this policy, and current standard-user qualification
on Windows client editions, remain open.

For f84ffb0, push workflow 40003837369 completed its company-component checks.
Its original Windows artifact (ID 12524175355) matched GitHub's SHA-256 digest
2f6909f404f82823941454c9bfe4adcea7b4ff6732fddec1b8805b6f63b70efb.
All three binary hashes matched SOURCE.json; the corresponding source archive
matched 157 tracked public-source files, with 14 exact blobs and 143 differences
limited to recorded Windows checkout line endings. No archive was normalized.
The PR workflow's separate artifact identified its merge commit 00aefa5a, so it
was retained separately and was not used as the f84ffb0 input. The exact push
binaries were staged and signed with the existing nonexportable disposable lab
publisher; timestamp and publisher checks passed, and eight original signing
records/artifacts were collected and hash-verified. Services were unchanged.
Independent native builds 40003860186 and 40003868465 were still running;
matching release qualification and deployment were not established by these
component/source/signing checks.

The existing company server and worker were subsequently upgraded together to
the lab-signed f84ffb0 components. A fresh management-only encrypted backup was
authenticated independently, including wrong-passphrase/tamper rejection,
SQLite quick_check and exact release/job snapshot checks. Six original backup
records and eight original upgrade records were collected and hash-verified.
The upgrade preserved customer identity/token, consent, sequence, pending
installation evidence, Running watcher and service process, profile revision 18,
paused rollout, HTTPS binding bytes, transport keys, service environments and
release approvals. The live bundle recipe matched the pinned source calculation.
Old binaries were retained and checked; neither rollback nor restore was run.
This backup does not include the separate TLS/transport private material.

The new worker then generated a protected technician bundle for the already
approved d8fad20 sequence-13 release (job 805e52fe-b286-40d1-b90e-5c7535858534).
Its seven installer/license texts matched f84ffb0 exactly after the documented
line-ending conversion; the signed MSI, agent and release envelope were unchanged
from the earlier pinned release. Public bootstrap/trust remained unchanged,
profile/release signatures verified, and anonymous download returned 401 while
authenticated download returned 200. No new release approval or rollout occurred.

That exact generated bundle completed installation in the company's owned
Administrator profile while the same ProductCode was installed in a different
Windows account. The real catalog moved from state 2 to state 5 for the new
account; the unmodified generated helper/setup, payload publishers, installed
release, cached MSI and agent verification all passed. Sequence 13 and profile
18 were recorded. The account's earlier uninstall cancellation and rollback
marker were resolved by normal complete setup, without manual deletion. The
other technician account and existing customer were unchanged. Nine original
records/files were collected and independently verified, including cached signed
state without device credentials. This verifies the actual per-user correction
and setup cancellation recovery in an administrative profile, using f84ffb0
packaging with d8fad20 endpoint payloads. It is not standard-user, first-install
on a new disk, f84ffb0 native-payload, login or remote-session qualification.

The same unmodified generated setup then passed explicit `-Repair` in that
second administrative profile after its previously hash-verified technician
executable was moved into protected test evidence. Repair restored the exact
signed executable; the retained original hash, agent signature, MSI registration
and final installed-identity verification passed. Managed state and installed
release metadata remained byte-identical, with no setup/cancellation marker left.
The other technician profile and customer process, pending marker and watcher
were preserved. Four original files, including the retained executable, were
collected and independently hash-checked. This remains f84ffb0 packaging with
d8fad20 payloads and does not establish automatic-update recovery.

Two subsequent real uninstall/explicit-repair reinstall cycles passed in that
same technician profile. Both MSI removals returned 0, removed that account's
payload, retained its cancellation marker and company state, and cleared obsolete
uninstall rollback records. Because the product remained installed under the
other account, the current account's catalog state became 2 rather than unknown.
Each unmodified generated `-Repair` then reinstalled the exact signed payload,
returned catalog state 5, cleared cancellation/setup markers, preserved managed
state and release bytes, and passed agent verification. The other technician
account and existing customer were unchanged. Eleven original records/logs were
collected and hash-verified; independent post-test catalog, state hashes and
marker checks also passed. This is administrative f84ffb0 packaging/d8fad20
payload lifecycle evidence, not automatic-update or standard-user acceptance.

The exact f84ffb0 push workflow 40003837369 also completed the actual Linux
deployment job 120586573130. Its original log was retained with SHA-256
c733aa743f6676cb514e22d2a6217b87234497cf51723b22b313e22d04ceef4f.
Fresh non-root/read-only Docker startup, private storage, company setup,
administrator MFA/replay denial, enrollment approval and restart persistence
passed. Both pinned upstream and maintained transport stack rehearsals passed
HTTPS trust/untrusted rejection, actual signed profiles, MFA/enrollment/logout,
complete encrypted archive recovery and replacement startup with original trust.
The three integration scripts matched their previously inspected d8fad20 source
exactly. These are actual isolated Linux deployment tests; they do not establish
native desktop sessions, UDP traversal, public ACME/external reachability or
Windows TLS/transport restore. A fresh Windows 11 qualification environment also
awaits additional lab storage while the older moved endpoint disk remains frozen.

Both f84ffb0 native product builds, 40003860186 and 40003868465, completed
successfully. Main CI, component, transport and portable wait checks also passed.
The separate full Flutter workflow 40003844885 failed: its Windows/Linux x64
non-company builds compiled TX_COMPANY's lazy_static expansion while CompanyTask
was excluded, reporting E0573/E0574/E0412. The feature guard was moved around
the entire company-only macro invocation; validation of that correction is
pending a new run. The legacy 32-bit job separately selected a 32-bit compiler
host that current rustup refused on its runner. Its pinned nightly now selects
the runner host while keeping the i686 compilation target; this also needs CI
validation and does not add 32-bit product support. Original failure logs are
retained. Independent f84ffb0 artifact comparison and local download verification
remain pending; these failures are not hidden by the dedicated product builds.

The original f84ffb0 comparison subsequently failed in workflow 40016176888:
four shipping outputs matched (server setup, agent, management and worker), while
both client MSIs and both portable EXE outputs differed across
the builds. The unmodified failed report is retained as
[native-f84ffb0-rebuild-comparison.json](../deployment/windows/native-f84ffb0-rebuild-comparison.json).
This is a failed reproducibility gate, not a passing rebuild with metadata ignored.
Bounded ZIP manifest diagnostics isolated 12 differing packaged files: the native
launcher and C++ Flutter plugins. A separate pinned diagnostic workflow
40020904002 verified both original archive hashes, extracted the original
payloads without execution and retained their byte/section/compiler/debug
observations. All copied diagnostic payload hashes were checked independently.
Compiler-generated lambda/anonymous-namespace identifiers differed; compiler
Rich records and linker versions matched in the observed binaries. A controlled
compile-only MSVC probe is pending; no normalization or speculative build-flag
fix has been accepted as a successful reproduction.

For the separate non-company build failure, the later 7928f87 workflow
40017774069 passed its actual Windows x64 job 120636061516 and Linux x64 job
120636061669. This verifies compilation after moving the feature guard around
the company-only macro. The full workflow is still running; its legacy 32-bit
job now passes compiler installation but fails at a later build step. Deferred
platform compilation is not advertised product support, and the workflow is not
reported green. No newer native product rebuild has yet qualified the C++ bytes.


The full first f84ffb0 native archive subsequently downloaded and passed its
provider digest, ZIP CRC, corresponding source and shipping checksum checks
(artifact 12526383159, SHA-256
9e450f89d14d2bbec9ea9505f44b57f42abc7b703c4e1a0eef71bf419b5b03cc).
Downloaded executables were not run; the cross-build gate remains failed.

Compiler probe 40027967521 at public tooling source 0fd6519 completed after its
process-tree wait was corrected. Its original report is retained as
[cpp-path-probe-0fd6519-observations.json](../deployment/windows/cpp-path-probe-0fd6519-observations.json).
The provider archive digest verified as
 dfdb62605446753e04bf560bff0a2e819bb02dc93f226989a72c8faee05fcccf.
All four variants produced identical original DLL bytes when only source mtime
changed at the same path, but differing lambda identifiers and DLL bytes when
build paths changed. Compiler /Brepro, /experimental:deterministic and the
initial /d1trimfile form did not remove path sensitivity. This compile-only
experiment is not a product rebuild. No generated DLL was run or normalized.
A follow-up tests relative source input and path mapping before selecting a
product change. The prior timed-out probe's original partials remain retained.


Follow-up compiler probe 40028879677 (d93b067) also passed its harness. Original
[observations](../deployment/windows/cpp-path-probe-d93b067-observations.json)
show that relative input and /pathmap did not make different-directory DLLs
identical. No product flags were changed based on these experiments.

Inspection of both full native job logs then established different hosted images
(20261021.343.1 versus 20261012.333.1) and Visual Studio servicing versions
(17.14.37806.7 versus 17.14.37710.0). Both logs name the same MSVC 14.44.35207
installation directory. The original log digests and selected observations are
retained in [native-f84ffb0-toolchain-observations.json](../deployment/windows/native-f84ffb0-toolchain-observations.json).
Matching PE Rich records do not establish identical compiler binaries, libraries
or headers. Toolchain drift is now a concrete difference to investigate; it is
not yet established as the cause. A future passing reproduction must compare
original shipping bytes with pinned toolchain identity as well as public source.


Toolchain capture probe 40030510222 passed and retained 618 compiler, header and
library input hashes under Visual Studio 17.14.37806.7, default MSVC directory
14.44.35207. Its provider ZIP digest and CRC passed independent collection.
The independent manifest identity calculation exposed culture-sensitive record
ordering in the capture script. The script now explicitly sorts paths ordinally;
that corrected identity format still requires an actual Windows run. The native
product workflow now retains this manifest in both client and final artifacts.
This adds provenance evidence; it does not establish a reproducible product,
identify the historical mismatch's cause or pin Windows SDK inputs.


Corrected ordinal capture then passed actual Windows probe 40031644530 at
bc6eed56f0e3e6ac11d361fba50bd0e4415a080c. The retained provider archive digest
and CRC verified; an independent Python calculation of all 618 original input
records agreed with the captured identity. The actual comparison verifier also
accepted those original manifest bytes, including their UTF-8 BOM. Its bounded
[validation record](../deployment/windows/cpp-toolchain-ordinal-validation.json)
records the original manifest and archive digests. Eleven focused comparison
trust-boundary tests passed, including missing/changed input identity and unsafe
or duplicate records. No generated DLL was executed and no shipping bytes were
normalized. Independent product builds 40031742464 and 40031748532 at da0c621
are still running; their provenance and shipping equality remain unverified.


Independent native product builds 40031742464 and 40031748532 at exact public
source da0c621845057bed321a98a1886ef15891e431a9 both completed successfully.
Frozen comparator 4102bb36b02923bd12c297abf7d91c47bf638364, workflow 40040238668,
verified both original provider archive digests, public source/bridge provenance,
shipping checksums and required compiler manifests. All eight original unsigned
shipping outputs matched byte-for-byte. Both compiler records independently
validated the same 618-file identity
cc28be208566e3a22107767ee1c5a7dbd926154258b518b9abf7f15810a81f47.
The original [comparison report](../deployment/windows/native-da0c621-rebuild-comparison.json)
was collected with provider digest and ZIP CRC verification, without executing
payloads or normalizing any shipping bytes. The older f84 failed report remains
preserved; this new passing pair does not retroactively explain that failure.
This establishes bounded unsigned reproducibility for da0c621 and the recorded
MSVC inputs. SDK/actual CMake command provenance, lab signing, clean-machine
runtime qualification, public sessions and production signing remain separate.
The first full archive is downloading locally for subsequent installer testing;
its local completion has not yet been verified.


The full first da0c621 native archive completed local download and passed its
provider digest, ZIP CRC, exact source and shipping checksum checks. The packed
customer payload was extracted without execution and all 95 files matched the
native MSI manifest. Both company-lab-signed MSIs were built using 213 pinned
inputs; originals and timestamp/publisher receipts were independently collected
and checked, including the 96-entry signed customer manifest. The existing
nonexportable lab key stayed inside the VM. Production signing is not approved.

The existing company worker generated new sequence-14 customer and sequence-15
technician bundles after unapproved build attempts were rejected. Customer
download was public (200); technician anonymous access returned 401 and the
authenticated download returned 200. Seven bundled texts matched exact public
source da0c621, bootstrap contained only six public fields, both signatures
validated against preexisting pins, and MSI/agent hashes matched. Profile 18 and
paused rollout remained unchanged, as did the existing customer's identity,
consent, sequence and service process. Exact job/artifact identities and bounded
results are retained in
[da0c621-lab-distribution-evidence.json](../deployment/windows/da0c621-lab-distribution-evidence.json).

The complete generated technician bundle then passed explicit upgrade from
sequence 13 to 15 in the existing administrative test profile. Installed signed
EXE/agent, sequence-15 cached MSI, MSI registration, original trust/configuration,
failed-setup archive and before/after native verification all passed. Thirteen
original runtime evidence files were independently collected and hash-checked.
The first collector mistakenly sampled the old sequence-13 MSI under a
sequence-15 name; that failed collection was preserved and the corrected path
was collected separately without rerunning installation. This does not establish
standard-user installation, automatic updating, technician login or sessions.

On the separate Server2016 endpoint (14393.693 x64), the current signed customer
MSI passed administrative reinstall after retained uninstall cancellation,
damaged-license repair and uninstall, all returning 0. All 96 payload hashes and
installed publishers were checked; 13 original evidence files were collected.
This reused a previously tested disk and is not fresh-disk enrollment, GUI,
updates or complete OS qualification. Cross-VM HTTPS and session tests still
require the pending guarded private-listener approval; Windows11 storage awaits
the user's additional drive. No acceptance scope has been broadened from these
bounded tests.


Current lab-signed da0c621 customer MSI also passed administrative install,
damaged-license repair and uninstall on the existing Windows10 Pro x64 disk
(19045.3803), each returning 0. A real intervening reboot was independently
established; all 96 payload hashes, installed publishers and automatic service
startup under LocalSystem passed. Original test, MSI and reboot records were
collected and hash-checked. The first post-reboot observation incorrectly mixed
UTC and local DateTime values; its failure was retained, and the test driver was
corrected to compare UTC instants and enumerate decoded PowerShell5 arrays.
The correction did not rerun installation or the reboot. See
[da0c621-win10-bounded-evidence.json](../deployment/windows/da0c621-win10-bounded-evidence.json).

Later in that lab run, the Windows10 console went black and SSH/WinRM stopped
responding while QEMU remained running. ACPI shutdown did not stop it. A recorded
forced VM stop was followed by a qcow2 container check with zero errors and a
successful cold-start recovery; event evidence includes a disk retry (153) near
the stall, without an established cause. A subsequent normal shutdown completed.
These bounded MSI checks are not complete Windows10 qualification, fresh-disk
acceptance, enrollment, sessions or approved automatic-update evidence.

A legacy private lab helper's password was inadvertently displayed during the
run. Its embedded literal was removed from the helper; the original was retained
privately. The first reset attempt stalled before changing the password, as
confirmed by the account's original last-set timestamp after recovery. Automatic
approval review then rejected both a combined reset/verification script and a
simpler reset, and also rejected temporary-file cleanup, with "blocked by policy".
Explicit approval is pending. The replacement was not displayed; its one-time
transfer remains in a verified Administrator/SYSTEM-only directory inside the
affected VM, which is now offline. This incident remains unresolved pending reset
and cleanup and is not treated as a passing security check.

The original Server2016 endpoint was restored on the same disk, MAC and memory,
without setup media and with its prior loopback ports preserved. Its latest MSI
task still reports success and the zero-byte cancellation marker remains. The
company server/worker/watcher are running, legacy customer PID1924 is unchanged,
profile18 remains paused, and the original Caddy binding/pending-install hashes
are unchanged. The company VM and host RustDesk services were not stopped for
these tests. Cross-VM HTTPS approval and additional Windows11 storage remain open.


On a newly created endpoint-only Server2022 Standard Evaluation Desktop Experience
VM (20348.587 x64), the current lab-signed da0c621 customer MSI passed
administrative install, damaged-license repair, a real reboot with automatic
LocalSystem service startup, all 96 installed payload hashes and publishers,
and uninstall. Install, repair and uninstall each returned 0. A pre-install
record established both product registrations absent and no prior endpoint data.
The technician MSI then passed administrative per-user install,
missing-executable repair and uninstall, each returning 0, in its first
application context on that same VM after customer testing. Eighteen customer
and thirteen technician original evidence files were independently collected
and hash-checked. See
[the fresh Server2022 MSI evidence](../deployment/windows/da0c621-server22-fresh-msi-evidence.json).

Initial lab setup had reached the desktop but SSH was stopped: its copied host
key inherited Authenticated Users read access, and the exact sshd diagnostic
rejected that key. The two lab access files were restricted to Administrator and
SYSTEM with explicit protected ACLs, preserving their bytes and the pinned host
identity. The corrected local seed generator checks those ACLs. A temporary
loopback management forward used for diagnosis was removed after pinned SSH
worked, and both setup CDs were detached before Swan MSI execution. An optional
TrustedPublisher store import returned access denied before any MSI ran; that
attempt was retained, and subsequent tests used the already installed lab root,
valid signatures and explicit publisher verification without retrying that store.

These are raw administrative MSI lifecycle checks with lab signing. They do not
establish standard-user technician installation, generated company enrollment,
GUI login, sessions, public transport, software-version upgrades or automatic
updates. The approved company test metadata remains Server2016-only. Production
signing, the remaining OS matrix and full product acceptance remain incomplete.


A subsequent non-administrator technician test on that Server2022 VM failed
installation with MSI1601. Original evidence verifies the user's non-admin token
and separate loaded profile. The Windows Installer log reports failure to
connect to its service with 0x80070005, before the technician executable existed.
This was a password-backed scheduled batch task, so it does not establish the
result of an interactive desktop installation. The connection denial's root
cause remains unproven; standard-user acceptance remains failed/incomplete.
[The original failed test evidence](../deployment/windows/da0c621-server22-standard-batch-failure.json)
retains 16 independently collected and hash-checked records.

Earlier lab launch attempts were also preserved: password-free task registration
was denied, secondary-logon process initialization failed before the harness,
and scheduled startup initially lacked the batch-logon right (0x80070569).
After confirming no explicit batch deny policy, only the owned test account was
temporarily granted SeBatchLogonRight. That allowed the non-admin test to execute.
The temporary right was then removed and the prior account rights verified;
both test tasks are disabled with no running instance. Windows Installer policy
and administrator group membership were not relaxed. Interactive installation
and the failed standard-user requirement need further verification and any
necessary product fix; the earlier administrative passes remain bounded.

An independently configured Linux company lab built the pinned public source
`761b5c5` using two Cargo jobs and started the published management, Caddy and
transport components with loopback-only test ports. HTTPS certificate and
hostname validation, signed custom branding, fresh inventory, administrator
TOTP login, invalid-code/replay rejection, protected worker registration and
logout revocation passed. [The bounded Linux evidence](../deployment/linux/fresh-company-lab-evidence.json)
does not establish worker-generated packages, native sessions, public ACME,
Internet reachability or Windows server setup/restore. New private lab inputs
remain outside Git; the existing company's disk and keys were preserved.

The Windows10 standard-user console login was verified, but its password-free
interactive technician test task reached its ten-minute limit before the
first driver observation or MSI execution. The task is disabled and its
[failed bounded attempt](../deployment/windows/win10-standard-desktop-bounded-failure.json)
is retained. No standard-user installer pass or root cause is claimed.

The independent Linux company subsequently registered its own Windows2019
installer worker, imported and approved new test-only release metadata, and
completed one customer and one technician bundle. Both downloaded over validated
HTTPS with matching server artifact hashes. Anonymous customer download returned
200; anonymous technician download returned 401 and authenticated download 200.
Independent ZIP checks verified the public-only bootstrap schema, company and
release signatures against their existing pins, tamper rejection, all seven
bundled source texts, and signed MSI/agent hashes.
[Worker and bundle evidence](../deployment/linux/company-worker-bundle-evidence.json)
remains limited to test signing and Server2019 release metadata. Rollout remains
paused. Generated customer installation/enrollment and native sessions require
separate evidence and are not implied by bundle generation.

The generated Linux-company customer bundle was then downloaded anonymously by
the Server2019 endpoint over validated HTTPS and installed using its original
company setup script with explicit lab hostname confirmation. It automatically
pinned company configuration and enrolled the native device without address
editing or Tailscale. Installed identities and release sequence were recorded,
the customer service ran, the existing installer worker was preserved, and no
installation recovery marker remained. Unattended consent stayed false. The
server inventory independently matched the enrolled device, retained pending
approval, and rejected an administrator's pre-approval session grant with 403.
[Generated customer enrollment evidence](../deployment/linux/generated-customer-enrollment-evidence.json)
does not establish a native authorized session, UAC, Internet transport,
automatic updates or the full Windows matrix.

The generated technician bundle also completed per-user company setup in the
owned Server2019 administrator profile. It pinned the independent company's
identity, verified the installed publisher, recorded technician release sequence
2, and fetched signed branding revision 2 published after the bundle was built.
Its state held no customer device token and no pending installation marker.
[Technician company-setup evidence](../deployment/linux/generated-technician-setup-evidence.json)
is explicitly administrative: standard-user installation, technician MFA login,
permissions and native sessions remain separate requirements.

The installed customer's configuration watcher subsequently fetched the signed
branding revision 2 automatically, without an explicit sync command, installer
rebuild or reinstall. Its cached envelope independently verified against the
original company signing pin. Device/company identity, unattended consent and
software release sequence were preserved; the customer service and watcher
remained running. [Automatic branding synchronization evidence](../deployment/linux/automatic-branding-sync-evidence.json)
qualifies the installed configuration path only. Native visible branding,
offline display, restart/upgrade persistence and trusted key rotation still need
their separate acceptance evidence.

Management authorization was then checked against that genuinely enrolled device
after explicit lab approval. Two technician accounts authenticated using TOTP.
The assigned technician could list the device and receive an independently
verified signed attended grant bound to company, device, technician, proof key
and short expiry. The unassigned technician saw no devices and grant requests
returned 403. Technician attempts to create an administrator-only worker returned
401. Unattended grants without consent returned 403. Removing group access hid
the inventory and denied new grants; the lab assignment was subsequently restored.
[Technician authorization evidence](../deployment/linux/technician-authorization-evidence.json)
qualifies the management API only. Native receiver enforcement, customer consent,
actual desktop sessions, lease expiry and in-session revocation remain open.

The generated technician edition was launched through its original verified
launcher on the owned administrator desktop. Its actual Flutter interface showed
updated company branding and HTTPS origin, accepted username/password/TOTP,
listed the authorized native-enrolled device and group, and displayed session
history. An independent server audit matched this later GUI login, distinct from
the earlier API tests. GUI sign-out cleared inventory and secret input fields.
[Native technician interface evidence](../deployment/linux/native-technician-interface-evidence.json)
does not qualify a remote desktop session: this was the same Windows2019 machine
as the customer endpoint, and standard-user installation remains open.

During a guarded five-minute outage of only the isolated management container,
the native customer UI opened from its installed shortcut and displayed cached
updated branding. Its window initially became unresponsive and later recovered
while management was still offline. Customer Stop support eventually changed to
Resume support; an independent query confirmed the customer service was absent
and the installer worker remained running. Resume support restored the customer
service without changing enrollment, consent, software sequence, signed executable
or branding revision. The management restart safeguard restored healthy HTTPS
and the original company identity/revision.
[Offline customer/control evidence](../deployment/linux/customer-offline-controls-evidence.json)
records significant delays. Their cause and acceptable responsiveness remain
unresolved. No active remote session was present, so immediate disconnection,
standard-user control and offline authorization denial are not qualified.

The independent Linux company's coordinated encrypted deployment export included
management data, transport keys/database, full TLS storage, the actual proxy
recipe/deployment settings and both generated installer bundles. Wrong-password
restore failed before creating files. Authenticated restore retained exact
transport key bytes, the complete TLS file tree, settings and installer hashes.
Separate recovered service volumes then started management, Caddy and transport.
HTTPS validated against the original lab root/hostname; the signed profile
verified against its original pin. Administrator MFA, device approval/consent,
completed jobs, both existing download paths/hashes and logout revocation passed.
The existing Windows customer's credential also authenticated to the recovered
API over the same trust without changing its bootstrap or credential. Recovery
containers were stopped afterward, original endpoints remained unchanged, and
the verified encrypted archive/passphrase are stored separately outside Git.
[Full Linux recovery evidence](../deployment/linux/company-full-backup-restore-evidence.json)
retains the initial internal-network HTTPS timeout and corrected loopback bridge
check. It does not qualify native desktop transport, public ACME/Internet access,
or full Windows company-server recovery.

The unreleased 1.5.2 candidate closes legacy updater entry points for managed
editions. Background start/stop helpers do not initialize the upstream worker;
manual requests and direct legacy checks reject before MSI inspection or upstream
version requests. The company agent remains responsible for approved updates.
A native regression checks all legacy entry points and verifies that no upstream
worker starts. The Windows build workflow must compile and run that regression
before producing candidate artifacts. Version metadata/lockfiles agree locally;
the first native Windows workflow (40398269765) has now compiled and passed
its regression and packaging steps. Artifact integrity/source verification, an
independent rebuild (40410425646), and real 1.5.1-to-1.5.2 rollout/recovery remain
pending.
This candidate is not a production release or an accepted automatic-update test.

Signed company policy revision 3 enabled unattended support as a company option,
while the approved customer retained false device consent. An assigned technician's
unattended grant remained denied with 403. The customer UI then displayed the
company/domain and sign-in-screen implications in an explicit confirmation.
Accepting it persisted local consent revision 1, updated the server and enabled
an independently verified signed unattended grant bound to company, technician,
device, proof and expiry. Native GUI revocation increased the local consent
revision, cleared server consent and denied subsequent unattended grants with
403 while leaving the customer service running. The prior unused grant was
closed and its lease cleared, but it had expired before the final inspection.
[Native unattended consent evidence](../deployment/linux/native-unattended-consent-evidence.json)
does not qualify an active native session, unexpired-lease termination,
standard-user consent, reboot/logon-screen access or public connectivity.

The independent Linux company completed a profile-signing key rotation against
both installed 1.5.1 editions on the Server 2019 lab endpoint. A fresh encrypted
full deployment backup preceded preparation. Both apps cached old-key-signed
transition revision 4 before activation, then adopted revision 5 and its new pin
through their normal background refresh without manual agent sync or reinstall.
Customer unattended consent remained revoked at consent revision 2; software
release sequences stayed unchanged. A separate shipping-agent fixture that
missed the transition rejected the replacement signature and retained its exact
cached state. Old customer and technician download links returned 404. The worker
pin was explicitly updated from the accepted transition, and replacement test
bundles completed. Both replacement downloads verified against the rotated
profile key, unchanged release key, package hashes and exact bundled source;
customer downloads remained public and technician downloads required authentication.
[Installed profile-key rotation evidence](../deployment/linux/installed-profile-key-rotation-evidence.json)
does not qualify active native-session termination, interrupted activation,
rotated server restore, the wider Windows matrix or public connectivity.

The installed-release configuration agent was also executed against isolated,
credential-free cached-state copies on Server 2019. Its verify-bootstrap command
accepted the unchanged valid control, rejected a modified signed payload,
rejected a wrong company, and rejected a revision below the stored minimum.
Using a fresh disposable fixture issuer, it accepted a correctly signed fresh
control and rejected a correctly signed expired policy. Every fixture remained
byte-identical after validation, and no installed state or system clock changed.
[Shipping-agent cached-policy evidence](../deployment/linux/shipping-agent-cached-policy-negative-evidence.json)
is local validation evidence; incoming hostile HTTPS profiles, active native
sessions and other Windows/account contexts remain separately unqualified.

A separate encrypted backup taken after rotation was restored into fresh storage.
The recovered management service used the rotated database key while its legacy
seed remained unchanged, and the recovered HTTPS proxy validated with the
original lab CA and hostname. Revision 5, administrator MFA, approved device
identity, revoked unattended consent, both replacement bundle hashes and the
withdrawn old links survived. The existing Windows customer credential also
authenticated to this recovery endpoint without changing its bootstrap or token;
its credential stayed on the guest. Recovery services were stopped afterward.
This extends the installed-key rotation evidence above; no recovered transport
or native session was started, and interrupted activation remains unqualified.

The first 1.5.2 native artifact from d74a631 was downloaded and its complete
archive digest and SHA256SUMS entries verified. All 1,186 tracked source files
matched their public Git objects (allowing only declared text checkout newline
conversion), and six generated bridge files matched that exact workflow's
bridge archive. Native security regression logs and packaging steps passed.
The independent attempt 40410425646 stopped before native compilation: the old
Flutter installation action selected latest stable despite its requested
version, and the engine selector rejected the missing pinned SDK. Its failure
is retained; byte reproduction has not passed. The Windows workflow now uses a
fixed official archive/hash and verifies framework/engine revisions, with local
negative tests proving wrong versions and altered archives cannot extract or
change the exported environment. The first fixed-SDK workflow 40415706839 passed
fresh installation, native security regressions and packaging. Its archive and
shipping hashes were verified; tracked source and six generated bridge files
matched pinned public source and the exact workflow bridge archive. Independent
reproduction and runtime upgrades remain separate gates.
[Fixed-SDK build evidence](../deployment/windows/a1d67be-native-build-evidence.json)
records the exact artifact digest and verified SDK identity.
[First 1.5.2 artifact evidence](../deployment/windows/d74a631-native-build-evidence.json)
is not a production release or a completed automatic-update qualification.

On the newly installed Server 2025 Standard Evaluation Desktop Experience VM,
build 26100.32230 x64, the da0c621 lab-signed technician MSI passed administrative
installation, missing-executable repair and uninstall, each returning 0. This was
the first Swan application installation context on that VM, before customer
edition testing. A one-time administrator S4U task ran the pinned test scripts;
no account password was supplied to task registration. All 13 original source,
result and MSI-log records were collected and hash-verified.
[Server 2025 technician MSI evidence](../deployment/windows/da0c621-server25-technician-msi-evidence.json)
does not qualify standard-user execution, technician login, native sessions,
software-version upgrades or automatic updates. Customer reboot verification
and the wider release matrix remain separate.

The same new Server 2025 VM subsequently passed the da0c621 lab-signed customer
MSI installation, damaged-license repair and uninstall, each returning 0. A real
Windows restart preserved all 96 packaged file hashes, the configuration agent
and valid installed publishers. The customer service started automatically as
LocalSystem; the verifier did not start it manually. The one-time startup task
then uninstalled the package successfully and disabled itself against repeat
execution. All 18 original customer source/result/log records were collected and
hash-verified, alongside the 13 technician records above.
[Server 2025 customer MSI evidence](../deployment/windows/da0c621-server25-customer-msi-evidence.json)
qualifies this administrative lifecycle only. Managed enrollment, standard-user
installation, native remote access including the logon screen, software-version
upgrades and approved automatic rollout remain separate acceptance requirements.

The isolated Linux company subsequently used a distinct protected Windows worker
on Server 2025 to generate an authenticated technician bundle. Its protected
local artifact matched the server result; anonymous technician download was
rejected. The actual shipped company setup installed the lab-signed 1.5.1
technician edition with current signed profile revision 5, release sequence 3
and no device token. Native username/password/TOTP login showed company
branding, the approved device/group and session history, independently matched
by the server login audit.
[Server 2025 generated company technician evidence](../deployment/windows/da0c621-server25-company-technician-evidence.json)
qualifies administrative per-user setup and native login only. The customer
was offline during login; separate-machine remote access, receiver permission
enforcement, standard-user setup and automatic rollout remain unqualified.

Subsequent separate-machine testing connected the Server 2025 technician to the
Server 2019 customer through this independent Linux company. The native customer
received and accepted an attended approval prompt; the technician saw its actual
desktop. Remote mouse input minimized the customer window, verified separately
on the receiving console. The receiver claimed the company grant and renewed
its lease beyond the initial expiry. Customer-local Stop support disconnected
the technician, removed the support service through the existing native stop
path and prevented reconnect; the customer showed Resume support. Server-side
closure of the retained lease record is not claimed.
[Two-machine session evidence](../deployment/windows/da0c621-linux-company-two-machine-session-evidence.json)
was subsequently correlated with the receiver and company relay logs using the
same connection UUID: this successful session used the company relay. An earlier
relay handshake failed to receive a public key before the second paired request
succeeded. This does not qualify direct NAT traversal, separately forced-relay
coverage, public Internet operation, hostile native connections, expired-lease termination, keyboard,
clipboard, transfer, UAC, logon-screen or multi-monitor behavior.

This run also exposed a failed automatic customer service start after the two
VMs booted together on HDD storage: Windows recorded events 7000/7009 and a
30-second service connection timeout. A single manual recovery succeeded before
the session above. The original failure is retained; its cause is unproven and
reliable cold-boot startup remains a failing acceptance gate. Earlier Server
2025 administrative reboot evidence does not resolve this Server 2019 failure.

Investigation confirmed that this headless service had loaded the Flutter engine
and desktop plugins. The original da0c621 launcher normally imports the engine
and eleven plugin DLLs, while its Rust core has no Flutter DLL import. The Windows
build now delay-loads the engine and generated shared plugin targets, using
[MSVC's supported delay-load mechanism](https://learn.microsoft.com/en-us/cpp/build/reference/linker-support-for-delay-loaded-dlls).
A new CI verifier rejects an eager bundled DLL import or a missing delayed
engine, and rejects malformed import descriptors; its local regression tests
pass. Native run 40435586938 also passed the import gate and actual service module
inspection described below. Repeated cold-boot tests are still required; this
change is not yet proof that the startup timeout is fixed.

A later technician relaunch remained inside `swan-agent verify-installed` while
its publisher-verification child was pending. The agent now bounds that child to
120 seconds, terminates and reaps it on timeout, and returns an error without
authorizing launch or installation. It still requires the original signature,
publisher, certificate and hash checks. New process tests cover actual successful
and rejected child exits plus timeout cleanup. Windows and Linux component CI
passed for 7407fa1, including the actual Windows child-process tests;
[CI evidence](../deployment/windows/7407fa1-publisher-timeout-ci-evidence.json)
does not yet qualify installed-app timeout behavior. The prior unbounded lab process is retained
for diagnosis rather than counted as a successful relaunch.

For source a1d67be, independent native runs 40415706839 and 40423647265 now pass
the original-byte comparison: all eight shipping EXE/MSI outputs are identical,
all 618 recorded MSVC inputs match, and each archive's 1,192 tracked source files
match pinned public Git objects. Six generated bridge files in each archive
also match that run's retained bridge artifact. Shipping binaries were not
normalized or executed for this check.
[Independent comparison](../deployment/windows/a1d67be-native-independent-comparison.json)
does not qualify runtime upgrades, the newer startup changes, signing or release
publication. The first Docker-context test for 06de236 failed because the new
verifier source was missing from the source allowlist; both context allowlists
now include that exact file. Run 40432863268 subsequently passed the Linux
deployment check: 46 required public files were exported and all configuration,
key and cache canaries excluded. The original failed run remains retained.

The outgoing-only technician startup path also previously allowed automatic
quick-support detection under an elevated account. That could enter the incoming
portable helper's display discovery and shared-memory allocation. Outgoing-only
editions now disable quick-support mode, reject direct helper CLI invocation and
reject both direct and logon helper requests before runtime initialization. The
native regression invokes the real helper entry point with outgoing-only policy
and checks that startup state and shared memory remain unchanged. That exact
test compiled and passed in native run 40435586938 for source 9ba592c. Installed
technician startup and reconnection remain unqualified.

The native workflow now includes an unsigned customer MSI smoke test on a
disposable Windows runner: install, require automatic LocalSystem service startup
without a manual start, inspect the service process for absence of Flutter engine
and plugin DLLs, then uninstall. It refuses existing customer state and retains
logs/results even when cleanup fails. Run 40435586938 passed this actual test:
the MSI install completed in 4.65 seconds, its automatic service loaded the Rust
core without the Flutter engine or plugin DLLs, and uninstall was verified.
The retained original result and logs were independently hash-verified against
artifact 12706347182; [evidence](../deployment/windows/9ba592c-native-startup-ci-evidence.json)
records the exact source, package and artifact identities. This administrative,
unsigned CI test does not qualify cold reboot, enrolled sessions, publisher
acceptance or the complete Windows compatibility matrix.
Windows PowerShell 5 also rejected the fixture on this host before resolving
its nonexistent test package, confirming the disposable-runner guard.
