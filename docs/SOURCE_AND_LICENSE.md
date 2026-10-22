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

## Patched Windows Flutter engine provenance

The managed Windows workflow currently downloads the RustDesk engine release
asset `windows-x64-release.zip` and requires SHA-256
`ec8cabf36ee4ff24c8d98de25b00e70781eb03876265aee84d0fe554a110036e`.
This verifies the reviewed binary input. It does not establish which source and
toolchain produced that input or prove that rebuilding it yields identical bytes.

The public [upstream build recipe](https://github.com/rustdesk/engine/blob/bc374ca81eae3f77329ebf7bc98efadfde10018e/.github/workflows/flutter-engine-windows-x64-release-build.yml)
checks out engine tag `3.24.4`, changes `ClearCurrent` to `MakeCurrent` in
`shell/platform/windows/flutter_windows_engine.cc`, adds `<string>` before
`<string_view>` in `fml/hex_codec.h`, synchronizes dependencies, and builds
`host_release`. The tag currently resolves to Flutter engine source
[`db49896cf25ceabc44096d5f088d86414e05a7aa`](https://github.com/flutter-team-archive/engine/tree/db49896cf25ceabc44096d5f088d86414e05a7aa).
Keep these source and patch references with the release's third-party notices.
They identify a public reconstruction candidate; they are not an attestation
that the pinned binary was built by this recipe.

On 22 October 2026 the upstream API identified release `188171540`, tag commit
`bc374ca81eae3f77329ebf7bc98efadfde10018e`, and asset `210168939`
(63,301,215 bytes, last updated 1 December 2024). Its digest field was null, and
the workflow-run query for the asset creation date returned no runs. The recipe
uses a floating Windows runner, depot-tools setup action and release overwrite.
Do not infer immutable build provenance from the mutable release tag or matching
archive hash alone. Full product source/reproducibility acceptance remains open
until the engine's build inputs, patches, toolchain/dependencies and rebuilt
artifacts are verified and published with the project release.

### Independent engine reconstruction

Run **Rebuild patched Windows Flutter engine from pinned source** in Actions to
build a separate candidate. The reviewed recipe is
`deployment/windows/flutter-engine.lock.json`: it pins the engine and depot-tools
commits, the dependency file hash, the two-file patch and original/patched file
hashes. The build rejects a different revision, changed dependency/source bytes
or additional tracked modifications before patching. It uses the engine's pinned
Ninja and Clang inputs, records actual dependency revisions and the available
Windows toolchain versions, and packages the DLL/import library/PDB with the
original public engine source, applied patch, generated GN arguments and build
recipe. It does not import certificates or change the application engine pin.
The old Flutter tinygltf mirror returned 404 for its pinned source commit during
reconstruction. The lock routes only that dependency to its declared canonical
upstream repository and retains the exact original DEPS commit
`9bb5806df4055ac973b970ba5b3e27ce27d98148`. The final package check verifies
both the checked-out commit and repository URL; the original engine DEPS file
remains unchanged. The effective gclient configuration is included as evidence.

For an equivalent local build, run this in a new PowerShell process on a Windows
machine with Visual Studio 2022 Enterprise, its C++ tools/Windows SDK, Git, Python,
and at least 40 GiB free on the chosen build drive:

```powershell
pwsh -NoProfile -File deployment/windows/Build-FlutterEngine.ps1 -BuildRoot C:\SwanEngineSourceBuild
```

The build directory must not already exist. Downloads and compilation take place
inside it. The recipe never removes existing toolchains to obtain disk space.
The output is a source reconstruction candidate, not an automatically accepted
replacement or production release. Two independent build comparisons, a complete
toolchain/input review and native Windows runtime qualification are still required
before selecting a new engine hash for the application workflow. Build receipts
retain explicit false values for those unproven release gates.
