# Signed installed payload identity

Release metadata now requires `installed_files`, an array of objects containing
relative `path` and SHA-256 `sha256`. Customer manifests cover the final Flutter
payload: the installed Swan executable, native libraries and application assets.
The installed main executable hash must also equal `installed_sha256`.
Technician manifests contain one entry for `SwanRemoteSupport-Technician.exe`;
that hash covers its complete self-extracting payload.

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
installation; agent replacement during updates remains incomplete.

RustDesk's copied `RuntimeBroker_rustdesk.exe` is Windows-owned and cannot be
listed in project metadata. Verification instead requires its bytes to match
`RuntimeBroker.exe` in the Windows system directory returned by the OS API.
Windows servicing can change those bytes, requiring the installed copy to be
refreshed. Compatibility and repair behavior after Windows servicing still need
clean-machine testing.

This is a development metadata change. Previously generated releases without
the mandatory manifest are rejected; do not weaken validation to import them.
No production compatibility or signing approval is implied. Full real signed
payload installation, update replacement, rollback and native behavior remain
acceptance requirements.

Interrupted-update receipts also block new managed sessions after the updater
process exits or releases its activity lock. Recovery rechecks Windows support
and the installed agent hash and publisher as well as the endpoint payload.
An old or missing agent cannot be accepted as a completed release. Agent replacement is implemented through a staged signed helper; its real
Windows installation and full rollback behavior remain unverified. A release
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

A watcher can resume a pending update after restart; administrators can invoke
`swan-agent resume-update` to hand off explicitly. `recover-update` continues to
verify an already completed installation without rerunning its installer.
Recovery may finish the previously selected signed release after metadata expiry;
it cannot authorize a new release or relax the stored sequence. The helper log
and previous agent/portable executable are retained in the protected update
staging directory. Automatic rollback, damaged-stage re-download and graphical
technician update coordination remain incomplete. Signed clean-machine tests
are required before claiming interruption or self-update acceptance.

Recovery validation is shared by completion, retry and helper execution. Its
regression tests reject wrong keys/editions, tampering, conflicting sequences
and unknown phases, including the case where a crash occurred after sequence
persistence. Automatic retries use a durable five-minute attempt delay. Explicit
`resume-update` requests retry immediately while retaining all trust and locking
checks. A failed helper requests restart of the matching customer task, provided
its installed agent still has a valid signature; pending receipts continue to
block sessions. Corrupt or missing canonical agents still require recovery from
a verified staged helper or explicit setup/repair. Actual scheduled-task and
power-loss behavior remain unverified.
