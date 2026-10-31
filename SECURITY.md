# Security Policy

## Supported version

Only the latest published Swan Remote Support release is supported. Until the first signed public release passes the documented acceptance gates, this repository contains development software and no build should be presented as customer-ready.

## Reporting a vulnerability

Use this repository's **Security → Report a vulnerability** private advisory flow. Do not open a public issue containing an exploit, customer identity, device ID/password, Tailscale credential, server secret, signing material, private log, screenshot, or backup content. If private vulnerability reporting is not yet enabled, contact the repository owner privately through the verified Swan Computing business channel and do not send secrets through ordinary email or public chat.

Include the affected version/commit, Windows version, reproducible steps, impact, and any non-sensitive logs or hashes. Remove customer data before attaching evidence. Swan Computing will acknowledge the report, assess severity, coordinate a fix and release, and credit the reporter if requested and safe.

## Deployment security boundaries

- Company HTTPS addresses and public trust keys are public bootstrap information. Private transport/configuration keys, worker credentials, account secrets, device identities, signing keys and backups must stay outside source control and public downloads.
- Managed editions require company enrollment, technician authorization and short-lived device-bound grants enforced by the receiving device. UI restrictions or network reachability alone are insufficient. See the acceptance record for verified behavior and outstanding gates.
- Each company hosts its own infrastructure. The preserved single-company baseline has different historical password and Tailscale boundaries; do not treat its network restrictions as managed authorization.
- Earlier development Windows worker installers stored credentials in service registry environment settings. Server 2016 testing found additional app-container read permissions. Revoke affected worker registrations, stop legacy workers and remove those registry secrets before replacement. Current development source requires protected private-file configuration for service mode; Server 2016 lab migration, standard-user read denial, package generation and reboot checks passed. The complete Windows matrix and production release qualification remain required.
- Before restoring a backup, reconcile revoked worker registrations: an old database can restore their credential hashes. Do not expose the restored deployment until legacy registrations have been revoked again.
- Never ask a customer to disable antivirus, SmartScreen, Windows Firewall, or MFA to install this software.

See [Code-signing policy](docs/CODE_SIGNING_POLICY.md), [Privacy](docs/PRIVACY.md), and [Tailscale deployment](docs/TAILSCALE_DEPLOYMENT.md).
