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
(`swan-worker.exe --service`, service name `SwanInstallerWorker`). For unattended
operation, installation tooling must register it under an appropriate account
at boot, load protected configuration, record protected logs and restart it
after process failure. The service handles SCM stop/shutdown requests. The
repository does not yet supply a Windows worker
service/startup installer or signed VM verification of this entrypoint; do not assume a manually started worker survives
reboot. Prevent concurrent supervisors from starting duplicate workers against
the same output directory. Verify startup and a completed job after reboot.

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
