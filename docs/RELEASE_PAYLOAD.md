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
An old or missing agent cannot be accepted as a completed release. Automatic
agent replacement and full interrupted-install retry/rollback remain incomplete;
a changed-agent release stays pending until its full installed identity is
restored and verified.
