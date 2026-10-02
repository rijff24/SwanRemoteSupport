# Explicit company installer repair

Run `Install-Company.ps1 -Repair` from the company's installation bundle after
reviewing the company HTTPS hostname. Customer installation requires
administrator access. Technician installation uses the current user's local
application directory.

Repair requires a previously recorded installation and its signed
`installed-release.json`. The supplied release envelope must exactly match that
record, and its sequence must equal the durable installed sequence. The agent
verifies the company, edition, pinned signing key, release metadata expiry,
Windows compatibility, package hash and publisher before installation. Older
releases and a different package signed with the same sequence are rejected.
Pending automatic-update recovery must be resolved first.

After installation, the agent verifies installed endpoint and configuration
agent identities, then records the same sequence. Device enrollment validates
the existing device and transport identity; it does not issue new credentials
or change existing unattended consent. Technician setup restores the portable
application, local launcher and Start menu shortcut.

Automatic updates do not use the repair exception and still require an
increasing release sequence. An automatic update cannot reinstall an application
that was removed. Missing or expired repair metadata requires administrator
recovery or a newer approved company bundle; do not delete device state or lower
the recorded sequence to work around verification.

Component tests cover the repair metadata trust gate and ordinary update replay
rejection. Complete signed EXE/MSI repair, interruption recovery, session
coordination during setup, and clean-machine behavior remain acceptance work.
This document describes the development implementation, not tested production
compatibility.
