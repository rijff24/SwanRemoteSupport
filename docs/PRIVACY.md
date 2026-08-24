# Swan Remote Support privacy statement

## Purpose

Swan Remote Support is used only to provide support, maintenance, updates, and separately agreed backup work for Swan Computing customers. Installation and unattended access require the computer owner's informed authorization. The customer can see the tray/service presence and can stop or uninstall the software.

## Data handled

Depending on the support task, the service may process:

- customer and device identification needed to associate a computer with its support agreement;
- the Swan Remote Support device ID and its unique unattended password;
- connection metadata and diagnostic logs from the self-hosted RustDesk server and client;
- device and account metadata processed by Tailscale under Tailscale's own service terms and privacy documentation;
- screen content, clipboard content, files, commands, or system information visible during an authorized support session; and
- backup data only when a separate backup scope, destination, encryption, retention, and restore procedure has been agreed.

The public source repository contains the RustDesk/Tailscale server address and the server's Ed25519 **public** key. These are routing and trust values, not customer records or private credentials. The repository and release artifacts must never contain the server private key, customer passwords, Tailscale auth keys, API tokens, signing keys, or customer data.

## Storage and access

Each customer computer receives a different cryptographically random Swan password. The one-time device ID/password receipt is copied during installation, transferred into Swan's organization password manager, and cleared from the clipboard automatically after 90 seconds when the GUI flow is used. The installer does not print the password unless an administrator explicitly chooses the higher-risk `-ShowCredentials` option.

Access to customer device records is limited to authorized Swan support staff. Technician devices and service accounts must use MFA where supported. Passwords must not be reused or sent through ordinary email or unencrypted chat.

## Retention and deletion

Support credentials and operational records are retained only for the active support relationship and any legally or contractually required period. When support ends, Swan disables unattended access, removes the Tailscale node from the tailnet, deletes or archives the password-manager record according to the agreed retention policy, and helps the customer uninstall the software if requested.

Backup retention is governed by the separate customer backup agreement. Ending remote support does not itself prove that every backup copy has been deleted; backup destinations and retention schedules must be reviewed separately.

## Customer choices

Customers may request attended support instead of unattended access, pause the Swan service, revoke authorization, uninstall Swan Remote Support, or ask what support records are held about their device. A customer-request process and business contact channel must be published on Swan Computing's customer download page before the first public release.

This repository documents the technical privacy controls. It is not a substitute for a jurisdiction-specific privacy notice or customer contract. Swan Computing should obtain appropriate South African legal/privacy review before production use, especially where customer backups may contain personal information.
