# Source availability and licensing

Swan Remote Support retains RustDesk attribution and AGPL-3.0 licensing. Publish complete corresponding source for every distributed release, including endpoint, management, protocol and worker changes, build scripts, pinned toolchains/dependencies and license notices. Release metadata must identify the exact immutable public source tag. Keep the source revisions for packaged rendezvous/relay components public in the transport manifest.

Each company hosts its own deployment. Company names, logos, colors and public bootstrap/profile trust information customize the product; they do not grant technician access. The configurable product uses enrollment, group authorization, signed session grants and receiving-device enforcement. New packages do not require Tailscale or collect reusable legacy support passwords.

Never publish company private transport/configuration keys, device credentials, technician/admin tokens, TOTP seeds, signing private keys/PFX passwords, backup decryption material, customer data or private operational logs. Installers carry public bootstrap information only. Machine-specific test/deployment settings and all test credentials belong in ignored local environment/data files.

Project-signed binaries retain the certificate publisher's embedded identity. Company branding assets do not change that identity. Changing embedded resources requires new signed binaries and the corresponding public source/build description. Never supply a project signing private key to company servers or workers.

The original single-company Tailscale deployment remains available at `swan-single-company-baseline-1.4.9`; its historical build/deployment documents apply to that baseline only.

Before production publication, build from the reviewed public source, verify artifacts and their signatures/hashes, complete the acceptance checks and publish exact corresponding source. Preserve the repository's `LICENCE` and upstream notices.

The Windows workflow also produces `SwanRemoteSupport-build-source.zip` before
application compilation. It contains the actual tracked source bytes, initialized
pinned submodule source, and the six explicitly named generated bridge files.
`BUILD-SOURCE.json` records repository revisions and every included file hash;
the public workflow in the archive describes subsequent packaging transformations,
including removal of the portable launcher's DPI manifest entry. Ignored local
settings, untracked logs, credentials and build binaries are not swept into this
archive. Missing bridge inputs, source links, unmerged indexes and submodule pin
mismatches fail the build. The artifact checksum list includes the archive hash.
Publish this archive alongside the exact source tag, build instructions and
third-party component source references. Its presence does not prove binary
reproducibility or replace release acceptance and signing checks.
