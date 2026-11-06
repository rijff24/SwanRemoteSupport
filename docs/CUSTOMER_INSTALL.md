# Company customer installation

The configurable product is still undergoing acceptance testing. See
[the acceptance report](ACCEPTANCE.md) before publishing production downloads.
Each company operates its own server and publishes its customer bundle from its
verified HTTPS website. Customer setup does not require Tailscale or manual
server-address changes.

1. Download the complete customer ZIP from your support company's website.
2. Extract the whole ZIP to a normal local folder. Keep its installer, scripts,
   `bootstrap.json`, signed profile and signed release metadata together.
3. Double-click `Install-SwanRemoteSupport.cmd`. Customer setup requests Windows
   administrator approval; declining it leaves setup unstarted.
4. Check the company's HTTPS hostname shown by setup and type that hostname to
   confirm the company you intended to install.
5. Setup verifies company configuration and the release's package hash and trusted
   publisher, installs the app, and enrolls this device. No technician password or
   reusable privileged enrollment secret is included in the public bundle.
6. Wait for the company to approve the new device. Sessions require an authorized
   technician and your approval by default.
7. Ask the company to demonstrate support status, stop controls and uninstall.
   Unattended support requires separate explicit consent; it is not enabled by
   double-clicking the setup entry point.

Use the complete company bundle for setup. Running the generic EXE/MSI alone
does not supply company configuration or complete managed enrollment. Managed
deployment and explicit repair can use `Install-Company.ps1` with an explicit
`-BootstrapPath`; see [installer repair](INSTALLER_REPAIR.md).

Technician bundles use the same double-click entry point and install for the
current Windows user without requesting elevation. Obtain them through the
company's authenticated internal download. Windows Installer policy still
applies to technician MSI installation; company IT must arrange an approved
deployment when unmanaged per-user packages are blocked. Technician login and
device authorization happen in the installed app.

The app retains product attribution and license information in About. Project
binaries retain their certificate publisher even when the company customizes
their visible branding. Missing or unexpected signatures and failed verification
must be resolved through the support company; setup does not bypass those checks.
See [privacy](PRIVACY.md), [security](../SECURITY.md), and
[source/licensing information](SOURCE_AND_LICENSE.md).

The original Swan-only Tailscale deployment is preserved in the
[tagged baseline](https://github.com/rijff24/SwanRemoteSupport/tree/swan-single-company-baseline-1.4.9),
with its original installation and build instructions.
