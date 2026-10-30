# Release readiness — 30 October 2026

**Not ready for production publication.** This index distinguishes current,
scoped evidence from older checkpoints in [ACCEPTANCE.md](ACCEPTANCE.md).
Passing one operating system or one component does not qualify the full product.

## Sources and signing

- Development candidate `1.5.11+79` gives configuration/recovery tasks and
  noninteractive trust/update helpers normal scheduling priority. Restart and
  restore migrate the validated existing configuration task without changing
  its action or account. This source change follows a Windows10 lab repair;
  [Windows/Linux component checks](evidence/1511-component-priority-and-recovery-ci-20261030.json)
  passed, including the real Windows parent/child priority test and large-backup
  recovery. The [full native build and local archive/source verification](evidence/1511-7b60309-native-build-and-source-ci-20261030.json)
  also passed, including customer headless service startup and unsigned technician
  install/repair/uninstall on CI. All1314 tracked source files and six generated
  bridge files matched. Independent build41093808835 is running; byte comparison,
  signing and installed-endpoint qualification remain open. A matching
  [Linux lab management build](evidence/1511-7b60309-linux-lab-management-fresh-company-20261030.json)
  passed fresh-company setup, MFA/replay, enrollment, restart and backup/restore
  checks using a verified cached toolchain and a new empty build path. The live
  company remains on361; coordinated server/worker upgrade, new-image HTTPS/native
  integration and canonical Docker image reproduction remain separate gates. The exact 1.5.10 receipts
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
| Clipboard and file transfer | [Native roundtrip](evidence/159-windows-native-clipboard-transfer-20261029.json) and [receiving permission denial](evidence/159-windows-receiving-file-permission-denial-20261029.json) passed in the separate Server 2016 lab. Unicode/image clipboard and large/interrupted transfer qualification remain open. |
| Secure desktop and unattended consent | [Lock/unlock](evidence/159-windows-attended-lock-unlock-20261029.json), [standard-user UAC](evidence/159-windows-standard-customer-uac-20261029.json), and [unattended reboot/logon/revocation](evidence/159-windows-unattended-reboot-logon-revocation-20261029.json) passed in that lab. This is not universal Windows qualification. |
| Multiple monitors | [Two real QXL displays](evidence/159-windows-native-two-monitor-20261029.json) passed native selection and input. Other display drivers and OS combinations remain open. |
| Company branding and cache | [Both native editions](evidence/159-windows-native-branding-sync-20261030.json) automatically rendered changed branding and monotonic restoration, then rendered cached branding after restart during verified management outages. This test does not qualify software-upgrade preservation or key rotation/negative profile cases. |
| Windows encrypted recovery and transport data | [Populated migration](evidence/361d7c3-windows-populated-transport-migration-20261029.json), [running recovery/rollback](evidence/361d7c3-windows-populated-running-recovery-20261029.json), and [native session with restored data active](evidence/361d7c3-windows-native-active-restored-data-20261030.json) passed. Different-host disaster recovery remains open; Linux migration has separate evidence below. |
| Current Linux recovery and migration | [Encrypted offline restore](evidence/361d7c3-linux-preupgrade-offline-backup-restore-20261030.json) preserved keys, settings, all21 installers and six TLS identities. [Live restore, migration and rollback](evidence/361d7c3-linux-live-migration-recovery-20261030.json) then passed with fresh MFA, permissions and download checks. The latest original data was upgraded to schema3, rolled back using a fresh schema2 checkpoint, and rolled forward successfully. A subsequent [current native attended session](evidence/1510-linux-native-attended-session-20261030.json) passed against this deployment. The [profile26 archive restore](evidence/361d7c3-post-profile26-restore-and-replacement-bundles-20261030.json) recovered the rotated signing key, exact HTTPS identity, MFA and permissions,23 installer files and six TLS identities in separate storage. It preserved withdrawn downloads and revoked workers; fresh replacement bundles subsequently passed public/protected downloads. Different-host recovery and a native session to that restored clone remain open. |
| Installer worker credential protection | [Private worker checks](evidence/1c43a38-private-worker-windows-checks.json) cover the earlier Windows company migration. Every existing lab worker must be separately migrated; revoked legacy registrations must stay revoked. |
| Server 2019 upgrade and startup | The [historical 1.5.9 timeout](evidence/159-server2019-upgrade-private-worker-cold-start-failure-20261030.json) is retained. [Actual 1.5.10 upgrade and three normal reboots](evidence/1510-linux-bundles-server2019-upgrade-three-reboots-20261030.json) passed with automatic native startup, no timeout events, unchanged identity/consent, installed-file verification and private worker authorization. Physical cold-power-on, remaining desktop features and other OS startup qualification remain open. |
| Server 2025 technician upgrade | [Existing 1.5.1 technician to 1.5.10](evidence/1510-server2025-technician-upgrade-20261030.json) passed exact signed-file, publisher and shipping verification, preserving company trust and consent while refreshing signed branding. The original observer used the customer version label and failed after setup; independent verification passed without repeating installation. The installed native app subsequently rendered company branding, completed MFA login and displayed authorized devices. The subsequent session has separate evidence below; standard-user deployment and the remaining lifecycle matrix remain open. |
| Current native Linux-company session | [1.5.10 technician/customer session](evidence/1510-linux-native-attended-session-20261030.json) passed MFA, authorized inventory, named customer approval, real desktop rendering, correlated pointer/Escape input, a live lease beyond its initial 300 seconds and customer-initiated disconnect with the same server record closed. Other desktop features and external transport qualification remain open. |
| Current authorization rejection | [Company and receiving-device HTTPS checks](evidence/1510-linux-authorization-negatives-20261030.json) rejected anonymous/unassigned access, pending-device grants, unattended access without consent, changed grant payload/signature, second claim and closed-grant renewal/replay before expiry. Existing devices and consent stayed unchanged. The native unassigned account completed MFA and showed no authorized devices or support controls. A separate [stock native client](evidence/1510-native-stock-client-legacy-denial-20261030.json) received an explicit company-authorization denial with a dummy legacy password and no grant. Current outage/recovery evidence is above. Direct-IP native paths and the remaining Windows matrix remain separate gates. |
| Approved automatic updates | [Actual Server 2019 technician automatic upgrade](evidence/1510-server2019-technician-automatic-upgrade-20261030.json) passed from 1.5.8 to 1.5.10. The same signed-in app deferred under pause, a closed maintenance window and zero-percent rollout, then downloaded and installed the approved MSI on its timer when policy opened. Exact signed files, shipping verification, identity/consent/trust preservation, automatic restart, fresh MFA/inventory and the exact company installation report passed. The original paused policy was restored. Current active-session coordination, interrupted recovery, download-progress reporting, publisher rejection and the remaining Windows/customer update matrix remain release gates; older update receipts remain source-specific. |
| Public connectivity without Tailscale | Native maintained-relay tests currently use separate VMs with lab host port forwarding. External NAT traversal, external relay fallback and negative-access tests are not qualified by those results. |
| Profile security and trust rotation | [Current compiled-agent rejection checks](evidence/1510-installed-agent-profile-rejection-20261030.json) passed 28 isolated public-fixture cases on Windows 10 and Server 2019. [Live installed key rotation](evidence/1510-installed-profile-key-rotation-and-native-recovery-20261030.json) then passed automatic trusted transition/adoption on the Windows 10 customer and both Server 2019 editions, preserving identity and consent. Current shipping agents accepted new-key HTTPS controls and rejected missed-transition caches; old downloads were withdrawn. Fresh native MFA/approval/desktop and customer stop passed after rotation. The worker pin was changed explicitly, preserving its credential and ACLs. [Post-rotation restore and replacement bundles](evidence/361d7c3-post-profile26-restore-and-replacement-bundles-20261030.json) also passed. Offline-client recovery, interrupted activation and the remaining live tamper cases stay open. |

## Windows release matrix

For the current candidate, do not substitute older-version test results for
clean installation, repair, upgrade, interrupted-upgrade recovery, reboot,
uninstall and native behavior on each advertised operating system.

| Operating system, x64 Desktop Experience | Current candidate status |
| --- | --- |
| Windows 10 | The [1.5.4 approval-manager pipe failure](evidence/154-win10-receiving-approval-pipe-failure-20261030.json) remains preserved. A subsequent [automatic upgrade and current native session](evidence/1510-win10-customer-automatic-upgrade-and-session-20261030.json) reached 1.5.10 with exact signed files, shipping verification and unchanged identity/trust/consent. The older watcher required a scoped task-priority repair and controlled restart; this does not qualify the unmodified default update flow. Current customer approval, desktop, correlated pointer/Escape, lease renewal and customer Stop support passed; the same server session closed and the technician lost access. Complete lifecycle and session/update coordination remain open. |
| Windows 11 | Earlier candidate coverage exists; current test is deferred while the host cannot retain its memory reserve with the company server running. |
| Windows Server 2016 | Substantial 1.5.9 native, installation and recovery evidence above; full install/update/uninstall matrix still open. |
| Windows Server 2019 | Explicit 1.5.9 to 1.5.10 customer upgrade and three normal reboots passed with identity/consent and private worker preserved. Current attended native session, pointer/Escape input and customer disconnect passed with the Server 2025 technician. The same-machine technician automatically upgraded from 1.5.8 to 1.5.10 after pause/window/cohort deferral checks, preserved trust and consent, restarted and completed fresh MFA/inventory and installation reporting. Remaining desktop, negative-access and installer/update cases remain open. |
| Windows Server 2022 | Unsigned 1.5.10 CI lifecycle passed; complete enrolled-machine matrix remains open. |
| Windows Server 2025 | Existing per-user technician 1.5.1 to 1.5.10 explicit upgrade passed on build26100, with exact signed files, trust and consent preserved. Native branded UI, MFA login, authorized inventory and an attended session to the Server 2019 customer passed. Standard-user installation, customer, remaining desktop and lifecycle cases remain open. |

Production publication also requires valid signing, exact corresponding source,
documented deployment/update/restore procedures and closure of the remaining
acceptance gates. The preserved Swan baseline and the company's production
RustDesk/Tailscale access paths are not replaced by these lab results.
