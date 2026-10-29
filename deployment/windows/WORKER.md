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

Installed services read `C:\ProgramData\SwanInstallerWorker\worker.json`.
The service registry contains only `SWAN_WORKER_CONFIG`, the public path to that
file. The directory and file must be owned by Administrators, have protected
permissions granting only SYSTEM and Administrators access, and contain no
reparse points. The executable checks these permissions before reading secrets
or claiming a job. App-container read access is rejected too.

Foreground operator processes may still use `SWAN_MANAGEMENT_URL`,
`SWAN_WORKER_TOKEN`, `SWAN_RELEASE_PUBLIC_KEY`, `SWAN_PROFILE_PUBLIC_KEY` and
`SWAN_ARTIFACT_DIR` in their process environment. Never persist these secret
values in service registry settings. Service mode requires the private file and
cannot fall back to environment credentials. HTTPS certificate validation must
remain enabled; workers need public trust pins, never project signing keys.

The executable supports foreground operation and a Windows SCM entrypoint
(`swan-worker.exe --service`, service name `SwanInstallerWorker`). Install it from
an administrator PowerShell session with `Install-Worker.ps1 -Executable <path>
-ConfigurationPath <private-json-path> -PublisherThumbprint <trusted-thumbprint>`.
The JSON file must contain exactly four string fields: `management_url`,
`worker_token`, `release_public_key` and `profile_public_key`. Obtain their actual
values through company setup and authenticated worker registration; never commit
this file or share the token.

The installer checks the source and copied executable's publisher, refuses an
existing installation, and creates the state directory with private permissions
atomically. It protects the empty configuration file before writing credentials,
then asks the compiled worker to validate it before creating the service. Keep
the input configuration in a private local administrator directory; its parent
and file must allow only SYSTEM, Administrators and the installing administrator,
with full access for SYSTEM and Administrators. Ship the matching
`Read-WorkerConfiguration.ps1` beside `Install-Worker.ps1`.

The service uses LocalSystem, automatic startup and SCM restart policy, with no
inbound firewall rule. Its artifacts use the canonical private state directory.
Service output is appended to protected `worker-service.log`; log rotation
remains pending. Do not run a foreground worker alongside the service.

**Legacy credential storage:** earlier development installers persisted the
worker token in the service registry. A real Server 2016 test found additional
app-container read permissions despite a protected registry ACL. Revoke those
worker registrations before upgrading, keep the old worker stopped, and remove
its secret environment values. Never merely copy the old token into a new file.
The source now uses private file storage. Compiled Windows permission checks
and a Server 2016 lab migration, actual standard-user read denial, both edition
builds and authenticated build completion after reboot passed; see the dated
acceptance evidence. Fresh installation with this installer, the complete Windows
matrix and production signing remain separate gates. Existing installations and
retained data are refused rather than silently overwritten by fresh setup. Preserve receipts and artifacts while
planning migration. A restored management backup can resurrect revoked token
hashes: revoke legacy registrations again before exposing a restored service.

For a stopped schema-1 installation, `Migrate-WorkerCredential.ps1 -Executable
<new-signed-worker> -PreviousConfigurationPath <private-original-json>
-ConfigurationPath <private-new-json> -PublisherThumbprint <trusted-thumbprint>`
provides an explicit migration path. First revoke the old registration and set
the service to manual startup. The script requires the same company and public
trust keys, a different credential, a real HTTPS 401 response for the old
credential, and HTTP 200 for the replacement worker credential. It preserves the old executable and receipt under a unique private
directory, retains artifacts, and records a migration journal. On failure it
keeps the service stopped/manual; it never reactivates the revoked legacy
credential. Inspect `-WhatIf` first. A retained journal/configuration requires
explicit recovery rather than repeating migration. The actual lab-signed worker
migration passed on Server 2016, preserving the legacy executable/receipt and a
retention fixture. Both branded packages were generated afterward; private file
permissions and authentication survived reboot. This does not qualify every
Windows version or a production publisher.

Approve a compatible release on the server, queue a customer or technician build,
and inspect its terminal status, protected log and artifact hash. Customer bundle
downloads are public; technician bundle downloads require authentication. Test
both access paths and verify the downloaded hash before installation. Failed jobs
must not be published as successful packages.

A permanent upload-validation rejection (HTTP 400, 413, 415 or 422) produces a
failed completion report with no download URL or artifact hash. The retained
local bundle remains available for inspection. This prevents malformed or
incompatible output from remaining a running job indefinitely. Review matching
server/worker releases and approved inputs before queueing a replacement job.
Authentication/authorization failures, conflicting state, network failures,
rate limits and temporary server errors retain recovery rather than fabricating
a terminal result. Temporary upload failures retry the identical buffered bytes.

Error details admit only fixed public rejection labels from a bounded JSON
response. Arbitrary server text, credentials and response URLs are not copied
into job logs; diagnostic body reading also has a short deadline. This behavior
requires deployment of the updated worker and does not repair old pending jobs
by itself. See the dated acceptance record for unit/CI and runtime scope.

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
replacement. Removal does not revoke the server-side worker token. Disposable
Server 2016 removal and reinstallation passed after explicitly archiving retained
data; automatic migration of retained data remains unverified.

Administrators can list worker IDs and disabled status with `GET /api/v1/workers`
and permanently disable a credential with `DELETE /api/v1/workers/{worker_id}`,
using their authenticated administrator session. Revocation atomically cancels
that worker's running and uploaded builds and records an audit event. Queue new
jobs for a replacement worker; cancelled claims cannot complete later. Completed
packages remain available, so withdraw their release separately if its published
packages must also be disabled. The list never returns tokens or token hashes.
