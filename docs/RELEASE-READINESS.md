# Release readiness — 5 November 2026

Current app candidate: **1.5.21+89**, public source `bd331ca`. Both technician formats and the customer MSI have package verification. Customer EXE30 was withdrawn after a lab MSI/EXE manifest pairing error; [corrected EXE31 shipping recovery, uninstall/reinstall and compact customer About now pass](evidence/1521-customer-exe31-recovery-uninstall-reinstall-and-about-20261104.json) on the existing Windows10 development VM. [Independent native comparison](evidence/1521-independent-native-original-output-reproduction-20261104.json) matches all eight original unsigned outputs; [existing standard-user technician checks](evidence/1521-standard-technician-upgrade-about-mfa-20261104.json) pass upgrade, About, genuine MFA/inventory and sign-out. Both endpoint About screens and the customer shipping uninstall fix now have genuine app evidence. Remaining permission, desktop-feature and update checks are open; current live-session results follow. Finish these on the existing development pair before the final advertised-Windows matrix. Reuse one VM per OS; do not create a VM for each candidate. This remains a development build, without production approval.

Next candidate **1.5.22+91**, source `ca16998`, fixes company window and tray
labels while preserving stable service/configuration names and device identity.
It uses trusted signed cached branding for display only; expired policy still
cannot authorize access. The single [native build](https://github.com/rijff24/SwanRemoteSupport/actions/runs/41964194947)
passed. Its256,777,570-byte original archive, manifest,1,406 public source files
and six generated bridges are verified. New compiled Rust branding and Flutter
title/approval tests passed; the Rust branding log was omitted from the original
artifact and is verified from the retained original completed job log instead.
Future CI uploads include it. All95 customer payload files were extracted without
execution and matched the verified manifest. The one [independent rebuild](https://github.com/rijff24/SwanRemoteSupport/actions/runs/41970609821)
passed its build/tests from the exact same source, but the [original-byte comparison](https://github.com/rijff24/SwanRemoteSupport/actions/runs/41976573481)
**failed**. Both source/bridge proofs and recorded MSVC inputs match. The server
installer, management, agent and worker match; the customer/technician EXE and MSI
outputs differ. Payload-level diagnosis is in progress. No candidate22 packages
are signed, approved or installed. Native22+90 was also never approved or installed
and is superseded. Reproduction, runtime branding, genuine byte-changing automatic
updates and final qualification remain open.

[Small real MSVC compiler fixtures](evidence/1522-msvc-lambda-path-controls-20261105.json)
then isolated a source-path effect on lambda RTTI names. Same-path output matched
across two CI jobs, but relocation changed original bytes. Experimental compiler
determinism/path mapping, compiler-level Brepro, path trimming and relative input
names did not restore raw equality in that fixture. These are failed hypotheses,
not an application fix. A subsequent [actual SDK reply-callable fixture](https://github.com/rijff24/SwanRemoteSupport/actions/runs/41983906171)
passed with identical raw executables at relocated source paths after replacing
the anonymous callback with a named callable. Its stub C API checks lifetime,
payload, duplicate replies, unavailable-engine handling and concurrent replies;
it does not qualify real engine sessions. Candidate **1.5.22+92** integrates that
source-only patch with exact upstream and patched-source pins, archived receipts,
and independent comparison checks. Five source-trust and sixteen comparison
tests pass locally. Actual cached SDK application, full native reproduction and
installed runtime checks remain required. No candidate22 signing has started.

The [full +92 primary build and original artifact verification](evidence/1522-named-wrapper-primary-build-20261105.json)
now pass: the actual pinned SDK source patch compiled, all native jobs completed,
and the original256,841,594-byte archive matches its GitHub digest. Verification
covers1,417 public source files, all six original generated bridge inputs,
the patch receipt and compiled branding/approval/transfer logs. CI technician MSI
lifecycle and customer headless-service checks passed. The one [exact-source
independent rebuild](https://github.com/rijff24/SwanRemoteSupport/actions/runs/41996756641)
passed from the pinned source tag after retrying only its failed Windows job:
the initial attempt hit an HTTP504 dependency download before app compilation.
The [original-output comparison](https://github.com/rijff24/SwanRemoteSupport/actions/runs/42009952812)
then **failed**: server setup and all three backend programs match, but the four
endpoint packages differ. Public-source, bridge, MSVC and patched-wrapper input
proofs match. The named-wrapper change alone has not fixed full-app reproduction.
Both original archives were then retained and their customer payloads extracted
without execution. All95 files per archive match their original MSI manifests;
94 raw files now match between builds, including the launcher, plugin DLLs,
engine and Dart payload. The sole mismatch is `librustdesk.dll`:73 bytes differ,
including one code byte,68 bytes in `.rdata` and four header bytes. Its other
six sections match. The code difference is isolated to RVA`0xddde2e`, but its
object/function origin is not identified yet. No byte normalization is applied.
Diagnosis continues; no further full build is started on a guessed fix.
No +92 package is signed, approved or installed. Existing development VMs remain on21.


[Current management evidence](evidence/1522-management-reproduction-upgrade-and-probe-20261105.json)
now records the corrected compiled Windows checker passing all15 public cases,
including valid DER and PEM trust. Two separate Windows component builds also
produced identical raw management, agent and worker binaries, with261 exact
corresponding source files verified and the current checker fixture passing.
The inherited32-bit Windows job fails because its legacy Rust1.75 toolchain
cannot parse the version4 lockfile;32-bit Windows is deferred. This is not an
x64 app pass: the dedicated pinned Rust1.90 native build remains required.
Two cold canonical Linux builds produced
identical raw management executables and source contexts using the pinned
published recipe; full OCI identifiers differ. Real fresh-company setup, MFA,
enrollment, persistence and backup/restore passed. After a fresh encrypted full
backup, the existing company upgraded to the canonical image with real HTTPS/MFA,
preserved company records and unchanged transport/TLS containers. Offline full
restore matched all57 installer files,6 TLS identity files and deployment keys.
The restored existing company then passed MFA/replay rejection, original-key
profile signing and device/user/release/build persistence across restart in a
network-disabled container. That copy is stopped and retained; restored HTTPS
and native sessions remain separate. Updates remain paused. No production
publication is implied.

[Current standard-user live checks](evidence/1521-standard-native-interrupted-transfer-consent-and-stop-20261105.json)
now pass attended desktop access, separate file consent, a real45-second network
interruption during a256MiB transfer, and original-window completion after fresh
approval with exact received hashes. Accepting desktop access while file approval
was pending kept the approval window visible, resolving the previously failing
native17 regression. Customer Stop closed both live server records and access;
native technician history displayed Closed. These results qualify this existing
development pair. They do not close the other permission, desktop-feature,
update, external-connectivity or final Windows-matrix gates. Exact resumed
offsets and exact live30-second timeout timing were not measured.

[Current direct TCP authorization](evidence/1521-native-direct-ip-denial-and-preservation-20261105.json)
also rejects a real legacy-style password login before company authorization.
The test reached the actual installed Windows10 receiver and obtained the
explicit company-denial response. No grant or enrollment was created; identities,
consent and native bytes were preserved, and the temporary listener/forward were
removed. The stock graphical control stayed black and is not counted as passed.
Its earlier genuine ID-based denial has separately verified unchanged receiving
source; external routing and final OS qualification remain open.

The new [company-run connection checker](NETWORK-CHECK.md) has
[scoped Linux development evidence](evidence/company-network-probe-development-20261105.json):
real trusted HTTPS, pinned signed profiles, four TCP ports and a stateless native
UDP rejection response. Wrong profile keys, untrusted certificates and private
paths presented as outside-network qualification are rejected. The initial CI
build-context test caught an omitted source-file allowlist entry; both Docker
filters now include that file, and an actual directory-context scratch export
includes every required public input while excluding all private canaries.
Canonical server builds, compiled Windows execution, and authenticated public-only
bootstrap export now have the scoped evidence above. The live company is on the
reproduced canonical server. New native sessions against that server remain to
be checked; external direct/relay sessions are not qualified by this checker.

The source-specific historical findings below retain their original scope and do not describe the current deployment policy.

[Native1.5.14 real automatic update](evidence/1514-real-automatic-msi-update-and-ui-restart-failure-20261102.json) installed and verified the exact signed MSI while preserving identity, trust and consent, but **failed customer UI recovery**. MSI renamed the mapped old image into rollback storage; the exact kernel-path guard rejected termination, and an advanced PowerShell failure returned an incorrect success status. Updates are paused at profile40. Pre-MSI process shutdown and explicit helper error propagation are draft fixes; production and full update acceptance remain blocked on their verification.

**Not ready for production publication.** This index distinguishes current,
scoped evidence from older checkpoints in [ACCEPTANCE.md](ACCEPTANCE.md).
Passing one operating system or one component does not qualify the full product.

[Windows10 customer1.5.10 to1.5.11 automatic update and normal reboot](evidence/1511-win10-automatic-upgrade-reboot-and-profile-rejection-20261031.json) passed with exact signed installed files, MSI registration, shipping verification, preserved identity/consent/branding and restored paused profile28. No manual update, installer invocation, watcher repair or controlled watcher restart was required. Fourteen isolated cases using the installed1.5.11 agent returned the expected profile signature/time/company/replay results. This does not qualify the remaining native-session, interruption or OS/edition matrix.

[Server2019 technician1.5.10 to1.5.11 automatic update](evidence/1511-server2019-technician-automatic-update-and-recovery-principal-finding-20261031.json) passed exact installed-file/publisher checks, shipping verification, fresh native MFA/inventory and company reporting. Staged rollout included technician cohort27 while the same-VM customer cohort78 remained on sequence13 with no sequence14 staging. Original pause/20% policy was restored at profile30. **Recovery remains unqualified:** Windows returns a short task principal name that the shipping ownership guard rejects; an older recovery task has exit1. [Candidate1.5.12](evidence/1512-recovery-principal-sid-regression-20261031.json) resolves the principal to its exact SID. Local Windows PowerShell and actual Server2019 existing-task inspection plus isolated task registration/removal regressions passed. The built-in Administrator token was unfiltered despite a Limited task run level; this is not standard-user qualification. Interrupted installation recovery remains pending; the compiled standard-user cleanup result below covers only task cleanup.

[The compiled1.5.12 recovery helper](evidence/1512-compiled-recovery-standard-user-cleanup-20261031.json) now passed on a genuine Windows10 standard-user token: the old helper failed against the short account name, while the signed candidate wrote its pinned embedded guard and removed its exact self-owned task. The actual customer identity, credential, trust, consent, sequence and original installed agent were preserved. This qualifies no-pending task cleanup, not interrupted installation recovery.

[Thirteen real shipping-agent package checks](evidence/1511-installed-windows-signed-package-rejection-20261031.json) accepted a valid1.5.11 MSI and rejected signature, replay, expiry, edition, architecture, channel, Windows compatibility, hash, publisher, certificate, MSI-version and invalid-package cases in isolated credential-free fixtures. No installer or online update was invoked.

[The first canonical Linux image](evidence/1512-canonical-linux-first-build-and-fresh-company-20261031.json) built from source942 with the published Dockerfile and pinned official Rust image. Its real fresh-company/MFA/replay/enrollment/persistence/backup/restore harness passed. [A second independent canonical build](evidence/1512-canonical-linux-reproduction-live-upgrade-and-offline-restore-20261031.json) produced the identical management executable, and its real fresh-company harness passed. The existing Linux company subsequently upgraded to the canonical image after a fresh encrypted backup; HTTPS/MFA, company records, profile30, paused policy and unchanged transport/TLS containers passed. Current anonymous/unassigned grant denial, authorized inventory, consent enforcement and public customer/protected technician download routes passed. That backup also restored offline with wrong-password rejection and exact key/settings/TLS/installer-file equality. Full OCI image identifiers differ; live restored-company and external native transport remain separate gates.

[The complete Windows1.5.12 native build and Windows/Linux component checks](evidence/1512-native-and-component-build-success-20261031.json) passed at source942. [Independent Windows build and original-output comparison](evidence/1512-independent-native-original-output-reproduction-20261031.json) passed: all eight unsigned outputs,1325 tracked source files, six generated bridges and618 MSVC input files per build matched. The original agent hash also matches the component agent used for the genuine standard-user recovery regression. Local full primary archive verification subsequently passed, including its complete hash, manifest,1325 tracked source files and six generated bridges; see the installed recovery evidence below. The second canonical Linux build completed from the identical source context with no compiled-layer reuse; its matching management bytes, runtime and live upgrade evidence are above. These build results do not qualify installed upgrades or production publication.

[Real Windows10 customer session coordination and interrupted1.5.12 recovery](evidence/1512-win10-real-session-update-deferral-and-interrupted-recovery-20261031.json) subsequently passed. Both company bundles were generated by the real Windows worker with lab-signed MSIs; anonymous customer downloads succeeded and anonymous technician downloads were rejected. A genuine standard-user customer approved a native1.5.11 session against the canonical Linux company, then stopped it and revoked the technician connection. During that actual desktop session, release15 downloaded but the Windows activity lock prevented installation. After the session ended, a guarded hypervisor power cut interrupted the real MSI client with its durable installing receipt. Normal boot automatically recovered to1.5.12, passed the shipping installed-file/publisher verifier, preserved identity/credentials/branding/consent and removed its recovery task. Unexpected-shutdown events41/6008 corroborated power loss. No manual update, installer, recovery or service-start command was used. The original paused policy was restored and accepted at profile32. This qualifies this Windows10 customer interruption boundary; technician recovery, other advertised systems, external transport and production signing remain open.

[The Windows10 standard-user idle customer1.5.12 to1.5.13 test](evidence/1513-win10-idle-customer-update-stale-gui-finding-20261031.json) verified the automatic MSI installation, exact signed files, service/watcher, preserved enrollment/consent and restored paused profile38, but failed the existing GUI check. The old pre-update frontend processes remained alive and displayed “Company setup is not complete” without the company branding. Closing only those exact old GUI processes and normally reopening the updated application restored branding and approval controls without re-enrollment, reinstalling or stopping the service. That manual diagnostic is not automatic-update acceptance. Customer GUI quiescence/restart needs implementation and a new genuine standard-user test before release.

Candidate1.5.14+82 implements customer UI recovery after a verified update while retaining session exclusion and the signed restart marker. [The real standard-user helper regression](evidence/1514-customer-ui-restart-functional-helper-20261031.json) passed process/owner guards, foreign-task rejection, durable intent across two injected interruption boundaries, actual unelevated relaunch, task self-cleanup, fresh-process idempotence and uninstall task cleanup. Company branding and consent controls rendered after relaunch. [Windows/Linux component checks](evidence/1514-component-ui-helper-and-linux-context-ci-20261031.json) passed at0aac973, including the real-child timeout/reaping regression, large-backup recovery and corrected private-file-excluding Linux context. The installed application in the helper test was1.5.13; the native1.5.14 build and actual13-to14 automatic MSI update are still required.

## Sources and signing

- Candidate `1.5.13+81` addresses a [real stale portable-runtime finding](evidence/1513-portable-stale-runtime-finding-and-cache-isolation-20261031.json). Server2019 technician files upgraded automatically to1.5.12 and passed exact signed installed-file verification, but the shared extracted executable remained1.5.11 and the current installation report did not complete. The new wrapper isolates each embedded payload by its SHA256 and refuses launch after extraction errors. Four Python metadata tests and two standalone Linux Rust tests passed. Windows exclusive-file-lock and Linux cache regressions, the full native build and exact corresponding-source verification subsequently passed. [The actual Server2019 technician1.5.12 to1.5.13 automatic update](evidence/1513-technician-automatic-update-runtime-mfa-and-report-20261031.json) also passed: the sole running native process is1.5.13+81 in its new payload-specific cache and matches the verified original runtime hash; fresh MFA/inventory and the exact company installation report passed. Identity, consent, trust and the excluded customer remained unchanged, and paused profile36 was restored. Two obsolete pre1.5.12 lab recovery tasks were separately removed through exact ownership checks after their old helpers failed; this maintenance is not claimed as upgrade cleanup acceptance. The test used the owned built-in Administrator, so standard-user technician and current-candidate remote desktop/lifecycle qualification remain open. [Independent native reproduction](evidence/1513-independent-native-reproduction-20261031.json) passed: all eight original unsigned shipping outputs are byte-identical, with1334 tracked public files, six generated bridges and618 matching MSVC inputs per build. This supersedes any inference that successful MSI verification alone qualified technician restart.

- Development candidate `1.5.11+79` gives configuration/recovery tasks and
  noninteractive trust/update helpers normal scheduling priority. Restart and
  restore migrate the validated existing configuration task without changing
  its action or account. This source change follows a Windows10 lab repair;
  [Windows/Linux component checks](evidence/1511-component-priority-and-recovery-ci-20261030.json)
  passed, including the real Windows parent/child priority test and large-backup
  recovery. The [full native build and local archive/source verification](evidence/1511-7b60309-native-build-and-source-ci-20261030.json)
  also passed, including customer headless service startup and unsigned technician
  install/repair/uninstall on CI. All1314 tracked source files and six generated
  bridge files matched. [Independent build41093808835 and original-byte comparison](evidence/1511-independent-build-coordinated-lab-upgrade-and-bundles-20261031.json) passed: all eight shipping outputs,1314 tracked source files, six bridge files and618 MSVC input files per build matched. Production signing and installed-endpoint qualification remain open.
  [Owned lab signing](evidence/1511-7b60309-owned-lab-signed-packages-20261030.json)
  subsequently produced verified customer/technician MSIs, their native programs,
  the agent, management and worker using the retained nonexportable lab key.
  These are test packages; no1.5.11 installation or live company upgrade is
  qualified by signing alone. A matching
  [Linux lab management build](evidence/1511-7b60309-linux-lab-management-fresh-company-20261030.json)
  passed fresh-company setup, MFA/replay, enrollment, restart and backup/restore
  checks using a verified cached toolchain and a new empty build path. The [owned company server and private Windows worker upgrade](evidence/1511-independent-build-coordinated-lab-upgrade-and-bundles-20261031.json) subsequently passed with a fresh encrypted backup, real HTTPS/MFA and unchanged company profile26, consent, accounts, permissions, release/job records, private worker credentials and TLS/transport containers. Both release14 company bundles passed exact-source/signature/hash checks and public customer/protected technician downloads. Automatic rollout remains paused; native integration, installed1.5.11 qualification and canonical Docker image reproduction remain open. The exact 1.5.10 receipts
  below remain scoped to their original source and lab settings.
- Working candidate `1.5.10+78` adds an early service-manager connection before
  loading the native core. The corrected complete build run40988690083 succeeded:
  [exact source, input hashes, security regressions and unsigned MSI lifecycle](evidence/1510-b1fbe1e-native-build-and-msi-ci-20261030.json)
  verified. The actual customer service started without desktop modules on CI;
  technician install/repair/uninstall passed. Broader enrolled-machine upgrade
  and reboot reliability qualification remains pending. [Independent Windows reproduction](evidence/1510-b1fbe1e-independent-native-reproduction-20261030.json)
  verified all eight original unsigned shipping files byte-for-byte, with matching
  recorded compiler inputs and no shipping-file normalization. The
  [startup design and boundary tests](../deployment/windows/SERVICE-STARTUP.md)
  passed actual SCM/loader tests on Windows Server 2022 CI and the Server 2019
  lab using simulated core DLLs; see the [boundary receipt](evidence/1510-service-bootstrap-scm-boundary-20261030.json).
  Actual lab-signed 1.5.10 subsequently passed an enrolled Server2019 upgrade and
  three normal reboots with automatic native startup and no timeout events;
  [distribution and reboot evidence](evidence/1510-linux-bundles-server2019-upgrade-three-reboots-20261030.json)
  retains the historical 1.5.9 failure and the remaining qualification limits.
  The first native build failed on a missing `winsvc` dependency feature; the
  feature is now explicit in source `b1fbe1e` used by that corrected run.
- Earlier native customer/technician candidate: `aff8ff61e3fbd6f34c4cf0bb9de6e330ec6e0521`,
  version `1.5.9+77`.
- Management storage/recovery candidate:
  `361d7c362de7f67b7b7456d33bf0191736eec7d5`.
- Migrated Windows installer worker:
  `1c43a382c87b05e44caff1c6c2bef4cf0b0bb6fc`.
- These Windows lab packages use a disposable lab publisher. Production signing
  approval and production-signed release artifacts remain outstanding.
- [Windows/Linux component reproduction](evidence/361d7c3-push-pr-windows-linux-component-reproduction-20261030.json)
  establishes identical management, agent and worker bytes in the two recorded
  component builds. The separate 1.5.10 Windows reproduction above covers the
  native packages; canonical Linux Docker image reproduction remains separate.

## Current acceptance boundaries

| Requirement | Evidence and remaining gate |
| --- | --- |
| Independently hosted company setup, packages and enrollment | [Separate Windows company lab](evidence/159-windows-company-install-enrollment-20261029.json) has customer MSI enrollment, protected technician distribution and a real authorized native session. Linux company evidence is separate; neither deployment supplies hosted infrastructure for other companies. |
| Customer approval and authorization loss | Windows company receipt records attended approval, grants, relay correlation, disconnect and management-outage lease termination. [Current 1.5.10 Linux-company outage and recovery](evidence/1510-linux-native-management-outage-recovery-20261030.json) ended native access early when authorization renewal failed, refused new requests, retained closure beyond the original deadline, rejected expired renewal, and recovered a fresh customer-approved desktop session after restoring the same management container. Customer stop then closed that new server record. The exact deadline-only termination branch, public network negative-access and all advertised OS combinations remain open. |
| Clipboard and file transfer | Earlier [receiving permission denial](evidence/159-windows-receiving-file-permission-denial-20261029.json) passed. Native1.5.13 now passed [bidirectional Unicode and RGB-image clipboard](evidence/1513-native-unicode-and-image-clipboard-20261031.json) with independent saved-text/pixel hashes and viewer clipboard disabled. A [64MiB random-file roundtrip and256MiB interrupted-transfer retry](evidence/1513-native-large-transfer-and-interrupted-retry-20261031.json) passed exact hashes and partial cleanup on the Windows10 standard-user customer. The old transfer UI stayed at11% after a real20-second network outage; a fresh customer-approved retry recovered the bytes. Bounded stalled-send handling remains open. These tests use an Administrator technician and do not qualify alpha-channel images, exact resumed offsets or automatic file-transfer recovery. |
| Secure desktop and unattended consent | [Lock/unlock](evidence/159-windows-attended-lock-unlock-20261029.json), [standard-user UAC](evidence/159-windows-standard-customer-uac-20261029.json), and [unattended reboot/logon/revocation](evidence/159-windows-unattended-reboot-logon-revocation-20261029.json) passed in that lab. This is not universal Windows qualification. |
| Multiple monitors | [Two real QXL displays](evidence/159-windows-native-two-monitor-20261029.json) passed native selection and input. Other display drivers and OS combinations remain open. |
| Company branding and cache | [Both native editions](evidence/159-windows-native-branding-sync-20261030.json) automatically rendered changed branding and monotonic restoration, then rendered cached branding after restart during verified management outages. This test does not qualify software-upgrade preservation or key rotation/negative profile cases. |
| Windows encrypted recovery and transport data | [Populated migration](evidence/361d7c3-windows-populated-transport-migration-20261029.json), [running recovery/rollback](evidence/361d7c3-windows-populated-running-recovery-20261029.json), and [native session with restored data active](evidence/361d7c3-windows-native-active-restored-data-20261030.json) passed. Different-host disaster recovery remains open; Linux migration has separate evidence below. |
| Current Linux recovery and migration | [Encrypted offline restore](evidence/361d7c3-linux-preupgrade-offline-backup-restore-20261030.json) preserved keys, settings, all21 installers and six TLS identities. [Live restore, migration and rollback](evidence/361d7c3-linux-live-migration-recovery-20261030.json) then passed with fresh MFA, permissions and download checks. The latest original data was upgraded to schema3, rolled back using a fresh schema2 checkpoint, and rolled forward successfully. A subsequent [current native attended session](evidence/1510-linux-native-attended-session-20261030.json) passed against this deployment. The [profile26 archive restore](evidence/361d7c3-post-profile26-restore-and-replacement-bundles-20261030.json) recovered the rotated signing key, exact HTTPS identity, MFA and permissions,23 installer files and six TLS identities in separate storage. It preserved withdrawn downloads and revoked workers; fresh replacement bundles subsequently passed public/protected downloads. Different-host recovery and a native session to that restored clone remain open. |
| Installer worker credential protection | [Private worker checks](evidence/1c43a38-private-worker-windows-checks.json) cover the earlier Windows company migration. Every existing lab worker must be separately migrated; revoked legacy registrations must stay revoked. |
| Server 2019 upgrade and startup | The [historical 1.5.9 timeout](evidence/159-server2019-upgrade-private-worker-cold-start-failure-20261030.json) is retained. [Actual 1.5.10 upgrade and three normal reboots](evidence/1510-linux-bundles-server2019-upgrade-three-reboots-20261030.json) passed with automatic native startup, no timeout events, unchanged identity/consent, installed-file verification and private worker authorization. Physical cold-power-on, remaining desktop features and other OS startup qualification remain open. |
| Server 2025 technician upgrade | [Existing 1.5.1 technician to 1.5.10](evidence/1510-server2025-technician-upgrade-20261030.json) passed exact signed-file, publisher and shipping verification, preserving company trust and consent while refreshing signed branding. The original observer used the customer version label and failed after setup; independent verification passed without repeating installation. The installed native app subsequently rendered company branding, completed MFA login and displayed authorized devices. The subsequent session has separate evidence below; standard-user deployment and the remaining lifecycle matrix remain open. |
| Current native Linux-company session | [1.5.10 technician/customer session](evidence/1510-linux-native-attended-session-20261030.json) passed MFA, authorized inventory, named customer approval, real desktop rendering, correlated pointer/Escape input, a live lease beyond its initial 300 seconds and customer-initiated disconnect with the same server record closed. Other desktop features and external transport qualification remain open. |
| Current authorization rejection | [Company and receiving-device HTTPS checks](evidence/1510-linux-authorization-negatives-20261030.json) rejected anonymous/unassigned access, pending-device grants, unattended access without consent, changed grant payload/signature, second claim and closed-grant renewal/replay before expiry. Existing devices and consent stayed unchanged. The native unassigned account completed MFA and showed no authorized devices or support controls. A separate [stock native client](evidence/1510-native-stock-client-legacy-denial-20261030.json) received an explicit company-authorization denial with a dummy legacy password and no grant. Current outage/recovery evidence is above. The current direct TCP denial is recorded above; remaining graphical route coverage and the Windows matrix retain separate gates. |
| Approved automatic updates | [Actual Server 2019 technician automatic upgrade](evidence/1510-server2019-technician-automatic-upgrade-20261030.json) passed from 1.5.8 to 1.5.10. The same signed-in app deferred under pause, a closed maintenance window and zero-percent rollout, then downloaded and installed the approved MSI on its timer when policy opened. Exact signed files, shipping verification, identity/consent/trust preservation, automatic restart, fresh MFA/inventory and the exact company installation report passed. The original paused policy was restored. The [Windows10 customer1.5.12 receipt](evidence/1512-win10-real-session-update-deferral-and-interrupted-recovery-20261031.json) now qualifies real active-session deferral and power-loss recovery on that endpoint. Technician interruption, remaining reporting/rejection cases and the OS/edition matrix remain open; older receipts remain source-specific. |
| Public connectivity without Tailscale | Native maintained-relay tests currently use separate VMs with lab host port forwarding. External NAT traversal, external relay fallback and negative-access tests are not qualified by those results. |
| Profile security and trust rotation | [Current compiled-agent rejection checks](evidence/1510-installed-agent-profile-rejection-20261030.json) passed 28 isolated public-fixture cases on Windows 10 and Server 2019. [Live installed key rotation](evidence/1510-installed-profile-key-rotation-and-native-recovery-20261030.json) then passed automatic trusted transition/adoption on the Windows 10 customer and both Server 2019 editions, preserving identity and consent. Current shipping agents accepted new-key HTTPS controls and rejected missed-transition caches; old downloads were withdrawn. Fresh native MFA/approval/desktop and customer stop passed after rotation. The worker pin was changed explicitly, preserving its credential and ACLs. [Post-rotation restore and replacement bundles](evidence/361d7c3-post-profile26-restore-and-replacement-bundles-20261030.json) also passed. Offline-client recovery, interrupted activation and the remaining live tamper cases stay open. |

## Windows release matrix

For the current candidate, do not substitute older-version test results for
clean installation, repair, upgrade, interrupted-upgrade recovery, reboot,
uninstall and native behavior on each advertised operating system.

| Operating system, x64 Desktop Experience | Current candidate status |
| --- | --- |
| Windows 10 | The [1.5.4 approval-manager pipe failure](evidence/154-win10-receiving-approval-pipe-failure-20261030.json) remains preserved. A subsequent [automatic upgrade and current native session](evidence/1510-win10-customer-automatic-upgrade-and-session-20261030.json) reached 1.5.10 with exact signed files, shipping verification and unchanged identity/trust/consent. The older watcher required a scoped task-priority repair and controlled restart; this does not qualify the unmodified default update flow. Current customer approval, desktop, correlated pointer/Escape, lease renewal and customer Stop support passed; the same server session closed and the technician lost access. Complete lifecycle and session/update coordination remain open. |
| Windows 11 | Earlier candidate coverage exists. Current qualification remains open; the lab must stop the two smaller endpoint VMs before allocating the Windows11 memory reserve. |
| Windows Server 2016 | Substantial 1.5.9 native, installation and recovery evidence above; full install/update/uninstall matrix still open. |
| Windows Server 2019 | Explicit 1.5.9 to 1.5.10 customer upgrade and three normal reboots passed with identity/consent and private worker preserved. Current attended native session, pointer/Escape input and customer disconnect passed with the Server 2025 technician. The same-machine technician automatically upgraded from 1.5.8 to 1.5.10 after pause/window/cohort deferral checks, preserved trust and consent, restarted and completed fresh MFA/inventory and installation reporting. Remaining desktop, negative-access and installer/update cases remain open. |
| Windows Server 2022 | Unsigned 1.5.10 CI lifecycle passed; complete enrolled-machine matrix remains open. |
| Windows Server 2025 | Existing per-user technician 1.5.1 to 1.5.10 explicit upgrade passed on build26100, with exact signed files, trust and consent preserved. Native branded UI, MFA login, authorized inventory and an attended session to the Server 2019 customer passed. Standard-user installation, customer, remaining desktop and lifecycle cases remain open. |

Production publication also requires valid signing, exact corresponding source,
documented deployment/update/restore procedures and closure of the remaining
acceptance gates. The preserved Swan baseline and the company's production
RustDesk/Tailscale access paths are not replaced by these lab results.
