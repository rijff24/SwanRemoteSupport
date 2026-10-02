# Swan Remote Support company product

Development implementation, not a production release. The original single-company application is preserved at tag `swan-single-company-baseline-1.4.9`.

Each company hosts one independent deployment. The management service uses Axum and SQLite, with an embedded administration interface. Pinned RustDesk OSS rendezvous and relay services carry transport; receiving managed applications enforce company authorization regardless of transport. The project supplies source and releases, not shared hosting or private signing keys.

## Build

Install Rust 1.90.0 and the platform C linker. From the repository root:

```
cargo +1.90.0 test --manifest-path product/Cargo.toml --workspace --locked
cargo +1.90.0 build --manifest-path product/Cargo.toml --workspace --release --locked
```

Windows production builds use MSVC. GNU compilation has verified the product components on the local development machine. Native endpoint builds additionally require the pinned Flutter and vcpkg dependencies in `.github/workflows/swan-windows-build.yml`; the company-product CI separately checks Windows and Linux management components. The root application enables management integration with the `swan_custom` feature.

## Implemented development flows

- One-time setup, local password/TOTP authentication with replay checks, administrator and technician roles, group access, pending device enrollment, revocation and auditing.
- Signed company profiles, pinned HTTPS bootstrap, profile refresh, local consent revocation and short-lived, target-bound session grants with challenge proof and authorization leases.
- Outbound authenticated worker jobs producing ZIP installation bundles from unchanged, hash-checked, Authenticode-verified release artifacts. Public customer downloads and authenticated technician downloads.
- Company release import and approval, stable/test selection, staged cohorts, pause and UTC maintenance windows. Endpoint updater checks signatures, hashes, publisher and installed executable identity, and excludes sessions during installation.
- Encrypted management backup and empty-directory restore.

Installers contain public bootstrap information only. The worker pins both profile and project release verification keys locally. Release metadata must include package, installed executable and agent SHA-256 identities, exact source URL, publisher, signing-certificate SHA-256 and supported Windows families. No profile value becomes a shell command.

The native Flutter technician home now includes company branding, MFA login, authorized inventory, attended/unattended connection requests and session history. Its Rust broker retains tokens and proof keys in memory and consumes the selected peer's ticket during the transport challenge. The graphical flow requires native build and end-to-end validation before it is considered delivered. The launcher opens this interface; automatic technician update handoff while closing the running executable remains incomplete.

The updater writes `pending-update.json` before installation and advances the sequence only after verifying the complete signed installed payload and agent. A staged, publisher-verified replacement agent performs installation and replaces the configuration agent while retaining session exclusion. Pending receipts support delayed retries; an already registered MSI uses forced repair. `swan-agent recover-update` completes recovery only when the signed installed identity matches. These paths compile and have component trust tests; actual signed self-replacement, interrupted installation and scheduled-task restart remain unverified. Full rollback and automatic technician GUI shutdown/handoff remain incomplete.

## Deployment and release gates

See `deployment/windows/LOCAL-TESTING.md` for this machine's isolated test process. Windows service installation is in `deployment/windows/Install-Server.ps1`; Linux container definitions are in `deployment/linux/`. Management must sit behind trusted HTTPS before endpoint use. Companies supply DNS and publicly reachable transport or their own relay. Local tests do not prove internet reachability, NAT traversal or relay fallback.

The pinned WiX recipes build real unsigned customer and technician MSIs locally. The customer package includes the full Flutter payload, service, Windows-owned runtime broker copy, license and configuration agent. Company bootstrap remains an explicit setup input. Build and table inspection do not prove installation or service behavior.

Still required: verify native graphical technician login/inventory/history and installed branding; complete equivalent deployment wizards, external reachability, NAT and relay tests; validate package installation, repair, upgrades and complete uninstall cleanup; deliver company signing integration; verify live transport and complete certificate-store restore, post-backup reconciliation and migration rollback; complete signed update recovery, rollback and technician handoff; and run clean-machine tests across every advertised Windows version. Windows 10/11 x64 and Windows Server 2016/2019/2022/2025 Desktop Experience remain intended targets, not verified compatibility claims. See `docs/ACCEPTANCE.md` for exact evidence and `deployment/windows/VM-TESTING.md` for the evaluation guest acceptance lab.

Never publish this development build as production. Signing-provider approval, valid production signatures, exact corresponding AGPL source and all acceptance checks are mandatory release gates. Retain RustDesk attribution and license notices when distributing changed binaries or hosting modified server code.
