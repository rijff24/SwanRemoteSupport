# Windows company server components

`Install-Server.ps1` currently installs the signed management service only.
The combined Windows wizard, HTTPS/transport service supervision and their
installation, repair and uninstall tests are still incomplete.

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
extracted; missing, duplicate or oversized executables are rejected. This does
not execute the supplier setup program, register services, open ports, change
DNS, or replace an existing transport deployment. An interrupted preparation
can leave partial output; use a new destination when retrying.

Archive hash verification establishes the pinned download identity. It does
not establish a trusted Authenticode publisher or production signing approval.
Production packaging must retain RustDesk AGPL source/license notices and
Caddy Apache-2.0 license/notices, publish exact corresponding project and
transport source, and pass the server lifecycle and Windows compatibility
checks. These prepared files are build inputs, not a complete server installer.
