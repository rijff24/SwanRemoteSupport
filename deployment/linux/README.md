# Company Linux deployment

Each company operates its own management, rendezvous and relay services. The
project does not supply hosting. These deployment files still require the
release checks in [ACCEPTANCE.md](../../docs/ACCEPTANCE.md); intended Windows
compatibility and production signing are not verified claims.

From this directory, copy `company.env.example` to `company.env`, set the
company's public hostname and trusted project release verification public key,
then run:

```sh
docker compose --env-file company.env up -d --build
```

The company supplies DNS, inbound TCP 80/443 for HTTPS provisioning, TCP
21115/21116/21117 and UDP 21116 for native transport. Office hosting may need
port forwarding or a separately reachable company relay under CGNAT. Setup
diagnostics run from the company server and do not prove external connectivity.
Do not alter an existing production deployment to rehearse these instructions.

Open `https://YOUR_COMPANY_HOSTNAME` for the first-run administration wizard.
Retrieve the one-time setup token privately on the company server:

```sh
docker compose --env-file company.env exec management cat /var/lib/swan/setup-token.txt
```

Keep the token out of shared logs and issues. Enter the company identity,
branding, HTTPS and transport addresses, administrator credentials and an
authenticator code in the wizard. The transport public key can be read from
`/root/id_ed25519.pub` in the `hbbs` container; the corresponding private key
must remain on the company server. Endpoints receive public bootstrap
configuration and require company approval after enrollment. A Linux server
also needs a company-controlled Windows worker for Windows installation bundles.

Management runs as UID 10001 with a read-only root filesystem and private
persistent storage. Docker's SIGTERM requests a graceful shutdown. Preserve
all management, transport and Caddy volumes. Follow [SERVER_BACKUP.md](../../docs/SERVER_BACKUP.md)
for encrypted database, configuration, transport and full TLS storage backup
and restore; backing up only the management volume is insufficient.

## Build and deployment checks

The root and management Dockerfile both carry matching deny-by-default context
rules, including explicit child exclusions for older Docker engines. They admit
the named Cargo manifests, source, embedded scripts and branding assets while
excluding local environment files, keys, databases, installers and caches from
the build daemon and intermediate layers. Add newly required public build
inputs explicitly when changing the embedded source.

The Rust builder, Debian runtime, Caddy HTTPS proxy and RustDesk transport images
are pinned by manifest digest.
Update those digests deliberately and rerun these checks when applying base-image
security updates. The runtime copies its public certificate trust bundle from
the pinned builder, rather than installing packages from live repositories.
Rustls reads that bundle through `SSL_CERT_FILE`; HTTPS certificate verification
remains enabled. Bit-for-bit reproducibility still requires a separate rebuild
comparison and is not established by the deployment tests.

From the repository root, with a working Linux Docker engine:

```sh
python3 deployment/linux/test_build_context.py
docker build --file deployment/linux/Dockerfile --tag swan-management:test .
python3 deployment/linux/test_container_startup.py swan-management:test
```

The context test uses tracked public source and synthetic canaries, never local
credentials. The container test uses uniquely named disposable storage and a
loopback-only listener. It checks fresh setup, MFA and replay denial, required
device approval, policy denial, graceful shutdown and persisted company/device
identity. Credentials stay in memory and the test removes only its own container
and volume. These are actual Docker checks, but are not evidence for public
HTTPS, transport sessions, worker-generated installations, certificate renewal
or the Windows compatibility matrix. The company-product CI runs both checks.
