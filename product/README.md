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

Installers contain public bootstrap information only. The worker pins both profile and project release verification keys locally. Release metadata must include package, installed executable and agent SHA-256 identities, exact source URL and publisher. No profile value becomes a shell command.

The updater writes `pending-update.json` before installation and advances the sequence only after verifying the installed executable. `swan-agent recover-update` completes recovery only if signed installed identity matches; partial failures remain blocked for repair. Automatic repair/retry, rollback and updating the configuration agent itself remain incomplete.

## Deployment and release gates

See `deployment/windows/LOCAL-TESTING.md` for this machine's isolated test process. Windows service installation is in `deployment/windows/Install-Server.ps1`; Linux container definitions are in `deployment/linux/`. Management must sit behind trusted HTTPS before endpoint use. Companies supply DNS and publicly reachable transport or their own relay. Local tests do not prove internet reachability, NAT traversal or relay fallback.

Still required: complete native graphical technician login/inventory/history, live logo/color branding, equivalent first-run deployment wizards and reachability diagnostics, executable/MSI packaging integration and repair, company signing integration, full transport-key backup, migrations with rollback, full update recovery and agent replacement, pinned native dependency artifacts, native builds and end-to-end sessions, and clean-machine installation/upgrade tests across every advertised Windows version. Windows 10/11 x64 and Windows Server 2016/2019/2022/2025 Desktop Experience remain intended targets, not verified compatibility claims.

Never publish this development build as production. Signing-provider approval, valid production signatures, exact corresponding AGPL source and all acceptance checks are mandatory release gates. Retain RustDesk attribution and license notices when distributing changed binaries or hosting modified server code.
