# Signed installed payload identity

Release metadata now requires `installed_files`, an array of objects containing
relative `path` and SHA-256 `sha256`. Customer manifests cover the final Flutter
payload: the installed Swan executable, native libraries and application assets.
The installed main executable hash must also equal `installed_sha256`.
Technician manifests contain one entry for `SwanRemoteSupport-Technician.exe`;
that hash covers its complete self-extracting payload.

Release metadata may declare `rollback_protocol: 1` only when both its endpoint
and configuration agent implement the failed-release quarantine and pending
rollback format described below. The release signer attests this compatibility;
do not add the declaration to old binaries. Omitted values default to zero and
permit forward recovery only. Unsupported protocol numbers are rejected.

Generate release inputs after signing the final binaries:

```text
python deployment/windows/generate-payload-manifest.py --edition customer --source FINAL_FLUTTER_DIRECTORY --output customer-installed-files.json
python deployment/windows/generate-payload-manifest.py --edition technician --source SIGNED_PORTABLE_EXE --output technician-installed-files.json
```

For customer packaging, the generator maps the build's `rustdesk.exe` to the
installed `Swan Remote Support.exe`. Include the resulting array in release
metadata and sign that metadata with the configured release-signing key.
Unsigned CI manifest drafts identify unsigned build bytes only; regenerate them
after signing. Do not relabel the drafts as signed release evidence.

Release validation rejects duplicate Windows path aliases, traversal, alternate
streams, reserved names, missing core native libraries and an inconsistent main
executable identity. Initial installation, repair, update completion, installed
metadata verification and recovery verify customer file hashes and reject
unlisted DLLs or EXEs. Paths resolving outside the installed directory fail.
The agent has its separate signed release hash and is checked during initial
installation and replacement through the staged update helper.

RustDesk's copied `RuntimeBroker_rustdesk.exe` is Windows-owned and cannot be
listed in project metadata. Verification instead requires its bytes to match
`RuntimeBroker.exe` in the Windows system directory returned by the OS API.
Windows servicing can change those bytes, requiring the installed copy to be
refreshed. Compatibility and repair behavior after Windows servicing still need
clean-machine testing.

This is a development metadata change. Previously generated releases without
the mandatory manifest are rejected; do not weaken validation to import them.
No production compatibility or signing approval is implied. Private lab-signed
customer MSI installation and automatic replacement have passed on Server 2016.
Rollback, the remaining Windows compatibility matrix and native session behavior
remain acceptance requirements; see [ACCEPTANCE.md](ACCEPTANCE.md).

Interrupted-update receipts also block new managed sessions after the updater
process exits or releases its activity lock. Recovery rechecks Windows support
and the installed agent hash and publisher as well as the endpoint payload.
An old or missing agent cannot be accepted as a completed release. Agent replacement
is implemented through a staged signed helper and has passed private lab-signed
customer updates on Server 2016. Full rollback behavior remains unverified. A release
stays pending until its full installed identity is restored and verified.

Automatic updates now stage both the installer and agent and verify their
signed hashes and pinned publishers before writing the pending receipt. The
watcher hands off to the staged agent and exits. The helper acquires exclusive
session exclusion, revalidates the receipt and staged artifacts, installs the
endpoint, replaces the old agent, and verifies the complete installed identity
before advancing the sequence or removing the receipt. Customer completion
restarts only the existing SYSTEM task whose action matches the canonical agent
and `watch` arguments. No credentials are passed to the helper.

Downloaded installers and helpers are published through a unique temporary file
on the destination volume, flushed before atomic replacement. Download staging
and replacement run outside the async runtime threads. A failed replacement
cleans its temporary file and retains the previous complete staged artifact;
hash and publisher verification still precede execution. Component regression
coverage checks complete replacement and a destination-conflict failure. This
does not establish recovery from a power loss or a failed native installer.

For the real HTTPS download regression, build the `swan-agent` test executable
with `cargo test --manifest-path product/Cargo.toml --locked -p swan-agent --no-run`.
Run `node deployment/windows/test-update-download.js PRIVATE_ENV TEST_EXE NEW_RESULT_JSON`.
The private environment supplies `SWAN_TEST_TLS_PFX`, `SWAN_TEST_TLS_PASSWORD`
and `SWAN_TEST_CA_FILE` for a verified localhost test identity. The harness binds
an ephemeral loopback port, executes only the explicitly ignored download test,
and closes its own listener. It checks an actual HTTPS redirect, absent artifact
credentials, tampered bytes and an interrupted response. Evidence includes the
test executable hash; no installer is executed. Do not commit private TLS files
or the environment. Release builds do not accept the debug-only test CA input.

A watcher can resume a pending update after restart; administrators can invoke
`swan-agent resume-update` to hand off explicitly. `recover-update` continues to
verify an already completed installation without rerunning its installer.
Recovery may finish the previously selected signed release after metadata expiry;
it cannot authorize a new release or relax the stored sequence. The helper log
and previous agent/portable executable are retained in the protected update
staging directory. Missing or hash-damaged staged installers and helpers are
re-downloaded from the exact HTTPS URLs in the validated pending receipt. These
downloads send no company credentials, must match the receipt's hashes and
pinned publisher, and do not select a newer release. Intact staging remains
usable offline. Recovery rechecks the unchanged receipt and session exclusion
before handoff. Permission and other staging I/O errors remain errors rather
than being treated as missing files. Complete customer/MSI rollback and canonical-agent
recovery remain incomplete. Graphical technician update coordination and this
staging recovery from missing or damaged files still require native end-to-end
verification. A Server 2016 lab test interrupted the helper before MSI execution;
the normal scheduled retry completed the selected signed customer MSI and agent
update, preserving identity and consent. This demonstrates forward recovery for
that interruption point, not power-loss recovery during MSI or technician updates.

Recovery validation is shared by completion, retry and helper execution. Its
regression tests reject wrong keys/editions, tampering, conflicting sequences
and unknown phases, including the case where a crash occurred after sequence
persistence. Automatic retries use a durable five-minute attempt delay. Explicit
`resume-update` requests retry immediately while retaining all trust and locking
checks. A failed helper requests restart of the matching customer task, provided
its installed agent still has a valid signature; pending receipts continue to
block sessions. Corrupt or missing canonical agents still require recovery from
a verified staged helper or explicit setup/repair. Scheduled forward retry and
post-upgrade reboot persistence have passed on Server 2016 with private lab trust.
Power-loss behavior remains unverified.

New automatic updates require a previously recorded signed installation. Before
writing their pending receipt, they verify the current installed release and
publish a complete rollback snapshot under `updates/SEQUENCE/rollback`: the
signed installed-release envelope, every manifest-listed endpoint file (including
customer libraries and assets), and the matching configuration agent. Snapshot
files are hash checked while copying, flushed, and verified before the complete
directory is renamed into place. The old metadata is accepted after expiry only
for its exact recorded sequence and edition under the existing release trust key.
Executable and agent publisher checks remain mandatory. The update receipt binds
that previous envelope; a helper verifies its snapshot before changing the payload.
Mutable branding, identity, credentials and consent are not copied into release
snapshots or restored to older values. Legacy receipts without a snapshot can
continue forward recovery, but do not gain rollback evidence retroactively.

Portable technician EXE updates now restore those verified inputs after an
installation or installed-identity failure, provided the previous signed metadata
declares rollback protocol 1 and the verified previous agent's `rollback-protocol`
command confirms support. A durable `rolling_back` receipt precedes restoration.
Automatic upgrades retain the installed EXE/MSI format; changing formats requires
explicit installation/migration so installer registrations are not silently lost.
Clients derive that format from their exact signed installed-release metadata
and send `format=exe` or `format=msi` when requesting an approved update. The
management server selects the latest approved, unexpired release matching that
format, edition and channel, while still applying rollout and maintenance policy.
Legacy requests without a format retain discovery compatibility. New clients
also reject a mismatched response, including one from an older server that ignores
the format query. Missing or invalid installed metadata requires explicit repair.
Retries finish that same rollback instead of attempting forward installation;
rollback does not require the new installer package to remain available. The
old EXE and agent are replaced through verified atomic staging, and their full
installed identity is checked before the previous metadata becomes current.
Deleted applications are not resurrected, and a committed newer installation
cannot be downgraded by this recovery path. The unchanged state retains the
previous installed sequence and current company identity and consent.

Before removing the pending receipt, rollback durably records `failed-update.json`
with the failed and previous signed envelopes. Discovery and fresh installation
reject sequences at or below that watermark; only a later approved sequence can
advance. Repair still requires the exact installed release. Invalid quarantine
metadata, wrong trust or edition, unknown phases/protocols and I/O errors fail
closed. Quarantine persists through subsequent successful installations. No
technician login token is passed to or restored by rollback.

Windows service/MSI registration restoration is implemented through the narrowly
validated `Restore-ReleaseMsi.ps1` path, but signed native rollback and full
power-loss acceptance remain unverified. Prepared portable technician rollback
has restored lab-signed endpoint and agent binaries on Server 2016; native
installed-identity and quarantine checks passed independently. The original
SSH harness and app restart remain unresolved, and automatic failure selection
has not been demonstrated. See [ACCEPTANCE.md](ACCEPTANCE.md) for the scope.
Component tests cover complete file
preservation, rejection of tampered/missing sources and unsafe paths, incomplete
snapshot cleanup, immutable published snapshots, and previous metadata trust and
sequence checks. Quarantine/receipt regressions cover replay suppression, eligibility
of later releases, metadata expiry, old protocol exclusion, wrong edition/key,
tampering, malformed storage and legacy forward recovery. Signed installed-app
rehearsals remain required; these tests do not substitute for native rollback.

Initial installation, explicit repair, completed updates and completion-only
recovery now retain the exact installer under `releases/SEQUENCE/installer.exe`
or `installer.msi`. Retention rechecks its size, signed hash and publisher before
and after atomic copying. `record-installation` therefore requires the original
installer path as well as `release.json`; the company setup wrapper supplies both.
Installed sequence advancement and removal of recovery markers wait for this
retention step. New snapshots also preserve the original installer in `package/`.
Older installed versions without a cache can fetch only their exact previously
recorded artifact through verified HTTPS before an update is prepared. Expiry
does not authorize a different package. A missing, corrupt or unverifiable old
artifact defers the update before installation; keep corresponding release files
available on company-controlled storage. Retained packages are software artifacts,
not copies of mutable enrollment or consent. MSI/service restoration still needs
signed native end-to-end tests, including registration, service/task consistency
and interruptions during restoration.

The agent and Windows worker launch Windows PowerShell from the OS-resolved
system directory and restrict module discovery to its built-in modules. This
prevents inherited PowerShell 7 module paths from breaking hash/signature checks.
Local regressions confirm the built-in commands are present and that an unsigned
installer with a correct hash is rejected without publishing a retained package.
An actual Windows-signed executable copied into the isolated fixture passes
positive retention; wrong publisher and certificate pins are rejected while
retaining the previous complete cache. The fixture is never executed and no
certificate is installed. This checks Authenticode retention, not a signed Swan
installer or Windows Installer registration recovery.
