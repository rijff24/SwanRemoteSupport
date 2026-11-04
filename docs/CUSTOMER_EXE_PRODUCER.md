# Customer EXE release producer

`deployment/windows/Build-CustomerExe.ps1` rebuilds the portable outer installer
after the installed customer executable has been signed. Signing the old outer
installer alone leaves its embedded executable unchanged and does not produce a
trusted customer installation.

This tool is for project release maintainers or a company's explicitly controlled
signing environment. The default company worker continues to consume approved,
unchanged project artifacts and package their public bootstrap/profile assets.
Company profile fields cannot select source code, compiler commands or scripts.
Project signing keys must remain with the project's signing provider.

The input payload may name its signed main executable `rustdesk.exe`. The
producer stages those unchanged bytes as `Swan Remote Support.exe` and uses that
name as the portable entry point. The native Flutter installer copies the
payload directory without renaming its executable; the staged names must
therefore match `installed_files` exactly. Verification must not translate an
embedded `rustdesk.exe` into the expected installed name. A valid signature and
matching content hash alone do not establish a usable installed service or
shortcut.

## Inputs

Use a Windows x64 build environment with Rust/Cargo1.90.0, Microsoft C++ tools and
a Windows SDK. Rust's [1.90.0 release](https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/)
and Microsoft's [Build Tools component reference](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2022)
identify the tools. Initialize the compiler's normal `PATH`, `LIB` and `INCLUDE`
environment before invoking the producer. The Windows SDK must be registered:
the pinned winres MSVC backend selects it through the 32-bit Installed Roots
registry key, rather than searching `PATH`. The producer checks that the actual
selected resource compiler matches the trusted Microsoft tool on `PATH` and
records its hash. Python3.12.10, brotli1.1.0 and pinned
Git2.x are required. Cargo resolves the complete public workspace lockfile,
including Git dependencies, even when compiling the portable member. Provide
network access to those pinned public sources; the producer keeps `--locked`.
Use a protected, dedicated local Cargo cache with an absolute path of at most24
characters, such as `D:\SwanCargo`. Git submodule paths in the full workspace can
otherwise exceed Windows path limits before the portable crate compiles. Existing
cached source objects may be reused; the pinned workspace lockfile still selects
the exact commits and checksummed crate versions.
The producer changes its own process environment and restores it on exit.

Prepare these independently verified inputs:

1. The native build's `SwanRemoteSupport-build-source.zip`, its SHA-256 and exact
   public source revision. Verify its complete corresponding source against Git,
   including pinned submodules and generated bridge files, before production use.
2. The final Flutter payload directory, including a signed `rustdesk.exe` from
   the intended release publisher. Retain the other native libraries and assets.
   Build the final `installed_files` manifest from this directory **after** signing:

   ```text
   python deployment/windows/generate-payload-manifest.py --edition customer --source SIGNED_PAYLOAD --output customer-installed-files.json
   ```

3. The trusted publisher certificate's SHA-256, exact compiler/linker/Git executable
   paths and their SHA-256 pins. Preserve the compiler, SDK, dependency and provider
   identities used for the release. Do not take these pins from a company profile.

Use a new work directory and output path. Existing work and artifacts are retained
instead of overwritten. A representative invocation is:

```powershell
./deployment/windows/Build-CustomerExe.ps1 `
  -SourceArchive $nativeSource -SourceArchiveSha256 $nativeSourceHash `
  -SourceRevision $nativeRevision -SourceDirectory $signedPayload `
  -PayloadManifest $customerManifest -PublisherCertificateSha256 $publisherHash `
  -CargoExe $cargoExe -CargoSha256 $cargoHash `
  -RustcExe $rustcExe -RustcSha256 $rustcHash `
  -GitExe $gitExe -GitSha256 $gitHash `
  -LinkExe $linkExe -LinkSha256 $linkHash -PythonExe $pythonExe `
  -CargoHome $shortCargoCache `
  -WorkDirectory $newBuildDirectory -Output $unsignedInstallExe
```

The output name must end in `-unsigned-install.exe`. The source archive and each
declared source file are checked before staging. Payload links, undeclared files,
hash changes and ambiguous Windows paths are rejected. The staged inner executable
is checked again for the expected trusted publisher. The only resource
transformation removes the `dpiAware` lines, matching the native portable-installer
build workflow; its before/after hashes are recorded.

The pinned portable generator compresses the exact staged payload and rebuilds
the packer with `cargo build --locked --release`. Native packer tests must pass.
An independent parser checks every embedded file against the final manifest
without running the payload. Its result proves matching bytes; Authenticode trust
is established separately by the PowerShell producer and final signing checks.
Windows PowerShell 5.1 accepts normal native stderr progress and checks the actual
exit code; a nonzero exit or missing tool fails the build. Run
`test_customer_exe_native_output.ps1 -PythonExe PATH_TO_PYTHON` to check that boundary.

## Signing and release approval

The producer leaves the outer EXE unsigned. Sign a new copy through the approved
provider or [company signing integration](../deployment/windows/COMPANY-SIGNING.md),
then verify its signature, trusted publisher, timestamp and final SHA-256. Run the
embedded-payload verifier again on that signed copy. Authenticode signing must not
change the embedded payload:

```text
python deployment/windows/verify-portable-payload.py --package SIGNED_INSTALL_EXE --manifest customer-installed-files.json --output signed-exe-payload-verification.json
```

Use this EXE producer's final installed-file manifest as the signed release's
`installed_files`. Run the command above against the exact list that will be
signed in that release. The MSI recipe has a different installed-file list,
including `LICENSE.txt`; reusing that list for an EXE can pass package signature
and hash checks but fail installed-file verification. Preserve a failed receipt,
withdraw that release approval, and use a newer corrected signed release with
explicit `-ReplaceFailedSetup` recovery. Do not delete the blocking receipt or
change already published metadata.

Keep the build/signing receipts, final installed manifest, source archive and
producer source revision/file hashes with the release evidence. Publish the
corresponding source and producer scripts used for the release. Generate signed
release metadata using the signed outer EXE hash and signed inner executable hash;
the server administrator must import and approve it before company packaging or
rollout. Any changed outer EXE also changes the technician portable artifact if
that format is derived from it, so its release hash must be regenerated.

Clean-machine installation, repair, update, reboot and uninstall tests are still
required for every advertised OS. The producer does not attest production signing
approval or independent reproducibility. Those gates must be demonstrated with
the actual release artifacts. Lab signatures remain lab-only.
