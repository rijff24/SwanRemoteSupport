# Unattended consent synchronization

The receiving device's durable consent is required for unattended access in
addition to company policy and a valid server grant. Revocation is written
locally before attempting HTTPS synchronization. An offline server cannot
prevent local revocation.

The customer agent's `watch` loop retries a disabled local consent choice every
five minutes, after reloading the latest saved state. It never enables consent
in the background. A pending device's request may be rejected until company
approval; local unattended access remains disabled.

`PUT /api/v1/device/consent` requires an authenticated device and JSON containing
`unattended` (boolean) and `revision` (nonnegative integer within SQLite's signed
integer range). The server records the revision transactionally with the choice.
Lower revisions receive HTTP 409. Equal revisions are idempotent, with disabled
consent taking precedence over a conflicting enable request. Repeated unchanged
requests do not create audit events.

Explicit enable requests propose the next local consent revision and must be
accepted by the server before persisting enabled consent. Persistence rejects
an enable response if another process changed consent while it was pending.
The customer must confirm again in that case. A later revocation is durable
locally and its retry cannot be overwritten by that older enable response.

This changes the development API: older clients sending no revision are
rejected rather than permitted to bypass ordering. Upgrade the server and
endpoint components together before testing this development build. This is
not a production compatibility promise. The additive consent-revision table
is included in management database backups.

Component tests cover ordering, same-revision conflicts, idempotency, and stale
local enable responses. Offline network recovery and native Windows controls
still require end-to-end acceptance tests.
