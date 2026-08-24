# Swan Remote Support code-signing policy

## Scope

This policy covers Swan-authored Windows executables, DLLs, PowerShell deployment helpers, and the final Swan Remote Support installer distributed from this repository. The official Tailscale MSI is not part of the Swan build or release payload; the bootstrap downloads it from Tailscale and independently verifies its checksum and publisher.

Free code signing provided by SignPath.io, certificate by SignPath Foundation.

SignPath Foundation eligibility and project approval are external gates. Until the project is accepted and the complete release passes this policy, all CI artifacts must remain clearly marked **unsigned** and must not be distributed as customer-ready software.

## Source and responsibilities

- Canonical source: the public Swan-controlled GitHub fork and the immutable Git tag for the release.
- Author/maintainer: Rijff Swanepoel (`@rijff24`).
- Reviewer: the maintainer records a self-review while Swan is a one-person project. Security-sensitive or dependency changes should receive an independent review before signing whenever another qualified reviewer is available.
- Signing approver: Rijff Swanepoel, using a separate MFA-protected SignPath approval step.
- Build system: the pinned GitHub Actions workflow in `.github/workflows/swan-windows-build.yml`.

The author and signing-approval actions must be separate auditable events even when performed by the same person. GitHub and SignPath accounts must use MFA. Shared accounts are prohibited.

## Approval requirements

A signing request may be approved only when all of the following are true:

1. The source commit is in the canonical public repository and the release tag is immutable.
2. The release contains no private server key, Tailscale auth key, API token, customer credential, signing key, PFX, password, or customer data.
3. The build uses the reviewed workflow and pinned action commits, toolchains, and dependency lockfiles.
4. The server address and Ed25519 **public** key match the documented public configuration.
5. Automated format, parse, compile, and applicable test checks pass from a clean checkout.
6. The inner Swan executables and DLLs are scanned, signed, timestamped, and signature-verified before packaging.
7. The final installer is built from those signed inner files, then signed, timestamped, scanned, and signature-verified.
8. A clean Windows 10/11 test confirms installation, Tailscale tag enforcement, numeric Swan ID registration, unattended connection, customer-visible stop/uninstall controls, reboot/logon-screen access, rollback, and uninstall.
9. The release notes publish the final SHA-256, corresponding-source link, privacy link, known limitations, and rollback instructions.
10. No unexplained files or unreviewed changes remain in the release tree.

An approver must reject a request if the source commit, workflow run, artifact hash, expected publisher, or test evidence cannot be matched exactly.

## Tailscale boundary

Swan does not sign, modify, or represent the Tailscale Windows client as a Swan product. The deployment bootstrap accepts the official Tailscale package only if both the pinned hash and valid `Tailscale Inc.` Authenticode organization match. A mismatch is a hard failure. The temporary MSI is removed after the attempt.

This separation is intentional for licensing, provenance, update safety, and SignPath review. A future monolithic bundle must not be released unless Tailscale redistribution terms and SignPath's proprietary-component rules have been reviewed in writing.

## Key protection and incident response

No signing private key is stored in this repository or on a general-purpose developer PC. Signing is performed by the approved hosted service with MFA and auditable approval. Signing credentials may not be printed, copied to issue comments, or exposed to pull requests from forks.

If a signing account, workflow, release artifact, maintainer device, or customer credential may be compromised:

1. Stop publication and customer installation immediately.
2. Revoke affected repository/session credentials and notify SignPath where signing trust may be affected.
3. Remove or mark the release unsafe without rewriting the source history needed for investigation.
4. Preserve hashes, workflow logs, and relevant security evidence without publishing customer secrets.
5. Rotate affected Swan passwords, Tailscale node identities/tags, and server secrets as applicable.
6. Publish a clear security advisory and a clean replacement release only after root-cause review and full revalidation.

See [Security Policy](../SECURITY.md), [Privacy](PRIVACY.md), [Windows signing process](WINDOWS_CODE_SIGNING.md), and [Tailscale deployment](TAILSCALE_DEPLOYMENT.md).
