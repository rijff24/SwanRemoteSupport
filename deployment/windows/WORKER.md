# Company Windows installer worker

Each company provides its own Windows worker. A Linux company server also needs
this Windows machine; the project does not host a shared build service. The
worker connects outbound to the company's HTTPS management endpoint and consumes
approved signed release inputs. It does not execute company-provided build scripts.

Use the worker executable from the same pinned source/release as the management
server: the server validates a canonical bundle recipe. Verify the published
artifact hash and trusted Authenticode publisher before running it. A lab-signed
binary is a test fixture, not a production release.

An administrator registers a worker through the authenticated administration
interface/API (`POST /api/v1/workers`). The returned worker token is a secret.
Store it outside Git, in a local configuration file restricted to the worker
account, SYSTEM and administrators. Do not put it in command-line arguments,
public installers, website files, build artifacts or logs.

The worker reads these environment variables at startup:

| Variable | Meaning |
| --- | --- |
| `SWAN_MANAGEMENT_URL` | Company HTTPS management URL, without credentials. |
| `SWAN_WORKER_TOKEN` | Registered worker credential. |
| `SWAN_RELEASE_PUBLIC_KEY` | Independently verified release-signing public key. |
| `SWAN_PROFILE_PUBLIC_KEY` | Company's verified configuration-signing public key. |
| `SWAN_ARTIFACT_DIR` | Dedicated protected output directory. |

Load those values into the worker process environment from protected local
configuration, then launch `swan-worker.exe`. Use the normal Windows certificate
trust configuration for HTTPS; do not disable certificate validation. Keep
private TLS and signing keys on their owning server/signing provider. The worker
needs public trust pins, not the project signing private key.

The executable supports foreground operation and a Windows SCM entrypoint
(`swan-worker.exe --service`, service name `SwanInstallerWorker`). Install it from
an administrator PowerShell session with `Install-Worker.ps1 -Executable <path>
-ConfigurationPath <private-json-path> -PublisherThumbprint <trusted-thumbprint>`.
The JSON file must contain exactly four string fields: `management_url`,
`worker_token`, `release_public_key` and `profile_public_key`. Obtain their actual
values through company setup and authenticated worker registration; never commit
this file or share the token.

The installer checks the source and copied executable's publisher, refuses an
existing installation, protects its directories, and stores the service's secret
environment in a registry key restricted to SYSTEM and administrators. It uses
LocalSystem, automatic startup and SCM restart policy. It creates no inbound
firewall rule. Keep the original configuration file protected too. A manually
started foreground worker does not survive reboot. Verify authenticated build
completion, standard-user credential denial, service stop/restart and startup
after reboot before deployment; signed VM evidence for this new service path is
still pending. Service output is appended to protected `C:\ProgramData\SwanInstallerWorker\worker-service.log`; runtime log verification and log rotation remain pending.
Prevent a foreground worker from running alongside the service against the same
output directory.

Approve a compatible release on the server, queue a customer or technician build,
and inspect its terminal status, protected log and artifact hash. Customer bundle
downloads are public; technician bundle downloads require authentication. Test
both access paths and verify the downloaded hash before installation. Failed jobs
must not be published as successful packages.

After a company configuration-key rotation, independently verify the new public
pin and update the protected worker configuration before restarting it. Automatic
worker trust-pin rotation is not currently implemented. Regenerate bundles after
rotation; old completed builds are invalidated by the management service.

See [signing requirements](../../docs/WINDOWS_CODE_SIGNING.md),
[company signing integration](COMPANY-SIGNING.md), and
[acceptance evidence](../../docs/ACCEPTANCE.md) before production publication.

Remove an owned installation with administrator `Uninstall-Worker.ps1`; use
`-WhatIf` to inspect the operation first. It verifies service identity, receipt,
executable hash and publisher before stopping anything. It retains private
artifacts and the receipt, and does not remove the original configuration file.
Reinstallation currently refuses retained data; preserve it before planning a
replacement. Removal does not revoke the server-side worker token. Removal and
reinstallation VM acceptance are still pending.

Administrators can list worker IDs and disabled status with `GET /api/v1/workers`
and permanently disable a credential with `DELETE /api/v1/workers/{worker_id}`,
using their authenticated administrator session. Revocation atomically cancels
that worker's running and uploaded builds and records an audit event. Queue new
jobs for a replacement worker; cancelled claims cannot complete later. Completed
packages remain available, so withdraw their release separately if its published
packages must also be disabled. The list never returns tokens or token hashes.
