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

The portable packer sorts payload entries and pins its Brotli encoder version.
Its existing `timestamp` metadata field now carries a nonzero content-derived
extraction-cache identity within the TOML signed-integer range. Identical packed
bytes retain the same cache identity; changed payload bytes or startup executable
selection refresh it even when the software version is unchanged. The metadata
format stays compatible with the current launcher. This cache marker is not an
authorization credential or a replacement for package signatures and hashes.

Managed native builds require `SOURCE_DATE_EPOCH` and render their application
build date in UTC from that recorded timestamp. The Windows workflow derives it
from the exact public source commit; the application source capsule records it
as `source_date_epoch` in `BUILD-SOURCE.json`. For a local managed build, set
`$env:SOURCE_DATE_EPOCH = (git show -s --format=%ct HEAD).Trim()` before invoking
the build. When rebuilding an exported source capsule without Git, use its
recorded epoch. Missing, invalid or unsupported timestamps fail the managed
build instead of silently substituting local wall-clock time. The ordinary
upstream build retains its existing metadata path.

The topmost-window helper is built from public upstream commit
`ecd8d6a139eee76845ea66423fb739af450fda90`. Its x64 release workflow requests
`/Brepro` for compilation and linking, disables incremental linking and records
a PDB basename. These linker environment settings are documented by
[Microsoft](https://learn.microsoft.com/en-us/cpp/build/reference/linking?view=msvc-170).
The PE verifier requires the documented
[REPRO debug marker](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
and rejects an embedded absolute PDB path. These checks establish build metadata
properties; an independent binary comparison is still required. Its original
source, public workflow and verification scripts are retained in a separate
source artifact and carried alongside native build outputs, outside the
installed endpoint payload.

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
commits, the dependency file hash, the three-file engine patch and original/patched file
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

The reconstruction selects MSVC tools `14.38.33130`, their matching x64 ATL
component (including `atls.lib`), and Windows SDK
`10.0.22621.0`. A hash-checked patch to the engine's pinned Flutter buildroot
`e265c359126b24351f534080fb22edaa159f2215` passes `-vcvars_ver=14.38` to
Microsoft's environment setup. Before compilation and again before packaging,
the verifier rejects generated commands that reference another Microsoft toolset
or SDK, or disable compiler/STL compatibility checks. The original buildroot
source, applied patch and actual compile/link commands accompany the result.
The release compiler patch also requests content-based COFF/PE timestamps
(`/Brepro`) and a relative debug compilation directory; the linker uses a stable
PDB basename and explicitly disables incremental linking. The final linker
response file is retained and checked separately, because the Ninja command
listing alone refers to that file without exposing its flags. These controls
follow [LLVM's deterministic-build guidance](https://blog.llvm.org/2019/11/deterministic-builds-with-clang-and-lld.html).
They are necessary inputs to the independent comparison, not proof that all
remaining sources of binary variation have been removed.
The two timestamp-controlled builds produced identical DLL/import-library bytes
but different PDBs, despite identical original engine/buildroot source bytes,
recorded toolchain hashes and generated commands. The next reconstruction
experiment replaces the Microsoft linker with `lld-link.exe` from the same
pinned engine DEPS compiler package and requests one linker integration thread.
Its public buildroot patch records all three changed source files, and receipts
record the linker's version and hash. Generated-command checks reject a different
linker, and final-response checks reject absent or conflicting thread controls.
Ninja does not require a Microsoft-specific intermediate `.exp` output for LLD.
The engine's Windows archive definition also omits that intermediate file;
both generated outputs and archive inputs must agree for GN generation.
This is an investigation of debug-symbol reproducibility, not a demonstrated
fix. New independent output comparisons and matching-bundle runtime tests are
required; the application workflow still defaults to its previous engine.
The corrected recipe at `05d5369` subsequently completed in
[run 39753153773](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39753153773).
Its archive, all 19 evidence files, original public engine/buildroot trees and
generated linker commands were verified. An isolated standard-user experiment
with the matching `e6b87b9` application loaded that engine, rendered the branded
technician login and exited cleanly on Server 2016. Authentication and sessions
were not exercised in that experiment. An independent rebuild is still needed
to compare DLL, import library and PDB bytes; no default-engine promotion follows
from the compile or startup results.
The disposable GitHub runner installs the selected side-by-side Microsoft C++
component when needed; this provisioning script refuses execution outside the
GitHub Windows runner. Local builds require that component to be installed
beforehand. The ATL library is required at preflight and its hash is recorded
alongside the compiler/standard-library hashes. Toolchain diagnostics are recorded before
compilation so a failed build still exposes its actual inputs.

For an equivalent local build, run this in a new PowerShell process on a Windows
machine with Visual Studio 2022, the selected MSVC/SDK versions, Git, Python,
and at least 40 GiB free on the chosen build drive. The default Visual Studio path
matches the GitHub runner's Enterprise installation:

```powershell
pwsh -NoProfile -File deployment/windows/Build-FlutterEngine.ps1 -BuildRoot C:\SwanEngineSourceBuild
```

For another Visual Studio edition, supply its installation directory through
`-VisualStudioPath`, for example
`-VisualStudioPath 'C:\Program Files\Microsoft Visual Studio\2022\Community'`.

The build directory must not already exist. Downloads and compilation take place
inside it. The recipe never removes existing toolchains to obtain disk space.
The output is a source reconstruction candidate, not an automatically accepted
replacement or production release. Two independent build comparisons, a complete
toolchain/input review and native Windows runtime qualification are still required
before promoting a new engine hash to the default production selection. Build receipts
retain explicit false values for those unproven release gates.

The native Windows workflow's optional `source-candidate-9f97ef0` selection
is a lab experiment, with the current upstream selection still the default.
It selects the exact reviewed source-built DLL, import library and symbols,
preserves Flutter 3.24.5's matching snapshot compiler, and records its hash.
The original source-engine artifact and selection receipt accompany the outputs.
An older app bundle failed before GUI startup when only its engine DLL was
replaced: its compiled Dart snapshot format differed from the source runtime.
Rebuild the app bundle with the matching compiler; a successful engine build
alone does not establish compatibility or authorize production publication.

The additional explicit lab choice `source-candidate-05d5369` uses the LLVM-linked
engine from run `39753153773`. Its DLL, import library and PDB matched original
bytes from independent run `39761977920`; the public
[comparison record](../deployment/windows/engine-05d5369-reproducibility.json)
contains both artifact digests, output hashes and source-verification counts.
The selector pins the complete first output archive and every binary, rejects
unknown selections, and preserves the SDK snapshot compiler. Its receipt reports
the independent rebuild separately from runtime and production approval, which
remain false. The upstream workflow choice remains the default. A full matching
application build and remote-session/installer/update qualification are still
required for this candidate.

### Company component comparisons

Two independent Ubuntu 24.04 CI jobs for the component inputs reviewed at
`9f97ef0` produced identical management, agent and worker binaries. Their full
component input archives were checked against the public source, and all 130
included source files matched between each platform's pair of builds. The
Windows pair differed in linker timestamps and debug identifiers. After adding
`/Brepro` to the x64 Rust configuration, both Windows and Linux component pairs
at reviewed source `a413ab5` produced identical unsigned executable bytes.
That result covers the recorded CI platforms and input/toolchain versions;
broader toolchain qualification and native application, engine and installer
comparisons remain open.
See the dated [acceptance evidence](ACCEPTANCE.md#independent-company-component-binary-comparisons-22-october)
for exact jobs, artifact hashes and limits. Neither normalized diagnostic hashes
nor unsigned component comparisons qualify complete signed installer releases.

### Native release comparison workflow

The manual **Compare independent native build artifacts** workflow accepts two
different successful native run IDs and their exact shared public source commit.
It checks out that source and pinned submodules, verifies both original archive
digests, validates every archived tracked source file against Git objects, and
checks generated bridge equality. Source text checkout conversions are reported;
shipping files are never normalized. Both builds must contain the complete set
of eight expected executable/MSI outputs. A missing output, invalid digest,
source mismatch or differing shipping binary fails the job. When binary
comparison completes, the report is retained even when outputs differ.

The verifier never installs or executes downloaded artifacts. It uses the
workflow's read-only repository token, does not forward it to storage, and keeps
signed download URLs out of diagnostic errors. Its output records independent
build verification separately from runtime qualification and production
approval, which remain false. Passing this check does not replace signing,
installer, update, session or Windows compatibility acceptance.

### First native comparison result (22 October 2026)

[Comparison run 39789394627](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39789394627)
verified the original archives and public source for native runs `39760022829`
and `39769432105` (attempt 2), both at
`49ced6854f3cbaab84d95807e9bf342e11588f0f`. Its
[retained report](../deployment/windows/native-49ced68-rebuild-comparison.json)
shows byte-identical customer MSI, agent, management service and worker outputs.
The portable technician EXE, its identical installer-name copy, technician MSI
and server setup EXE differ. The comparison correctly failed; complete native
reproducibility is **not established**. No shipping bytes were normalized.
The newer pinned Roslyn setup compiler and source-built engine are separate
changes requiring their own exact-source comparison and runtime qualification.

### Source-built engine application comparison (22 October 2026)

[Comparison run 39800887785](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39800887785)
verified public source and original archives for successful native builds
`39777271372` and `39789147510`, both at
`9e4ea6dfce7dc1e66eccf6e1068780a23ae8fd59` with the explicitly selected,
independently reproduced `05d5369` engine. The
[original report](../deployment/windows/native-9e4ea6d-rebuild-comparison.json)
shows identical customer MSI, agent, management service, worker and server setup
EXE bytes. This establishes the pinned Roslyn server launcher correction across
two actual native CI builds. The portable technician EXE, its installer-name
copy and the technician MSI still differ. The comparison failed as intended;
complete native reproducibility and runtime/release qualification remain open.
No shipping files were normalized or executed by the comparison.

Local analysis of the preserved `49ced68` originals found identical bytes in all
95 extracted application files and identical portable executable code sections.
The portable version-resource string entries have different iteration orders;
their resulting resource bytes and linker content hashes differ. The pinned
`winres` 0.1.12 generator iterates unordered maps. The portable build now orders
contiguous generated version/value directives before passing its RC source to
the resource compiler, using `winres`'s existing-resource API. Resource blocks,
values, icons, manifests and unsigned shipping inputs are preserved. Standalone
ordering regressions run on Linux and Windows CI and in the native packaging
step. The independent full-build comparison below verifies the correction for
the exact source and selected engine recorded there.

### Complete unsigned native comparison (22 October 2026)

[Comparison run 39829786141](https://github.com/rijff24/SwanRemoteSupport/actions/runs/39829786141)
passed for independent successful native runs `39807079997` and `39819129959`,
both at `e9d89ca97319590905a39663b76718ee53bcdd78` with the explicitly selected
source-built `05d5369` engine. The
[original report](../deployment/windows/native-e9d89ca-rebuild-comparison.json)
records **all eight shipping outputs byte-identical**: customer MSI, technician
EXE, technician MSI, installer-name EXE copy, server setup EXE, agent, management
service and worker. No shipping bytes were normalized or executed. Each archive's
1,155 tracked public source files and six generated bridge files were verified.
Archive compression differs; the original shipping outputs match exactly.

The retained comparison artifact is `12448516219`, 17,424 bytes, SHA-256
`611f3d46f88b9ce202d9679a6018662ca8d3496875d12b281895fbefaf9eefaa`.
This establishes unsigned native reproducibility for those inputs, including the
portable resource-order correction. It does not qualify other source revisions,
the default upstream engine, Windows behavior, updates or production signatures.
The separate 1.5.1 build and runtime/release gates require their own evidence.

### Server launcher compiler

The server setup recipe uses MIT-licensed Microsoft Roslyn
`Microsoft.Net.Compilers.Toolset.Framework` 4.12.0, with the NuGet archive and
compiler executable hashes recorded in `deployment/windows/server-compiler.json`.
The package declares corresponding source at
[`dotnet/roslyn` commit dfa7fc6](https://github.com/dotnet/roslyn/commit/dfa7fc6bdea31a858a402168384192b633c811fa).
The compiler runs only during building and is not embedded in the server setup
executable. Its Microsoft Authenticode signature is checked before execution.
The build uses explicit .NET Framework/PowerShell reference assemblies from
hash-pinned Microsoft SDK packages, rather than the host's runtime assemblies.
`deployment/windows/server-references.json` records the packages, exact versions,
archive hashes and five selected assembly hashes. The suppliers publish
[`Microsoft.NETFramework.ReferenceAssemblies.net462` 1.0.3](https://www.nuget.org/packages/Microsoft.NETFramework.ReferenceAssemblies.net462/1.0.3)
and [`Microsoft.PowerShell.5.ReferenceAssemblies` 1.1.0](https://www.nuget.org/packages/Microsoft.PowerShell.5.ReferenceAssemblies/1.1.0).
These are build-only references; package install scripts never run, and the
reference DLLs are not embedded in the application. Matching local builds still
do not establish independent cross-image reproducibility or runtime compatibility.
