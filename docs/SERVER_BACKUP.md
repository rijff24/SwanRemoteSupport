# Company server backup and restore

Supply `SWAN_DATA_DIR` and `SWAN_BACKUP_PASSPHRASE` to the management process
through private environment configuration. Keep the passphrase separate from
the backup file. It must contain at least 16 characters. These commands never
accept a passphrase on the command line.

Management-only export remains available:

```text
swan-management backup company.swan-backup
```

For a deployment export, replace the path placeholders with company-controlled
paths. Omit `--tls-identity` only when HTTPS identity will be reissued separately.
The TLS identity argument accepts one existing file, such as a PFX or PEM bundle.
Use `--tls-directory HTTPS_STORAGE_DIRECTORY` for the complete certificate-service
storage tree, including ACME account and certificate files. For Caddy, supply its
persistent data directory. Stop certificate renewal and other writes to this tree
for the duration of export. Export reads it without stopping or modifying services.

```text
swan-management backup company.swan-backup --transport-directory TRANSPORT_DATA --deployment-env PRIVATE_ENV_FILE --tls-identity TLS_IDENTITY_FILE
```

Alternatively, replace the final option with `--tls-directory HTTPS_STORAGE_DIRECTORY`.
Both options can be included when the deployment uses separate storage and identity files.
Directory exports reject symbolic links, Windows reparse points, special files,
unsafe paths, and more than 4096 total files. The 512 MiB total plaintext limit applies.
When the company data directory contains `Caddyfile`, export also encrypts that
HTTPS configuration and restore places it back in the replacement data directory.
Keep configuration edits stopped during export. Older archives without this
file remain supported; reconstruct and validate their HTTPS recipe before use.

The transport directory must contain both `id_ed25519` and `id_ed25519.pub`.
When present, `db_v2.sqlite3` is also snapshotted using SQLite's backup API.
Management uses the same API for its database, including committed WAL data.
Schema version 2 keeps active and pending profile-signing keys in that database;
the original `profile-key.hex` file alone is insufficient after rotation. See
[profile key rotation](PROFILE_KEY_ROTATION.md) for the migration and maintenance procedure.
Stop configuration changes during the export when a coordinated snapshot of
the separately stored environment, TLS identity and transport data is needed.
The export does not stop or modify running transport services.

All contents are encrypted with AES-256-GCM and an Argon2-derived key. Existing
backup files are never overwritten. Temporary plaintext database snapshots are
removed after archive generation. Archives and plaintext contents have size
limits; this is not a general-purpose filesystem backup tool.

Restore into an **empty directory** while the replacement management service
is stopped and its public endpoints are unavailable:

```text
swan-management restore company.swan-backup
```

Set `SWAN_DATA_DIR` to that empty replacement directory before running restore.
The restorer authenticates and validates the archive before writing contents.
It permits fixed management filenames and validated relative paths under
`tls-storage/`, rejects duplicate or Windows-colliding names, file/directory
conflicts and incomplete key pairs, and protects restored files with private Unix permissions or Windows
ACLs for the invoking account, SYSTEM and Administrators.

Management files restore directly. Deployment extras are staged in that same
protected directory; restore never overwrites external service files:

| Restored file | Destination to configure before exposing the server |
| --- | --- |
| `transport-id_ed25519` | Transport data directory as `id_ed25519` |
| `transport-id_ed25519.pub` | Transport data directory as `id_ed25519.pub` |
| `transport-db.sqlite3` | Transport data directory as `db_v2.sqlite3` |
| `deployment.env` | Private deployment environment configuration |
| `tls-identity` | Company HTTPS identity location, or replace through certificate reissuance |
| `tls-storage/` | Complete HTTPS service data directory; copy while that service is stopped |
| `Caddyfile` | HTTPS recipe in the replacement company data directory; validate paths and hostnames before starting |

Keep transport services stopped while placing their restored data. Preserve
private ownership and permissions appropriate to the service accounts. Use the
same application release first, then perform administrator-controlled upgrades.
Current restores also accept older management-only three-file backups; older
restorers cannot consume the extended deployment archive.

Before exposing the restored deployment, verify the transport and profile trust
keys, database integrity, MFA login, device identities, consent, permissions and
release approvals. Reconcile any revocations made after the backup. Installed
clients reject profiles older than their cached revision, so recover a profile
revision newer than those clients have accepted before attempting normal sync.
These restore rehearsals and migration rollback checks remain release gates.

## Isolated restore rehearsal

`node deployment/windows/test-server-restore.js ORIGINAL_PRIVATE_ENV
RESTORED_PRIVATE_ENV` checks two separately running local HTTPS test servers.
Use only the isolated fixture created by `test-company-lifecycle.js`; its private
administrator credentials and customer state must remain in protected test
directories. Never point this harness at a company production deployment.

The actual management CLI at `20f88e3` exported and restored that fixture with
its private deployment environment and TLS identity. Hash comparisons preserved
the profile-signing material, setup token, environment and TLS identity. The
restored SQLite database passed `PRAGMA quick_check`. The restored server passed
real HTTPS verification of signed configuration, administrator MFA, accounts,
device state, group capabilities, and denial of a revoked device and its session
grant. Private results are recorded in the restored test directory.

This does not demonstrate live hbbs/hbbr recovery, certificate renewal, recovery
of changes after the backup, profile revision reconciliation or migration
rollback. Those remain acceptance requirements.

The extended storage component test additionally preserves nested certificate
private-key and ACME account fixtures through encrypted export and restore.
It does not demonstrate a running Caddy service restoring or renewing certificates.

The extended HTTPS rotation rehearsal additionally passed actual CLI export and
restore of a rotated management database, deployment environment and the local
HTTPS PFX/certificate storage. Its replacement HTTPS proxy serves using the
restored identity; rotated profile trust, device credentials and closed-grant
denial survive. Both original/restored fixture processes stop afterward. This
remains isolated local TLS evidence; ACME renewal and live transport restore
are still unverified.

The Linux `test_container_stack.py` rehearsal now additionally restores actual
pinned RustDesk transport keys/database and Caddy storage, alongside management
and private deployment settings, from one encrypted archive. It starts replacement
management, rendezvous, relay and HTTPS containers on separate volumes. The
original CA still validates HTTPS, signed company configuration and transport
identity remain unchanged, and enrollment, consent, MFA replay protection,
logout and device revocation survive. The original deployment starts again and
retains its pending device despite revocation in the replacement. Private key
contents never leave the volumes. This supersedes the earlier local transport
restore limitation above; remote desktop sessions, public ACME renewal,
post-backup reconciliation and migration rollback remain unverified.
