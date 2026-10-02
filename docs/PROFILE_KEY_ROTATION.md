# Company profile signing-key rotation

These commands operate on a company's private management data directory. They
do not rotate RustDesk transport keys or software release-signing keys. Keep
`SWAN_DATA_DIR` and `SWAN_BACKUP_PASSPHRASE` in private environment configuration.
Stop the management service before either command. Upgraded servers hold an
exclusive data-directory lock; older server versions must also be stopped.

1. Create an encrypted deployment backup and retain its passphrase separately.
2. Run `swan-management prepare-profile-key`. This generates a private pending
   key in SQLite and advances the profile revision with its public replacement
   key. The active signing key stays unchanged. The command prints public
   information only.
3. Restart management. Installed customer and technician apps must synchronize
   this transition while the old key still signs it. Verify synchronization on
   the intended devices, including technician installations, before activation.
   A machine that misses this transition cannot independently trust the new key;
   it needs explicit trusted reconfiguration. The server currently cannot prove
   that every installation received the transition.
4. Stop management again and run
   `swan-management activate-profile-key --confirm-clients-synced`.
   Activation checks the pending public key matches the published transition,
   then atomically changes the active private key and advances the profile
   revision in SQLite. The next process loads this key. Outstanding session
   grants close, and old installer jobs/downloads become unavailable. Existing
   device credentials, company identity, consent and release trust stay intact.
5. Restart management. Verify the public key through its HTTPS status endpoint
   and verify synchronized clients accept the new signed profile. Generate new
   company installer bundles and replace website/internal distribution links.
   Previously downloaded old bundles must also be withdrawn by the operator.

Activation invalidates current sessions as their authorization is revalidated.
Schedule rotation as a maintenance operation. This does not create an automatic
trust reset or permit a profile signed by an unrelated key.

Schema version 2 stores active and pending profile-signing keys in the protected
management SQLite database. `profile-key.hex` remains the initial seed for
legacy migration; it no longer identifies the active key after rotation. Do not
edit that file to rotate keys. Encrypted backups must include the database.
Restoring the database preserves active/pending keys and their profile revisions.

When opening a version-1 management database, the upgraded server requires
`SWAN_BACKUP_PASSPHRASE` and creates an encrypted `pre-upgrade-v1-*.swan-backup`
before migration. It refuses migration without that password. Version-1 server
binaries refuse the version-2 database; downgrade recovery uses the matching
pre-upgrade backup in an empty directory, with public endpoints unavailable.
Reconcile all changes made since that backup before exposing a restored server.

Component tests cover process exclusion, authorized transition checks, restart,
new-key signatures, encrypted restore and pre-upgrade backup recovery. Real
HTTPS rotation across installed apps, offline machines, interrupted activation,
live session termination and complete server migration rollback remain release
acceptance requirements.

## Local HTTPS rehearsal

`node deployment/windows/test-profile-rotation.js PRIVATE_ENV BINARY_DIRECTORY`
owns fresh management and HTTPS proxy processes for a marked isolated fixture.
It requires `rotation-fixture.marker`, a fresh data directory without a database,
private TLS configuration and debug management/agent binaries. Use the local
test certificate mechanism; do not import a test CA into production trust or
point this harness at an existing company. All credentials/results stay in the
protected fixture directory. Processes stop on completion or failure.

The rehearsal passed with binaries built from `7843334`: real HTTPS customer and
technician agent sync adopted the prepared key, retained enrollment and revoked
consent, rejected a client that missed the transition, required activation
confirmation, and preserved the active key through restart. A claimed grant was
still unexpired when activation denied its renewal. This tests built agents and
the management authorization API; it does not prove the graphical receiving app
terminates a live remote session or that production HTTPS deployment passes.
