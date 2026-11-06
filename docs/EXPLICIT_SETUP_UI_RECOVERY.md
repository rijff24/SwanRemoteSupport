# Explicit customer setup and UI recovery

The native1.5.18 lab exposed an explicit MSI upgrade that verified its new files
but left the customer's old window mapped into MSI rollback storage. That window
lost its company configuration. The original failed test remains in
[the acceptance evidence](evidence/1518-explicit-customer-upgrade-open-ui-failure-20261103.json).

Explicit customer preparation now reuses the automatic updater's durable,
limited-user UI recovery. The agent holds exclusive session access, verifies the
requested package, and writes the signed pending-install marker before closing
an existing window. It copies only its exact release-pinned agent into canonical,
protected customer storage and verifies that copy's publisher before a fixed
SYSTEM task executes it. Company profile fields cannot supply commands or paths.

The task broker validates the complete action, principal and task permissions.
It continues an existing running attempt instead of starting a duplicate, and
requires a completion receipt bound to the installation transaction, operation,
marker and agent hashes. An old success receipt cannot complete a new attempt.
A bounded timeout preserves the task and installation marker for recovery.

After installed files and MSI registration are verified, recording restores the
original users' windows through their existing limited interactive tokens. It
does not collect passwords or start a user GUI as SYSTEM. The setup marker stays
in place if restoration fails, including after installation metadata commits;
retrying the exact verified setup can finish that transaction. Uninstall cancels
UI restoration and removes validated broker tasks before removing user restart
tasks. Completed helper files remain private with the existing recovery cache.

The shared UI helper still requires SYSTEM. Explicit repair can prepare a missing
executable only for a previously recorded installation and still checks each
process's path, creation time and owner. An unrecorded existing executable must
match the verified target; other existing software requires an explicit migration.

The Windows component test uses an inert executable in empty, guarded company
storage on a disposable runner. It tests actual SYSTEM execution, task tampering,
hash and receipt rejection, timeout continuation and uninstall cleanup. It does
not qualify signing or the real installer. The signed upgrade and same-release
repair must still be repeated with an already-open standard-user customer window,
and active-session, interruption, uninstall and Windows-version tests remain
release gates until their actual results are recorded.
