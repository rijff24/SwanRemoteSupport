# Source availability and licensing

Swan Remote Support is a modified RustDesk client distributed under the GNU Affero General Public License v3.0. Anyone receiving a Swan binary must be able to obtain the complete corresponding source for that exact version, including Swan modifications, build scripts, pinned build configuration, and licence notices.

The corresponding-source link in every release must point to the immutable public Git tag that produced that binary. A private source copy offered only on request is not the selected release model for this project. The public server address and Ed25519 public key may appear in source; they do not grant access without Tailscale authorization and a device-specific Swan password.

Never publish:

- the RustDesk server's private `id_ed25519` key;
- customer device IDs/passwords or customer data;
- Tailscale auth keys, API/OAuth secrets, or exported session credentials;
- code-signing keys, PFX files, passwords, or recovery material; or
- production logs, backup contents, or support-session recordings.

The official Tailscale Windows MSI is downloaded separately from Tailscale, is not modified or rebranded, and is not part of Swan's AGPL corresponding source. Tailscale's open-source client code is BSD 3-Clause licensed, while its official Windows product and hosted service have additional licensing and service terms. See [Tailscale deployment](TAILSCALE_DEPLOYMENT.md).

Before publishing a release, run a repository-history and artifact secret scan, build from a clean checkout, verify the release hashes and signatures, and confirm the public tag matches the exact source used. Retain upstream RustDesk attribution and the repository's `LICENCE` file.
