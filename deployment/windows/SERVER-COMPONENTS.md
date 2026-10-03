# Windows company server components

`Install-Server.ps1` installs the signed management service, optionally with
packaged rendezvous, relay and HTTPS components. The service supervises its
own children; component failure stops management, and service stop or a service
process crash terminates its assigned children. Windows process-job lifecycle
tests pass, but clean-machine installation, repair and uninstall are still
unverified. `Setup-Server.ps1` supplies the graphical setup flow, and an
unsigned test EXE bundles it with management and pinned transport/HTTPS inputs.
Production signing and clean-machine wizard installation remain incomplete.

`server-components.json` pins the Windows x64 RustDesk server 1.1.15 and Caddy
2.10.2 archives by SHA-256, with upstream source references. Download each exact
URL in that manifest to local storage, then prepare their executables:

```powershell
./deployment/windows/Prepare-ServerComponents.ps1 `
  -TransportPackage ./rustdesk-server-windows-x86_64-unsigned.zip `
  -HttpsPackage ./caddy_2.10.2_windows_amd64.zip `
  -OutputDirectory ./prepared-server-components
```

The destination must be new. Both archive hashes are checked before any output
is created. Only fixed `hbbs.exe`, `hbbr.exe` and `caddy.exe` basenames are
extracted; missing, duplicate or oversized executables are rejected. The output
also retains Caddy's archive license, the AGPL license and public source notices.
This does
not execute the supplier setup program, register services, open ports, change
DNS, or replace an existing transport deployment. An interrupted preparation
can leave partial output; use a new destination when retrying.

Archive hash verification establishes the pinned download identity. It does
not establish a trusted Authenticode publisher or production signing approval.
Production packaging must retain RustDesk AGPL source/license notices and
Caddy Apache-2.0 license/notices, publish exact corresponding project and
transport source, and pass the server lifecycle and Windows compatibility
checks. These prepared files are build inputs, not a complete server installer.

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
present in the targeted desktop Windows releases. Build only for isolated tests:

```powershell
./deployment/windows/Build-ServerSetup.ps1 -UnsignedTest `
  -ManagementExecutable ./swan-management.exe `
  -ComponentsDirectory ./prepared-server-components `
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

Use `--render-preview OUTPUT.png` to inspect the packaged layout without opening
the window or installing services. The launcher executes the installer from its
embedded resource in memory, using structured parameters. Public binary payloads
are extracted under a unique protected directory; elevated extraction grants
only Administrators/SYSTEM access and ownership. Non-elevated previews use the
current user's private temporary directory. Known payload files are cleaned up
on normal exit without recursive deletion. Elevated extraction, interrupted
cleanup and privileged installation remain clean-VM acceptance requirements.
The installed management copy is signature-checked again before service creation.

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
