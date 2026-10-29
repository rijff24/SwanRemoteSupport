# Windows company server components

`Install-Server.ps1` installs the signed management service, optionally with
packaged rendezvous, relay and HTTPS components. The service supervises its
own children; component failure stops management, and service stop or a service
process crash terminates its assigned children. Windows process-job lifecycle
tests pass, but clean-machine installation, repair and uninstall are still
unverified. `Setup-Server.ps1` supplies the graphical setup flow, and an
unsigned test EXE bundles it with management and pinned transport/HTTPS inputs.
Production signing and clean-machine wizard installation remain incomplete.

`server-components.json` now requires the maintained Windows rendezvous and
relay builds, with fixed executable hashes and a canonical digest of their
complete compiler inputs and public recipe. `deployment/transport/source.json`
pins upstream RustDesk server and its shared submodule, Rust, Cargo.lock and the
reviewed UDP address normalization. The stock upstream Windows archive is no
longer accepted by this preparation path. Caddy 2.10.2 remains archive- and
executable-hash pinned.

Native and component workflows build the maintained Windows transport in the
same workflow run, archive its source before compilation, and consume that run's
artifact. Download the `swan-transport-candidate-windows-2022` artifact ZIP and
the exact HTTPS archive named in the manifest, then prepare them with Python
available as a build tool:

```powershell
./deployment/windows/Prepare-ServerComponents.ps1 `
  -TransportPackage ./swan-transport-candidate-windows-2022.zip `
  -HttpsPackage ./caddy_2.10.2_windows_amd64.zip `
  -OutputDirectory ./prepared-server-components
```

For an artifact already extracted by the CI download action, use
`-TransportDirectory` instead of `-TransportPackage`. The four expected inputs
retain their artifact paths (`transport-source/target/release/`,
`transport-source/SWAN-SOURCE.json` and `transport-artifacts/`). Use
`-PythonExecutable` for an explicit Python path. Python is a build requirement;
the installed server and setup wizard do not require it.

The destination must be new. All inputs are verified before output creation.
The verifier rejects altered executables, changed/extra/missing source files,
forged receipts, duplicate/unsafe archive entries, and a wrong HTTPS archive.
Only fixed filenames are written; supplier executables are not run. The prepared
set includes both licenses, attribution, the manifest and the complete
`transport-build-source.zip`. That source archive is embedded in the server
setup, retained in the installed components directory and included in removal
ownership checks. Its ZIP metadata and text checkout line endings are normalized
for reproducible packaging; executable bytes remain unchanged. Public build
receipt revisions remain in the archive, separate from the digest of invariant
compiler/recipe inputs. `component-verification.json` records preparation hashes
outside the embedded installer payload.

This is the unsigned test packaging path. The maintained Windows transport has
separate source, binary reproducibility and IPv4/IPv6 registration evidence;
new installer compilation, installation, full native relay sessions, the Windows
matrix and production signing remain release gates. A new management binary
requires its matching transport pins: do not replace an existing management
executable alone. Preserve company data and keys, stop builds/sessions, create
an encrypted full deployment backup and perform a coordinated administrator
upgrade with tested rollback. Existing lab services are not changed by source
or packaging preparation.

Open the graphical wizard from an administrator Windows PowerShell session:

```powershell
powershell.exe -STA -File ./deployment/windows/Setup-Server.ps1
```

It collects the company hostname, trusted release key and publisher thumbprint,
signed management executable and prepared components. Installation runs in a
background PowerShell instance with structured parameters, so the form stays
responsive; it prevents closing while installation runs. After success, open
company HTTPS setup to configure branding, administrator MFA, accounts,
enrollment and update policy. No secrets are embedded in the wizard or copied
to public installation bundles. Without administrator rights, installation is
disabled. The wizard does not request elevation automatically.

For layout inspection, `-RenderPreview OUTPUT.png` renders the form without
showing it, starting services or enabling installation/browser actions. The
preview is layout evidence; it does not test privileged installation or backend
completion/error events. Those require the clean Windows test matrix.

## Unsigned test executable

The x64 .NET Framework launcher uses the Windows PowerShell runtime already
present in the targeted desktop Windows releases. Building requires .NET Framework
4.7.2 or newer and the hash-pinned Microsoft Roslyn 4.12.0 package described in
`server-compiler.json`. The compiler is a build dependency and is not embedded
in the installer. Download it separately using `Get-PinnedArchive.ps1`; the
builder verifies the whole package, extracts into a fresh folder and checks the
compiler executable hash and Microsoft signature before executing it. Download
both build-only SDK reference packages pinned in `server-references.json` too;
the builder does not compile against reference assemblies from the host.
Build only for isolated tests:

```powershell
./deployment/windows/Build-ServerSetup.ps1 -UnsignedTest `
  -ManagementExecutable ./swan-management.exe `
  -ComponentsDirectory ./prepared-server-components `
  -CompilerPackage ./microsoft.net.compilers.toolset.framework.4.12.0.nupkg `
  -FrameworkReferencesPackage ./microsoft.netframework.referenceassemblies.net462.1.0.3.nupkg `
  -PowerShellReferencesPackage ./microsoft.powershell.5.referenceassemblies.1.1.0.nupkg `
  -OutputDirectory ./server-setup-test
```

It embeds the wizard and installer scripts, management, pinned transport/HTTPS
executables, licenses and public source/hash metadata. Input PE architecture
and component hashes are checked before output; compiled resources must match
their input hashes. Output is `SwanServerSetup-UNSIGNED-TEST.exe`, with test
status also shown in its window title. Building without `-UnsignedTest` is
rejected. A supplied unsigned management executable may be packaged for layout
tests, but installation still requires its valid trusted publisher signature.
This does not bypass signing gates or grant production publication approval.
Resource input paths containing commas are currently unsupported by this builder.

The recipe uses deterministic compilation, explicit .NET Framework references,
mapped source paths and ordered compact source metadata. `SOURCE.json` records
compiler/source/package pins and reference-assembly hashes; every embedded
resource, including that manifest, is verified after compilation. The reference
packages are `Microsoft.NETFramework.ReferenceAssemblies.net462` 1.0.3 and
`Microsoft.PowerShell.5.ReferenceAssemblies` 1.1.0. Only five fixed, individually
hash-checked DLL basenames are extracted; SDK install scripts never run and
reference assemblies are not embedded in the server installer. Run
`test_server_setup_reproducibility.ps1` with the same build inputs and a fresh
`-WorkDirectory` to compare two actual unsigned outputs and test rejection of a
wrong compiler or reference package. It also executes the packaged launcher's
preview-only mode and retains its PNG to check runtime binding to Windows
PowerShell. Independent cross-image comparisons remain required. This does not
install or qualify the company server.

Use `--render-preview OUTPUT.png` to inspect the packaged layout without opening
the window or installing services. The launcher executes the installer from its
embedded resource in memory, using structured parameters. Public binary payloads
are extracted under a unique protected directory; elevated extraction grants
only Administrators/SYSTEM access and ownership. Non-elevated previews use the
current user's private temporary directory. Known payload files are cleaned up
on normal exit without recursive deletion. Elevated extraction, interrupted
cleanup and privileged installation remain clean-VM acceptance requirements.
The installed management copy is signature-checked again before service creation.
It must be the matching management build supporting `prepare-server-directory`.
After verifying the protected Program Files copy, installation invokes that
build's embedded private-directory helper before writing any company state.
The helper creates the canonical server data root with SYSTEM/Administrators
permissions atomically and rejects existing state, including a directory that
appeared after the fresh-install precheck. Existing state is never adopted by
changing its owner or permissions. Preparation has a bounded 30-second deadline;
incompatible older executables are not a supported installation input. Unknown
commands in current management builds fail before opening company data or
starting a listener. Failure retains newly staged files for investigation.

For a fresh company deployment, run with administrator rights and a trusted
signed management build containing component supervision:

```powershell
./deployment/windows/Install-Server.ps1 `
  -Executable ./swan-management.exe `
  -ReleasePublicKey PROJECT_RELEASE_PUBLIC_KEY `
  -PublisherThumbprint TRUSTED_RELEASE_CERTIFICATE_THUMBPRINT `
  -ComponentsDirectory ./prepared-server-components `
  -PublicHostname support.example.com
```

The script rejects existing service/data/install directories, missing licenses,
modified component executables, invalid hostnames and occupied component ports.
The management signature must be valid and match the explicitly configured
certificate thumbprint from the trusted project or company release policy.
It protects company data, copies the components under the management installation,
writes a Caddy recipe, registers automatic service recovery, and opens TCP
80/443/21115–21117 and UDP 21116 for this deployment. Management stays on loopback.
Configure public company DNS and forwarding before completing the shared web
setup wizard. Public ACME issuance and external transport tests remain required;
the script's management health check proves neither.

The service rechecks exact executable hashes before launching fixed commands.
Rendezvous and relay share `transport/` beneath company data; relay starts only
after a public transport key is available. Caddy keeps its state beneath `tls/`
and `proxy-config/`, with its administration API disabled. Component logs are in
the protected `logs/` directory. Child processes are terminated and waited for
on service shutdown; this is forced termination, so full backup must wait until
the service has stopped and must snapshot SQLite using the management CLI.
Include the complete transport and TLS directories in encrypted backups.

`swan-management.exe check-components` validates packaged executable hashes,
hostname and the presence of a Caddyfile without launching services. It creates
component data/log directories under `SWAN_DATA_DIR`; it does not parse the
HTTPS recipe or prove DNS, certificates or connectivity. Set `SWAN_PUBLIC_HOST`
and `SWAN_DATA_DIR` for this preflight. Installation failure removes only its
new service and firewall rules, retaining files/private data for inspection.

## Removal

Run `Uninstall-Server.ps1` with administrator rights to remove this company
service, its owned firewall rules and the known packaged files. `-WhatIf` checks
the removal plan without changing them. The installer records public ownership
metadata in the protected `installation.json`; removal refuses a different
service executable, company directory, reparse point or unexpected firewall
ownership. It does not recursively delete installation directories or remove
unknown files. Company database, configuration, enrollment, keys, TLS storage,
logs and receipt remain in ProgramData for recovery.

Existing installations without this receipt need an administrator-reviewed
maintenance procedure. A fresh install deliberately refuses retained company
data, so do not delete it merely to get past setup. Native removal, reinstallation
and restore tests still require clean Windows VMs; removal-plan fixtures prove
ownership rejection and exclusion of data, not successful SCM/firewall removal.

Windows CI is configured to build `SwanServerSetup-UNSIGNED-TEST.exe` with these pinned
transport/HTTPS archives and the matching management executable. Its artifact
includes `SwanServerSetup-SOURCE.json` and a checksum. The setup builder checks
embedded resource hashes and retains component licenses/source notices. These
outputs remain test packages: production signing and clean-machine setup
acceptance are required before publication.
