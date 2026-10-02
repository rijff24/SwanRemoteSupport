# Windows company server components

`Install-Server.ps1` installs the signed management service, optionally with
packaged rendezvous, relay and HTTPS components. The service supervises its
own children; component failure stops management, and service stop or a service
process crash terminates its assigned children. Windows process-job lifecycle
tests pass, but clean-machine installation, repair and uninstall are still
unverified. A graphical setup wrapper remains incomplete.

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

For a fresh company deployment, run with administrator rights and a trusted
signed management build containing component supervision:

```powershell
./deployment/windows/Install-Server.ps1 `
  -Executable ./swan-management.exe `
  -ReleasePublicKey PROJECT_RELEASE_PUBLIC_KEY `
  -ComponentsDirectory ./prepared-server-components `
  -PublicHostname support.example.com
```

The script rejects existing service/data/install directories, missing licenses,
modified component executables, invalid hostnames and occupied component ports.
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
