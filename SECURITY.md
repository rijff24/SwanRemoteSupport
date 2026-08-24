# Security Policy

## Supported version

Only the latest published Swan Remote Support release is supported. Until the first signed public release passes the documented acceptance gates, this repository contains development software and no build should be presented as customer-ready.

## Reporting a vulnerability

Use this repository's **Security → Report a vulnerability** private advisory flow. Do not open a public issue containing an exploit, customer identity, device ID/password, Tailscale credential, server secret, signing material, private log, screenshot, or backup content. If private vulnerability reporting is not yet enabled, contact the repository owner privately through the verified Swan Computing business channel and do not send secrets through ordinary email or public chat.

Include the affected version/commit, Windows version, reproducible steps, impact, and any non-sensitive logs or hashes. Remove customer data before attaching evidence. Swan Computing will acknowledge the report, assess severity, coordinate a fix and release, and credit the reporter if requested and safe.

## Deployment security boundaries

- The server address and Ed25519 public key embedded in the client are public values.
- The RustDesk server private key, customer unattended passwords, Tailscale enrollment credentials, and signing keys are secrets and must never enter source control.
- Tailscale tag policy limits network reachability but does not replace the unique Swan password or customer authorization.
- RustDesk Server OSS does not provide a Swan technician identity/MFA authorization layer. Anyone who obtains a device ID and its password can attempt access from an allowed network device.
- Never ask a customer to disable antivirus, SmartScreen, Windows Firewall, or MFA to install this software.

See [Code-signing policy](docs/CODE_SIGNING_POLICY.md), [Privacy](docs/PRIVACY.md), and [Tailscale deployment](docs/TAILSCALE_DEPLOYMENT.md).
