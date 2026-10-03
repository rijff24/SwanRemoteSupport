# Swan Remote Support code-signing policy

Production publication requires provider approval, valid signatures and completed release checks. Project signing through an open-source program is an intended integration, not an approved service or a guarantee of free signing. Company signing is optional and requires its own publisher policy and keys.

## Project releases

Use immutable public source tags, reviewed pinned build workflows/toolchains/dependencies and exact artifact/source hashes. Sign the project executables and DLLs before packaging, then sign and verify the final installer. Publish signed release metadata identifying edition, x64 architecture, supported Windows families, package/installed/agent identities, certificate fingerprint, publisher and corresponding source.

Keep project signing private keys with the approved signing provider. Never distribute them to company servers or workers. Company workers consume pinned unchanged project release binaries plus validated company profiles and assets. Profile text must never become a build script or shell command. Customized embedded EXE resources or publisher identity require newly signed company artifacts and corresponding public source.

Verify Authenticode trust, publisher, signing-certificate SHA-256 and signed artifact hashes before executing installation or updates. Company administrators approve imported release metadata before rollout. Signing does not substitute for enrollment, receiver authorization or acceptance testing.

## Release gate

The maintainer must match source commit, workflow run, artifact hashes, publisher policy and recorded test evidence. Complete clean-machine installation, repair/upgrade/recovery/uninstall and native authorization/session tests for every advertised Windows family. Preserve identity, enrollment, consent, branding and permissions. Publish source and known limitations with each release.

Until these gates pass, mark build artifacts as development/unsigned and do not publish them as customer-ready production software. Local test certificates and development release keys stay in isolated protected test environments. A test-signed package is not a trusted production release.

The original single-company signing and Tailscale deployment instructions remain in the tagged baseline. Tailscale is not a required component of new company packages. See [the acceptance report](ACCEPTANCE.md) for remaining signing, packaging and native test work.
