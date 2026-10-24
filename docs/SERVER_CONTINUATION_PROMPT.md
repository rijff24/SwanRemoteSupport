# Server continuation prompt

> Historical continuation prompt for the single-company RustDesk/Tailscale baseline. Use it only when maintaining that preserved deployment. Current company-hosted product work follows the [product guide](../product/README.md) and [acceptance report](ACCEPTANCE.md).

Paste the following into Codex when working on the RustDesk/Tailscale server. It is intentionally evidence-first because live Tailnet, firewall, and key changes can interrupt support access.

```text
Continue the Swan Remote Support project from the public repository:
https://github.com/rijff24/SwanRemoteSupport

Goal: validate and safely prepare the live RustDesk OSS server and Tailscale network for the Swan customer client. The intended server Tailscale IP is 100.82.236.84. Native RustDesk clients require TCP 21115-21117 and UDP 21116. The public Ed25519 key expected by the client is I0s1JvqJ19WDNKBLFY+FC5HekcOqNWPpH+6xGmKUQLI=. Never print or publish the private id_ed25519 key.

First clone or update the repository in a normal development directory, inspect the exact checkout, read AGENTS.md completely, and read these files before changing anything:
- docs/SWAN_CUSTOM_BUILD.md
- docs/TAILSCALE_DEPLOYMENT.md
- config/tailscale-swan-policy.example.json
- docs/WINDOWS_CODE_SIGNING.md
- docs/CODE_SIGNING_POLICY.md
- docs/PRIVACY.md
- docs/SOURCE_AND_LICENSE.md

Work in this order:

1. Record the server OS/version, repository path/branch/commit, current git status, Tailscale version/status, and whether the server has tag:swan-rustdesk-server. Do not dump peer lists, auth material, customer data, or private keys into chat/logs.
2. Discover how hbbs and hbbr actually run (Docker Compose, containers, systemd, or another supervisor). Record service names, image/binary versions, restart policy, config paths, bind addresses, mounted data directories, and health. Preserve unrelated configuration.
3. Locate the RustDesk server data directory without displaying private-key contents. Check owner/group/mode on id_ed25519 and id_ed25519.pub. Independently derive/read only the public key and confirm it exactly matches the client value above. If it differs, stop and report; do not rotate or replace either key.
4. Make a dated, access-restricted backup of the RustDesk configuration/data and the current service definition before any mutation. Report the backup path, permissions, contents at a high level, and a restore command. Do not put the backup inside the public Git repository.
5. Inspect listening sockets and firewall rules. Confirm hbbs/hbbr are reachable on the Tailscale interface for TCP 21115, 21116, 21117 and UDP 21116. Do not expose these ports to the public internet unless the existing approved architecture explicitly requires it. Report the exact current bind/firewall state before proposing changes.
6. Inspect the live Tailnet policy and export/back up its exact current form through an authenticated admin session. The laptop audit saw only two visible devices, both untagged: this technician laptop and the server at 100.82.236.84. Re-check rather than assuming that remains true.
7. Compare the live policy with config/tailscale-swan-policy.example.json. Explain any existing rules that would be removed or broadened. Tailscale grants are additive, so an old allow-all rule would defeat the example restrictions. Do not replace the live policy, assign tags, or risk administrator lockout until you show me the backup, diff, rollback method, and ask for approval.
8. After approval, assign tag:swan-rustdesk-server to 100.82.236.84 and tag:swan-support-operator only to the dedicated technician laptop. Apply the reviewed policy. Never assign the operator tag to customer or general personal devices. Preserve an authenticated admin recovery session while applying it.
9. Validate negative and positive network behavior: customer-tagged test device to required server ports succeeds; customer to customer/unrelated services fails; operator to server succeeds; operator to customer supports RustDesk; RustDesk ID allocation succeeds; and a forced/observed relay session succeeds. UDP 21116 must be tested operationally, not inferred from a TCP probe.
10. Inspect RustDesk logs for service errors and public rendezvous fallback without exposing device IDs, passwords, IP inventories, or customer data. Confirm hbbs and hbbr restart cleanly only if a restart was approved and necessary.
11. Confirm server backups, log rotation, OS/container updates, Tailscale updates, firewall persistence, monitoring, and restore testing. Weekly customer-data backups are a separate service: do not configure them until the customer-specific scope, source, destination, encryption, retention, and restore-test requirements are supplied.
12. Re-run checks after any approved change. Update relevant repository documentation and the ignored docs/AI_CHANGELOG.md. Do not commit secrets or live config exports. Check git status and explain every changed file. Commit and push only source/documentation changes that belong in the public project; keep server backups and production configuration private.

Important gates:
- Do not rotate the RustDesk keypair.
- Do not create or embed a reusable Tailscale auth key.
- Do not disable antivirus, firewall, MFA, or TLS verification.
- Do not expose the RustDesk ports publicly just to make a test pass.
- Do not claim the client is customer-ready: the GitHub Windows build, clean-VM UI/runtime tests, public release, SignPath acceptance, two-stage signing, and Defender checks are still pending.
- Tailscale Personal is not intended for commercial use. Confirm the live account is on an appropriate business plan before customer deployment.

At the first checkpoint, return a concise evidence table with: item, observed state, pass/fail, proposed change, risk, backup/rollback. Then ask for approval only for the exact live changes that remain.
```
