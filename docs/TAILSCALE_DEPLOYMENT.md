# Tailscale deployment for Swan Remote Support

> Historical baseline deployment only. The configurable product uses company-hosted rendezvous/relay and signed session grants; new packages must not install or require Tailscale. Keep the existing private deployment available until public direct, relay and denied-access tests pass. The pricing and service information below is historical, not current purchasing guidance.

## Why Tailscale is installed separately

Tailscale is largely open source, but it is not one wholly open-source Windows product. The `tailscaled` daemon and CLI are published under the BSD 3-Clause licence; the official Windows graphical client and Tailscale's hosted coordination service include components that are not part of that open-source repository. See Tailscale's [open-source overview](https://tailscale.com/opensource), the [client source and licence](https://github.com/tailscale/tailscale), and the [service terms](https://tailscale.com/terms).

Swan therefore does **not** rebrand, recompile, or embed the Tailscale Windows MSI. The Swan bootstrap downloads an exact official package from `pkgs.tailscale.com`, verifies both its published SHA-256 and its valid `Tailscale Inc.` Authenticode publisher, installs it unchanged, and deletes the temporary MSI. This keeps the Tailscale package and Swan's AGPL corresponding source clearly separate.

Tailscale's hosted-service plan still applies. As checked on 24 August 2026, Tailscale's [pricing page](https://tailscale.com/pricing) expressly describes the free Personal plan as suitable only for non-commercial use. Swan's paid customer-support use should therefore use a business plan or written authorization from Tailscale. The current Standard price is USD 8 per user per month, with 50 tagged resources included and additional tagged resources listed at USD 1 each per month. A tagged customer computer is a resource rather than a user seat, but confirm the actual device classification and invoice in the live Tailscale Billing page before rollout because plan names, limits, and prices can change.

## Security model

The tailnet is an additional network boundary; it does not replace the unique RustDesk password or customer consent. The deployment uses three roles:

| Tag | Assigned to | Network access |
| --- | --- | --- |
| `tag:swan-customer` | Customer computers | RustDesk server TCP `21115-21117` and UDP `21116` only. No customer-to-customer rule. |
| `tag:swan-support-operator` | Dedicated Swan technician computers | RustDesk server ports and direct TCP/UDP sessions to customer devices. |
| `tag:swan-rustdesk-server` | The host at `100.82.236.84` | Receives the declared RustDesk service traffic. |

Tailscale grants are additive. Leaving an existing allow-all ACL/grant in the policy would defeat these restrictions. The example at [`config/tailscale-swan-policy.example.json`](../config/tailscale-swan-policy.example.json) is a deliberately small complete policy, not a fragment that can safely be pasted alongside broad rules. Review and merge every existing tailnet requirement in the Tailscale policy editor before replacing the current policy.

The operator-to-customer rule permits TCP and UDP because a direct RustDesk peer session can use dynamically selected ports. This is intentionally broader than the customer-to-server rule. Protect every operator computer with full-disk encryption, current Windows patches, MFA on its Tailscale identity, a locked screen, and a separate non-administrator daily account. Do not give the operator tag to ordinary personal devices.

## One-time tailnet preparation

1. Enable MFA on the Tailscale identity provider and all tailnet administrators.
2. In the Tailscale policy editor, replace the default allow-all policy with a reviewed policy based on the example. Preserve unrelated required rules explicitly; grants are deny-by-default only after broad allow rules are removed.
3. Assign `tag:swan-rustdesk-server` to the device whose Tailscale address is `100.82.236.84`.
4. Assign `tag:swan-support-operator` to the dedicated Swan technician computer.
5. Confirm the tag owners are limited to tailnet administrators.
6. Use the policy preview and test tools before saving. Keep another authenticated admin session available so an incorrect policy can be reverted.
7. Confirm the server firewall and RustDesk services expose TCP `21115`, `21116`, and `21117`, plus UDP `21116`, on the Tailscale interface. Web-client ports `21118` and `21119` are not required by this native-client deployment.
8. Verify a customer-tagged test device cannot connect to another customer device or unrelated tailnet services.
9. Verify an operator-tagged test device can connect through Swan Remote Support, including a relayed session.

Do not apply the policy automatically from the customer installer. Tailnet policy replacement is an administrator action that can affect every device and can lock out unrelated services if merged incorrectly.

## Customer express installation

Place the signed Swan installer and the deployment helper files in one extracted folder, then double-click `Install-SwanRemoteSupport.cmd`. This is a one-entry installation flow; Tailscale remains a separately verified product rather than being hidden inside Swan's installer.

The bootstrap performs these checks in order:

1. Requires Windows administrator approval.
2. Requires a valid trusted signature on the Swan installer. The expected release publisher is `SignPath Foundation`; a future Swan commercial certificate can be selected explicitly with `-ExpectedSwanSignerOrganization`.
3. Finds a valid Tailscale installation or downloads pinned Tailscale `1.102.3` for Windows x64.
4. Verifies the Tailscale MSI SHA-256 `03ac8183c6e3ce276e9b44281ebe7e4c02aef28a971034ca170c4b665df42dce` and its `Tailscale Inc.` Authenticode publisher before execution.
5. Opens a manual Tailscale login that requests `tag:swan-customer`. It never embeds or saves an auth key.
6. Stops until the device is online with the exact customer tag.
7. Enables Tailscale unattended operation and automatic updates, disables subnet-route acceptance, Tailscale SSH, exit-node advertisement, shields-up, and the local Tailscale web client.
8. Requires the Swan server's three TCP ports to be reachable. The installed Swan client obtaining a numeric ID is the end-to-end check that also covers registration and the required UDP policy.
9. Installs Swan, starts its Windows service, creates a unique unattended password, and copies the one-time receipt for immediate storage in Swan's password manager.

The `-AllowUnsignedSwanInstaller` and `-SkipTailscaleCheck` switches exist only for isolated development tests. A customer installation must never use either switch.

## Manual login and tags

The login command requests `tag:swan-customer`; it does not contain a reusable enrollment credential. If the tailnet does not authorize that requested tag automatically, leave the installer open, approve/assign the tag in the Tailscale admin console, and allow the bootstrap to recheck it. The default wait is ten minutes.

Do not place Tailscale auth keys, OAuth client secrets, API tokens, reusable pre-authentication keys, or exported tailnet policy credentials in this repository, a GitHub Actions variable used by an untrusted fork, a filename, a command transcript, or a customer installer. If a future unattended enrollment system is required, use a separately protected deployment service that issues single-use, pre-approved, tagged keys just in time and audits every issue. It must not be implemented as a static key in the installer.

## Updating the pinned Tailscale package

Do not change only the version string. For each update:

1. Review Tailscale's release notes and Windows support requirements.
2. Download the intended `amd64.msi` and its adjacent `.sha256` file from the official [stable package index](https://pkgs.tailscale.com/stable/).
3. Verify HTTPS origin, SHA-256, and a valid `Tailscale Inc.` Authenticode signature on an isolated test machine.
4. Update both `TailscaleVersion` and `TailscaleMsiSha256` in `scripts/Install-SwanRemoteSupport.ps1` and this document.
5. Re-run PowerShell parsing, package verification, clean Windows installation, login/tag enforcement, policy-negative tests, Swan connection, reboot, update, rollback, and uninstall tests.

## Removal and customer offboarding

First run the Swan unattended-access disable helper or uninstall Swan Remote Support. Then remove the customer device from the Tailscale admin console so its node identity can no longer join the tailnet. Uninstall Tailscale only after confirming the customer does not use it for another agreed purpose. Remove the Swan password-manager record according to the support agreement and retention policy.

Weekly backups and software maintenance are separate authorized services, not features of Tailscale or RustDesk. Record the customer's consent, scope, schedule, destination, encryption, retention, restore test, and audit trail separately before using unattended access for those jobs.

## Live server validation checkpoint

The Swan server and dedicated technician device were validated against this design on 24 August 2026. At that checkpoint:

- the server had only `tag:swan-rustdesk-server`, and the dedicated technician device had only `tag:swan-support-operator`;
- the default allow-all grant had been removed and replaced by the two grants in the example policy, while the pre-existing self-only Tailscale SSH check rule was retained;
- the Windows server firewall allowed `hbbs` TCP `21115-21116`, `hbbs` UDP `21116`, and `hbbr` TCP `21117` only from the Tailscale adapter and CGNAT address range;
- older application-wide, unrestricted-port, and stale test-binary firewall rules were disabled but retained for rollback;
- the RustDesk server public key matched the documented public key, and the private-key ACL was restricted to Windows SYSTEM and Administrators; and
- `hbbs` and `hbbr` remained running after the policy, tag, firewall, and ACL changes without a service restart.

This checkpoint is not customer acceptance. The technician device was offline and no customer-tagged test device was enrolled, so operational UDP, operator-to-server, customer isolation, numeric ID allocation, forced or observed relay, and unauthorized-device tests remain pending. The signing, clean-VM, Defender, release, privacy, and hosted-service-plan gates elsewhere in this repository also remain in force.
