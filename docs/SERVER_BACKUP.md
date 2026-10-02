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
The TLS argument accepts one existing identity file, such as a PFX or PEM bundle;
it does not export an entire ACME storage directory.

```text
swan-management backup company.swan-backup --transport-directory TRANSPORT_DATA --deployment-env PRIVATE_ENV_FILE --tls-identity TLS_IDENTITY_FILE
```

The transport directory must contain both `id_ed25519` and `id_ed25519.pub`.
When present, `db_v2.sqlite3` is also snapshotted using SQLite's backup API.
Management uses the same API for its database, including committed WAL data.
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
It permits fixed filenames only, rejects duplicate names and incomplete key
pairs, and protects restored files with private Unix permissions or Windows
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
