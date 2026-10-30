# Release readiness — 30 October 2026

**Not ready for production publication.** This index distinguishes current,
scoped evidence from older checkpoints in [ACCEPTANCE.md](ACCEPTANCE.md).
Passing one operating system or one component does not qualify the full product.

## Sources and signing

- Working candidate `1.5.10+78` adds an early service-manager connection before
  loading the native core. It is not yet built, installed or qualified; the
  [startup design and boundary tests](../deployment/windows/SERVICE-STARTUP.md)
  do not clear the recorded 1.5.9 intermittent startup failure.
- Native customer/technician candidate: `aff8ff61e3fbd6f34c4cf0bb9de6e330ec6e0521`,
  version `1.5.9+77`.
- Management storage/recovery candidate:
  `361d7c362de7f67b7b7456d33bf0191736eec7d5`.
- Migrated Windows installer worker:
  `1c43a382c87b05e44caff1c6c2bef4cf0b0bb6fc`.
- These Windows lab packages use a disposable lab publisher. Production signing
  approval and production-signed release artifacts remain outstanding.
- [Windows/Linux component reproduction](evidence/361d7c3-push-pr-windows-linux-component-reproduction-20261030.json)
  establishes identical management, agent and worker bytes in the two recorded
  builds. It does not establish native Flutter/engine/MSI reproduction.

## Current acceptance boundaries

| Requirement | Evidence and remaining gate |
| --- | --- |
| Independently hosted company setup, packages and enrollment | [Separate Windows company lab](evidence/159-windows-company-install-enrollment-20261029.json) has customer MSI enrollment, protected technician distribution and a real authorized native session. Linux company evidence is separate; neither deployment supplies hosted infrastructure for other companies. |
| Customer approval and authorization loss | Windows company receipt records attended approval, grants, relay correlation, disconnect and management-outage lease termination. Public network negative-access and all advertised OS combinations remain open. |
| Clipboard and file transfer | [Native roundtrip](evidence/159-windows-native-clipboard-transfer-20261029.json) and [receiving permission denial](evidence/159-windows-receiving-file-permission-denial-20261029.json) passed in the separate Server 2016 lab. Unicode/image clipboard and large/interrupted transfer qualification remain open. |
| Secure desktop and unattended consent | [Lock/unlock](evidence/159-windows-attended-lock-unlock-20261029.json), [standard-user UAC](evidence/159-windows-standard-customer-uac-20261029.json), and [unattended reboot/logon/revocation](evidence/159-windows-unattended-reboot-logon-revocation-20261029.json) passed in that lab. This is not universal Windows qualification. |
| Multiple monitors | [Two real QXL displays](evidence/159-windows-native-two-monitor-20261029.json) passed native selection and input. Other display drivers and OS combinations remain open. |
| Company branding and cache | [Both native editions](evidence/159-windows-native-branding-sync-20261030.json) automatically rendered changed branding and monotonic restoration, then rendered cached branding after restart during verified management outages. This test does not qualify software-upgrade preservation or key rotation/negative profile cases. |
| Encrypted recovery and transport data | [Populated migration](evidence/361d7c3-windows-populated-transport-migration-20261029.json), [running recovery/rollback](evidence/361d7c3-windows-populated-running-recovery-20261029.json), and [native session with restored data active](evidence/361d7c3-windows-native-active-restored-data-20261030.json) passed. Different-host disaster recovery and current Linux migration still require their own evidence. |
| Installer worker credential protection | [Private worker checks](evidence/1c43a38-private-worker-windows-checks.json) cover the earlier Windows company migration. Every existing lab worker must be separately migrated; revoked legacy registrations must stay revoked. |
| Server 2019 upgrade and startup | [1.5.1 to 1.5.9 upgrade and worker migration](evidence/159-server2019-upgrade-private-worker-cold-start-failure-20261030.json) preserved customer identity, trust, credential and consent. The private worker restarted automatically and completed a post-reboot build. **Customer automatic startup failed with a 30-second service connection timeout.** A subsequent traced normal reboot passed without manual startup; the intermittent failure remains unresolved. |
| Approved automatic updates | Earlier explicit/automatic update, interruption and rollout tests are source-specific records in ACCEPTANCE.md. They are not blanket approval of 1.5.9 across every OS. Complete current-candidate update/recovery, active-session coordination, publisher-policy and rollout tests remain release gates. |
| Public connectivity without Tailscale | Native maintained-relay tests currently use separate VMs with lab host port forwarding. External NAT traversal, external relay fallback and negative-access tests are not qualified by those results. |
| Profile security and trust rotation | Earlier signed-profile, rejection and rotation records are source-specific. Full current installed-edition tamper, expiry, rollback, wrong-company and trusted rotation qualification remains to be collected. |

## Windows release matrix

For the current candidate, do not substitute older-version test results for
clean installation, repair, upgrade, interrupted-upgrade recovery, reboot,
uninstall and native behavior on each advertised operating system.

| Operating system, x64 Desktop Experience | Current candidate status |
| --- | --- |
| Windows 10 | Earlier candidate coverage exists; complete 1.5.9 matrix remains open. |
| Windows 11 | Earlier candidate coverage exists; current test is deferred while the host cannot retain its memory reserve with the company server running. |
| Windows Server 2016 | Substantial 1.5.9 native, installation and recovery evidence above; full install/update/uninstall matrix still open. |
| Windows Server 2019 | Explicit 1.5.1 to 1.5.9 upgrade and private worker migration passed; customer automatic startup after reboot failed. Full qualification remains open. |
| Windows Server 2022 | Earlier candidate coverage exists; complete 1.5.9 matrix remains open. |
| Windows Server 2025 | Earlier candidate coverage exists; complete 1.5.9 matrix remains open. |

Production publication also requires valid signing, exact corresponding source,
documented deployment/update/restore procedures and closure of the remaining
acceptance gates. The preserved Swan baseline and the company's production
RustDesk/Tailscale access paths are not replaced by these lab results.
