# Swan Remote Support build and deployment guide

## Purpose and current status

This branch turns the RustDesk `1.4.9` client into **Swan Remote Support** for Swan Computing. It provides:

- Swan Computing light/dark logos, application icon, product name, and Windows file metadata.
- Embedded public routing/trust defaults for the Tailscale-only RustDesk ID/relay server and its Ed25519 public key.
- No fallback from a Swan build to the public RustDesk rendezvous network.
- Incoming-only operation with the customer-facing device ID and password hidden after installation.
- A one-page express installer that creates and displays a different cryptographically random unattended password for every customer device.
- Optional command-line helpers for silent installation, unattended-access rotation, and revocation.
- A one-entry Windows deployment flow that verifies or installs the official Tailscale client, guides manual login, enforces the customer device tag, and validates server reachability before Swan installation.
- A Windows x64 GitHub Actions build that produces an unsigned self-extracting installer and SHA-256 checksum.

The installer is **not ready for customer distribution** until a build succeeds, every Swan executable/DLL and the outer installer are code-signed, and the acceptance checklist below passes.

## Product and design decisions

The product name is `Swan Remote Support`. The visual system uses Swan Computing teal (`#004B6E`) and the existing gear/swan mark. The wordmark is teal in light mode and white in dark mode. RustDesk attribution and the upstream AGPL licence remain in place.

The editable brand sources are in `branding/`. Flutter keeps rendered PNGs because the home-screen logo loader uses `Image.asset`; the Windows runner keeps ICO resources because the resource compiler requires that format. To regenerate them, render `swan-logo-light.svg` and `swan-logo-dark.svg` to transparent 600-pixel-wide PNGs without changing aspect ratio, render `swan-mark.svg` to a transparent 512-by-512 PNG, and copy the approved multi-resolution Swan favicon to the two tracked ICO paths. Confirm the light/dark assets visually after every regeneration.

The distributed client is a managed-support edition installed in person by Swan Computing:

- `conn-type=incoming`
- `approve-mode=password`
- `verification-method=use-permanent-password`
- `temporary-password-length=10`
- `allow-only-conn-window-open=N`
- `allow-logon-screen-password=Y`

The express installer generates a unique 24-character password, requires the installer to copy the device receipt before continuing, stores only RustDesk's protected password representation in the imported client configuration, and hides the ID/password from the ordinary customer screen. The tray icon and stop-service controls stay visible so the computer owner can see and pause the service.

## Required server information

The following public values were decoded from the supplied RustDesk configuration and independently checked against the supplied screenshot. Never copy the private `id_ed25519` file into this repository, an Actions variable, an installer, or a customer device.

| Build value | Embedded default | Purpose |
| --- | --- | --- |
| `RENDEZVOUS_SERVER` | `100.82.236.84` | Tailscale address of `hbbs`. |
| `RELAY_SERVER` | `100.82.236.84` | Tailscale address of `hbbr`. |
| `RS_PUB_KEY` | `I0s1JvqJ19WDNKBLFY+FC5HekcOqNWPpH+6xGmKUQLI=` | Exact 32-byte Ed25519 server public key in Base64. |
| `API_SERVER` | empty | No Pro API/account server is used. |

The ID server and public key are embedded in the client binary and should be treated as public routing/trust configuration, not secrets. The private server key and any Pro licence key are secrets and must not be embedded.

These defaults can still be overridden with build environment variables for a controlled test build. Do not override them in a customer release without updating this guide and rerunning every network/configuration acceptance test.

Because `100.82.236.84` is a Tailscale address, each customer PC and the Swan technician PC must be connected to the reviewed tailnet policy. The native client requires TCP `21115-21117` and UDP `21116`; the installed Swan client obtaining a numeric ID is the final operational registration check. Customer devices must have `tag:swan-customer`, the server must have `tag:swan-rustdesk-server`, and technician computers must have `tag:swan-support-operator`. See [Tailscale deployment](TAILSCALE_DEPLOYMENT.md) and its complete example policy.

## OSS versus Pro access control

RustDesk Server OSS routes encrypted connections but does not provide a Swan-only user identity policy. On OSS, “only Swan Computing” means that Swan Computing is the only party who knows each device's unique ID/password pair. Anyone who obtains both can attempt to connect. Use a password manager, never reuse passwords, and disable unattended access when support ends.

RustDesk Server Pro is the stronger choice when unattended access must be tied to an actual technician identity. Configure a Swan technician account with MFA, assign devices to the correct user/group, and apply server-side roles/strategies. Test that a second unprivileged account cannot access the device. The client-side scripts in this branch do not replace Pro authorization policy.

## Build on GitHub Actions

1. Create a Swan-controlled public GitHub fork. Keep the AGPL corresponding-source obligations below in mind for every distributed binary.
2. Open **Actions → Build Swan Remote Support for Windows x64 → Run workflow**.
3. Download the `SwanRemoteSupport-1.4.9-x64-unsigned` artifact.
4. Verify the executable against `SHA256SUMS.txt` before signing or testing.
5. Sign every Swan executable and DLL before packaging, then sign and timestamp the final outer installer. The current unsigned workflow artifact is a build input, not a customer release.

The workflow validates the server address shape, Base64 public key, key length, and optional HTTPS API URL before compiling. The source-level defaults mean an ordinary Swan build does not require repository variables.

### Code signing

The workflow intentionally labels its output `unsigned`. Do not give the unsigned build to customers as a finished product: Windows SmartScreen, Smart App Control, and antivirus products may block it, and users cannot verify its publisher.

See the [Swan code-signing policy](CODE_SIGNING_POLICY.md) and [Windows code signing and Defender release process](WINDOWS_CODE_SIGNING.md). Signing only the outer self-extracting installer is insufficient; the installed Swan executables and DLLs must also be signed. Keep the signing key in a hardware-backed or managed signing service and never commit a PFX or password.

```powershell
Get-AuthenticodeSignature -LiteralPath .\SwanRemoteSupport-1.4.9-x64.exe | Format-List
Get-FileHash -Algorithm SHA256 -LiteralPath .\SwanRemoteSupport-1.4.9-x64.exe
```

Record the final signed hash in the customer download page or release record.

## Local build

The current development PC did not have Rust, Cargo, CMake, Ninja, Flutter, LLVM/Clang, or `vcpkg` installed when this branch was created. Follow the pinned versions in `.github/workflows/swan-windows-build.yml` or RustDesk's Windows build documentation rather than installing arbitrary latest versions.

After the toolchain is installed, run from an MSVC-capable shell:

```powershell
$env:SWAN_BUILD = "1"
python .\build.py --portable --flutter --hwcodec --vram
```

The Swan server defaults are built in when `SWAN_BUILD=1`; the normal upstream build remains unchanged. Do not place private keys or customer credentials in a `.env` file. The repository ignores `.env`, but local plaintext still creates avoidable risk.

## Express installation on a customer PC

Prerequisites:

- The reviewed Tailnet policy is already active, the Swan server and technician device have their correct tags, and the person installing can manually sign the customer device into the correct tailnet.
- You are physically present and have the computer owner's authorization for unattended support.
- Your Swan password manager is open and ready for a new unique device record.
- The release has valid Authenticode signatures and its published SHA-256 matches.

Preferred one-entry deployment-bundle flow:

1. Extract the signed release bundle to a local folder; do not run it from inside the ZIP.
2. Double-click `Install-SwanRemoteSupport.cmd` and approve Windows UAC.
3. The bootstrap verifies the Swan signature, downloads/verifies/installs the official Tailscale MSI if necessary, and opens the manual Tailscale login.
4. Sign in to the Swan-managed tailnet. If prompted, approve or assign `tag:swan-customer` in the Tailscale admin console.
5. Let the bootstrap validate the tag and required RustDesk ports, then install Swan and start the service.
6. Immediately paste the copied computer name, Device ID, and unique password receipt into the correct customer/device record in the Swan password manager, verify it, and clear the clipboard.
7. Open the installed app and confirm **Ready for unattended support** and **Online**.
8. From the Swan technician PC, make one test connection using the saved unique ID/password.
9. Reboot the customer PC and repeat the connection test at the Windows sign-in screen.
10. Explain the visible tray icon and how the customer can stop or uninstall support.

This is one installation entry point, not one monolithic executable. Tailscale is intentionally downloaded as an unchanged, separately signed product so Swan does not rebrand or redistribute Tailscale's proprietary Windows GUI. A future wrapper executable must not be released until Tailscale's redistribution terms and SignPath's component rules have been reviewed.

For an isolated local test of an unsigned Swan build, an administrator may invoke the PowerShell helper with `-AllowUnsignedSwanInstaller`. The separate `-SkipTailscaleCheck` switch bypasses the network gate. Both switches are deliberately noisy and are prohibited for every customer installation.

The original signed Swan `install.exe` GUI remains available when Tailscale is already correctly installed and tagged. It clears the exact credential receipt from the clipboard after 90 seconds unless it has already been replaced. The deployment-bundle flow requires manual clipboard clearing immediately after the password-manager record is verified.

To rotate/re-enable unattended access after installation, open elevated PowerShell and run:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
& .\scripts\Enable-SwanUnattendedAccess.ps1 -CopyCredentialsToClipboard
```

The helper:

- requires Windows administrator rights;
- creates a 24-character password with a cryptographic random-number generator;
- enables permanent-password approval and background availability;
- copies the device ID/password receipt once for secure transfer to Swan Computing; visible terminal output is available only with the explicit higher-risk `-ShowCredentials` switch;
- never stores a plaintext password in this repository or reuses it on another device.

Store the ID/password in an organization password manager with customer/device ownership and access logging. Do not send credentials by ordinary email or an unencrypted chat.

To revoke unattended access and restore customer approval:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
& .\scripts\Disable-SwanUnattendedAccess.ps1
```

The permanent password may remain protected in the local RustDesk configuration, but this rollback disables its use by switching verification back to temporary-password mode and requiring the app window to be open. To remove all local support data, uninstall Swan Remote Support and confirm the customer-specific configuration directory is removed according to the customer's retention policy.

## Acceptance checklist before customer use

- Build completes from a clean checkout and the expected upstream tag/submodule revisions.
- The installer and installed executable both show Swan Computing branding and the correct icon.
- Light and dark themes show a readable, undistorted logo.
- Windows Apps & Features lists `Swan Remote Support` and the intended publisher.
- The client registers only with the Swan ID server; firewall/DNS logs show no RustDesk public rendezvous fallback.
- The customer device is online with `tag:swan-customer`; it can reach only the declared RustDesk server ports and cannot initiate traffic to another customer device.
- The dedicated technician device has `tag:swan-support-operator`, and no ordinary/personal device has that tag.
- The server public-key fingerprint matches the intended `id_ed25519.pub`.
- The express page refuses to install until a numeric ID is available and the unique credential receipt has been copied.
- The installed customer home page hides the ID/password and shows **Ready for unattended support**.
- The unattended device accepts its unique password after reboot and rejects another device's password.
- Unattended rollback restores click approval and window-open behavior.
- On Pro, an authorized Swan technician with MFA can connect and an unauthorized test account cannot.
- File transfer, clipboard, UAC elevation, multi-monitor behavior, reconnect, and uninstall are tested according to the customer's support agreement.
- Every Swan executable/DLL and the final installer have a valid trusted Authenticode signature and timestamp.
- The final signed SHA-256 hash, exact corresponding-source tag, version, privacy link, test evidence, and rollback instructions are published.
- No private server key, signing key, passwords, tokens, or customer data are present in Git history, Actions logs, or build artifacts.

## Updating the RustDesk base

This branch is pinned to `1.4.9`. Treat an upstream update as a controlled security release:

1. Read the upstream release notes and security changes.
2. Create a new update branch from the intended tag.
3. Reapply the small Swan diff and resolve it explicitly.
4. Rebuild from clean inputs.
5. Repeat the complete acceptance checklist.
6. Keep old signed installers and hashes available for rollback until the new build is accepted.

## Licence and source availability

RustDesk is distributed under AGPL-3.0. Swan Computing must retain notices and provide the complete corresponding source for the exact client binary it distributes, including Swan modifications and build instructions. The selected model is an immutable public source tag for every released binary; server private keys and customer credentials are never corresponding source. See [Source availability and licensing](SOURCE_AND_LICENSE.md). This guide is operational guidance, not legal advice; obtain qualified advice if the distribution model or proprietary additions are uncertain.
