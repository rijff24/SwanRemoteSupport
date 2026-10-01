# Swan Remote Support completion assessment

Research date: 1 October 2026. This assessment covers the company-configurable open-source product: a self-hosted server, public customer installer, internal technician installer, branding and policy synchronization, and automatic updates without requiring Tailscale.

The remote desktop engine and a Swan-specific unsigned customer build exist. The reusable company administration and distribution system does not yet exist in this checkout. We should extend the RustDesk OSS foundation with our own open-source management service and company configuration protocol. The proposed architecture below is a recommendation, not implemented behavior.

## Evidence and limits

Inspected checkout: `SwanRemoteSupport-main`, branch `main`, commit `9494a5a`, initially clean. The outer workspace repository has no commits; `SwanRemoteSupport` contains only Git metadata. The substantive application is in `SwanRemoteSupport-main`.

The GitHub API confirms the Swan Windows build [32726437306](https://github.com/rijff24/SwanRemoteSupport/actions/runs/32726437306) succeeded on 24 August 2026 from `0e400fd`. The API returned no published releases. A successful build is evidence of compilation and packaging, not acceptance testing or trusted signing. The local change log records verification of the unsigned artifact and preparation of a two-laptop test kit.

Today, the local `RustDesk`, `hbbs`, and `hbbr` Windows services are running with automatic startup. Earlier server hardening and recovery records are documented evidence; public internet reachability, current firewall policy, customer consent flows, and remote session behavior were not revalidated in this research. No live network or service configuration was changed.

The `libs/hbb_common` submodule is not initialized in this checkout. Cargo, Flutter, and CMake were not found on PATH in the research shell. No application build or end-to-end test was performed here.

## What already exists

| Area | Finding | Evidence |
| --- | --- | --- |
| Remote desktop foundation | RustDesk 1.4.9 client with upstream screen control, transfer, clipboard, service and relay code | `Cargo.toml`, `src/client.rs`, `src/server/`, `src/rendezvous_mediator.rs` |
| Customer branding | Swan logos, light/dark assets, product text, icon and Windows metadata | `branding/`, `flutter/windows/runner/Runner.rc`, `flutter/lib/common.dart` |
| Managed customer behavior | Incoming-only defaults, hidden ID/password panel and server settings, permanent-password setup | `src/common.rs:load_swan_client_defaults`, `flutter/lib/desktop/pages/desktop_home_page.dart` |
| Customer setup | Express installer and PowerShell deployment; unique password generation, credential receipt, rotation and revocation helpers | `scripts/`, `docs/SWAN_CUSTOM_BUILD.md` |
| Windows packaging | Manual x64 workflow producing an unsigned self-extracting EXE and checksum | `.github/workflows/swan-windows-build.yml` |
| Technician test launcher | Copies the same unsigned customer binary under a different filename to avoid opening the installer | `scripts/Run-Swan-Technician-UNSIGNED-TEST.cmd` |
| Server transport | Existing RustDesk OSS rendezvous and relay services; August audit recorded version 1.1.15 | `docs/AI_CHANGELOG.md` |
| Reliability | Windows service recovery and startup/hourly monitoring on this machine | `../rustdesk-reliability/README.md` |
| Release governance | Source availability, privacy, signing policy and acceptance documentation | `docs/SOURCE_AND_LICENSE.md`, `docs/CODE_SIGNING_POLICY.md` |

The technician launcher is not a separately configured technician product. It changes the filename, while `load_swan_client_defaults` still forces incoming-only settings and the Swan managed UI. Its ability to initiate the intended support sessions must be checked and corrected rather than assumed.

## Gaps against the requested product

| Requirement | Current gap |
| --- | --- |
| A company configures its own server | No company setup wizard, branding editor, technician administration, device enrollment service, or installer generator found |
| Clients inherit company branding and setup | Swan name, assets, private-network address and public key are built into the app/workflow; no company profile synchronization found |
| Website download and unattended configuration | Existing process expects an in-person technician, Tailscale login/tag approval, and manual credential collection |
| Internal technician edition | Need authenticated technician login, outgoing support UI, permissions, device inventory and proper packaging |
| Server generates installers | Current packaging runs in GitHub Actions; no server build queue or downloadable company packages |
| Automatic updates | Swan explicitly overrides `allow-auto-update` to `N`; upstream updater is not a Swan/company release system |
| All major Windows versions | Only the Swan x64 Flutter build is verified; no Swan ARM64, legacy x86, or Windows Server compatibility evidence |
| No Tailscale requirement | Installer, defaults, customer text and existing server deployment depend on Tailscale |
| Customer-ready publication | No published signed release or completed acceptance evidence found |

Upstream custom configuration is not immediately usable as our company profile format: `read_custom_client` verifies `custom.txt` using a fixed upstream public signing key. We need our own explicitly trusted profile mechanism. Also, `API_SERVER` currently refers to the RustDesk Pro protocol; pointing it at a new generic REST service will not create compatible login or configuration sync.

## Proposed company server and synchronization

Use one self-hosted deployment per company initially. Keep `hbbs` and `hbbr` as the transport and add an open-source management service with a database, administration UI, configuration API, enrollment API, audit records, download endpoints and release management.

Company setup should collect the public service hostname, transport addresses, server public key, company name, logos, theme, support contacts, consent wording, technician accounts, access policies and update channel. Provide backup and restore for configuration, device identity and server keys.

Each company installer needs a small initial configuration containing the management URL, company identifier, intended edition, and a trusted public configuration key. The installer must contain no administrator credentials, shared unattended password, signing private key or reusable technician enrollment secret.

On installation and subsequent refreshes, the app verifies and applies a versioned company profile, caches the last valid profile, and retries with backoff when offline. Protect against invalid signatures, expired metadata, older profile replay, and silent changes of company ownership. Plan trusted key rotation and address migration before shipping.

Public customer downloads are public: their embedded values cannot grant technician privileges. Enrollment needs unique per-device credentials and appropriate approval or scoped invitation. Unattended access requires clear owner consent. Avoid the current need for customers to copy support passwords into a technician's password manager.

Technicians authenticate separately with MFA and server-enforced device/group permissions. Hiding buttons or specifying an outgoing/incoming role is not authorization. Authorization must also protect direct and relayed sessions, device revocation and stolen credentials; implementing this likely requires transport/client changes in addition to the management API.

Allow logo, contact, theme and policy changes to sync without reinstalling. Windows executable icons, publisher/version resources and installation identity are packaging concerns: changing these may require a newly generated signed package. Use stable internal application, service and configuration identifiers so a display-name change does not lose device identity or break upgrades.

## Generating and publishing installers

Recommended product behavior: the company administrator clicks **Generate installers** and receives customer EXE/MSI packages for its website and technician EXE/MSI packages for authenticated internal distribution.

The server can orchestrate this, but Windows compilation should run on isolated Windows workers with pinned source/toolchains. A Linux-hosted company server can queue a Windows worker or CI job. Begin by adapting the working GitHub Actions build and upstream WiX MSI scaffolding in `res/msi/`; the latter is not already a finished Swan MSI pipeline.

Build shared runtime binaries per version/architecture, then assemble company-specific configuration and branding. Where resource customization changes a PE binary, sign the final customized binary. Sign inner executables/DLLs before packaging and the final installer afterward. Do not modify a signed artifact to insert company configuration. Keep signing access separate from the publicly reachable runtime server.

Each download should identify edition, architecture, version, publisher, hash and corresponding source. Keep an immutable release record; a stable website download link may point to the latest approved package. Protect technician downloads, but enforce technician authorization even if someone obtains that installer.

[Windows Installer major upgrades](https://learn.microsoft.com/en-us/windows/win32/msi/major-upgrades) provide package upgrade mechanisms and downgrade prevention. Proposed installers must preserve device identity, enrollment and consent through upgrade/repair, and implement explicit uninstall and recovery behavior.

## Automatic updates

Do not simply enable the existing upstream updater. `src/common.rs` checks the upstream version service, and `src/updater.rs` constructs RustDesk release filenames. It does not implement a company-approved Swan release feed. Its size-based cached-download reuse is not a package integrity check.

Implement a company-controlled feed identifying product, edition, architecture, OS compatibility, version, download URL, hash and metadata expiry. Verify signed metadata and artifact integrity, plus trusted Authenticode publisher policy, before privileged installation. Evaluate [The Update Framework](https://theupdateframework.io/docs/overview/) for the update trust design.

Add stable/test channels, staged rollout, maintenance windows, pause controls, retry/backoff and explicit status reporting. Preserve enrollment and settings. Coordinate installation with both client roles so sessions cannot start while an update is being applied. Define recovery after interrupted installation and controlled rollback to an approved version without permitting arbitrary replay of old releases. Separate company configuration revisions from software versions.

Clients can update automatically within company policy. Server updates should initially require an administrator maintenance action with database/key backups and tested migration rollback; a failed server upgrade could affect every customer.

## Removing Tailscale

This is feasible with the existing transport. [RustDesk self-hosting documentation](https://rustdesk.com/docs/en/self-host/) describes direct NAT traversal with relay fallback. Native clients need reachable server TCP 21115, 21116 and 21117, plus UDP 21116. Proposed management/download APIs use HTTPS 443.

Recommended deployment is a server with a public address and DNS name. An office-hosted server needs working port forwarding and a reachable public address; CGNAT may require a public VPS or another reachable relay. Ordinary shared website hosting does not necessarily permit the required long-running TCP/UDP services. A website can host downloads while the transport runs elsewhere.

Use DNS names rather than a Tailscale-only address. Validate direct and forced-relay sessions from independent external networks, including restrictive corporate networks; do not promise that the existing native transport works through every HTTPS-only proxy.

Tailscale currently provides network restrictions as well as connectivity. A public OSS server does not automatically reproduce technician identities, MFA or company access policy. Implement and validate replacement authorization, rate limiting, abuse controls and auditability. The public server key is a trust value, not an access secret. Migrate through a parallel endpoint and tested rollback before removing the existing private access path.

## Windows support and open source

Recommended initial release target: Windows 10 and 11 x64. Add ARM64 after validating the project's Rust, Flutter engine, plugins and native dependencies together. [Flutter's current supported-platform matrix](https://docs.flutter.dev/reference/supported-platforms) lists Windows 10/11 x64 and ARM64 and excludes Windows 8 and earlier; it does not prove compatibility of our pinned Flutter 3.24.5 and custom RustDesk engine.

Windows 7/8/8.1 and 32-bit Windows require a separate legacy UI/toolchain assessment and actual tests. Windows Server releases need explicit testing of interactive sessions, logon screen, UAC and service behavior. Installer format alone cannot make an unsupported runtime compatible. Do not advertise all Windows versions until the support matrix is tested.

Retain AGPL notices and exact corresponding-source/build links for distributed modified binaries and assess the new management service's license obligations. The [GNU AGPL](https://www.gnu.org/licenses/agpl.en.html) includes obligations for modified network software. Company keys and customer data remain private. Existing [RustDesk branded client generation](https://rustdesk.com/docs/en/self-host/client-configuration/) is a Pro feature; our fully open-source design needs its own implementation.

Signing remains unresolved. The existing SignPath policy documents an intended process, not proof of project approval. [SignPath Foundation terms](https://signpath.org/terms.html) have eligibility and provenance requirements; confirm coverage of company-specific generated builds. Publisher identity follows the actual certificate, not the company logo. Automatic per-company trusted signing cannot be assumed free.

## Recommended completion order

1. **Reproduce the baseline.** Initialize the pinned submodule in the development/build environment, rebuild the customer package, test the technician-role issue, and record current install/reboot/remote-session results. Correct stale documentation: the build has succeeded, and the spare-laptop guide uses `tag:swan-operator` where the policy uses `tag:swan-support-operator`.
2. **Implement company management and trust.** Deliver setup, profiles, enrollment, technician authentication, device groups and enforced session permissions. Demonstrate one company's profile cannot authorize access to another deployment.
3. **Create the two editions and sync.** Remove Swan-only assumptions, implement customer self-setup and technician login, and verify live profile refresh, offline caching and key rotation.
4. **Validate public transport.** Prepare a parallel public endpoint, implement replacement access/abuse controls, and pass external direct/relay and negative-access tests without Tailscale.
5. **Deliver signed packages.** Add company packaging jobs, proper MSI/EXE install/repair/uninstall, website downloads and protected technician distribution.
6. **Deliver safe updates.** Test valid updates, invalid signatures/hashes, wrong company/edition/architecture, expired/replayed metadata, active sessions, interrupted installation and recovery.
7. **Publish the supported release.** Complete clean-machine acceptance on every advertised OS/architecture; publish signed hashes, exact source, privacy and recovery instructions. Extend legacy compatibility only with separate evidence.

Completion means a fresh company deployment can configure branding once, generate both installers, publish the customer package, enroll a consenting customer without manual network configuration, authenticate an authorized technician, connect without Tailscale, change branding centrally, and upgrade both apps without losing identity or access controls.
