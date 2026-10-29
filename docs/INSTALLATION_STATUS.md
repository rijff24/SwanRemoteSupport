# Installation versions and update reports

Work in progress: the management API, shared message contract and administration
page are implemented. Automatic customer/technician senders and update-engine
integration are not yet connected. Older installed apps therefore produce no
reports. This is not completed rollout or native update acceptance.

Reports go only to the configured company management server over its existing
authenticated HTTPS connection. They contain installation identity, a monotonically
increasing report revision, reported installed version/release sequence, update
phase, optional target sequence and bounded byte progress. They contain no login
passwords, MFA seeds, enrollment tokens, screen content or local file paths.

Customer reports require an approved device credential and use that device's
server-assigned identity. Technician reports require a current MFA-authenticated
session and are bound to that account plus a persistent installation UUID.
An installation's revision cannot go backward; a repeated identical message is
idempotent and does not refresh the server's received time. Conflicting messages
at the same revision fail. The authenticated owner can retrieve its previous
report to recover the next local revision after an interrupted write.

Administrators can read all reports. Technicians can read reports from their own
technician installations and approved devices in their authorized groups. Device
revocation, group removal and account disablement take effect through the same
current permission queries used for inventory. The list is paginated, and the
Update status page refreshes every ten seconds while open. Leaving the page or
signing out stops refresh; an earlier response cannot repopulate the signed-out
page. Reports older than fifteen minutes are marked as old, without claiming the
device is disconnected.

Status is informational and self-reported. It never approves a device, grants
support access, records unattended consent, approves a release or proves the
installed bytes are authentic. The updater must continue checking signed release
metadata, hashes, trusted publishers, policy and session exclusion independently.
“Installed” means the recorded version, not necessarily the latest company release.

## Management storage upgrade

The report table uses database schema3. A schema1/2 upgrade requires
`SWAN_BACKUP_PASSPHRASE` in private process environment configuration. Management
first exports an encrypted `pre-upgrade-vN-*.swan-backup`, then applies schema and
key initialization in one transaction. A failed migration rolls back its DDL.
Fresh companies start directly at schema3. Existing deployments still need their
coordinated full deployment backup, including installer/TLS/transport data.

To roll back to a schema2 server, stop management, use the retained new management
binary to restore the pre-upgrade archive into an empty replacement directory,
and only then run the older binary against that restored schema2 data. Older
binaries cannot consume the SWANBK2 envelope or safely open a schema3 database.
Preserve the original upgraded data separately, keep public endpoints unavailable
and reconcile revocations/consent changes made after the backup before exposure.
The pre-upgrade archive preserves the original schema; it does not merge later
changes automatically. See [backup and restore](SERVER_BACKUP.md).

## Verification boundaries

HTTP component tests cover owner binding, role/group filtering, report replay,
revocation/disablement, pagination, foreign-company credentials and the fact that
status writes cannot create grants or consent. Migration tests cover backup
requirements, original schema/key recovery and rollback of failed DDL. Browser
contract checks cover escaped names, progress display, pagination, refresh lifetime
and late responses after sign-out. These do not replace real HTTPS reporting,
native installation/update phase tests, restart recovery or upgrade compatibility.
