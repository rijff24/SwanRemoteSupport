# Durable endpoint update recovery

Before launching an update helper, the agent registers a separate scheduled task
pointing to the exact hash/publisher-verified helper under
`updates/<release-sequence>/swan-agent.exe`. It stores no password, technician
token, device token or grant in task arguments. The only argument is
`recover-update-task`; the helper derives the company state directory from its
own staged location rather than inherited environment settings.

Customer recovery runs as SYSTEM at startup and every five minutes. Technician
recovery runs as the current user, with limited privileges, at logon and every
five minutes. Tasks use a per-directory, edition and sequence name. Registration
or deletion refuses unexpected task paths, actions, arguments or principals.
Failed registration stops handoff before installation. The staged folder is
outside the MSI-owned installed files; customer setup protects its state tree
with SYSTEM/administrator ACLs.

On each attempt, the staged helper checks its location, the signed pending
receipt, edition, sequence and its own hash/publisher. It resumes only that exact
release or its durably prepared rollback. Public artifact retries send no
credentials. Existing activity locks prevent concurrent installation or support
sessions. Logs remain in the private update folder. A customer update writes
signed `configuration-restart.json` before committing its release. If installation
committed but the watcher restart failed, its recovery task verifies the exact
installed release, payload and MSI registration, then retries the restart under
the activity lock. The marker blocks sessions and another automatic update until
restart succeeds. An older release's recovery task cannot restart a newer release.
When both recovery markers are absent, or uninstall cancellation is present, the
task removes only its own verified task registration and does not install software.

Before replacing customer files, the helper stops only the validated installation-
owned SYSTEM configuration task, releasing the previous agent executable. MSI
completion additionally requires the signed package's product code and version
to be registered with Windows Installer. Matching executable hashes alone do not
prove that an MSI with unchanged binaries installed. Explicit verified setup or
repair checks the same registration and takes responsibility for registering its
watcher before retiring an earlier restart marker.

Explicit MSI uninstall writes `pending-uninstall` before file removal, blocking
new sessions and automatic recovery. Paired uninstall rollback restores the
previous cancellation state, including an earlier cancellation. Software
rollback narrowly uninstalls its verified failed MSI with `SWAN_RECOVERY=1`,
excluding user-uninstall cancellation actions. Only explicit verified setup or
repair clears the cancellation marker. Explicit setup now verifies its incoming
package again under the activity lock, publishes a setup marker without
overwriting a different interrupted setup, and archives the exact original
cancelled receipt at `updates/<sequence>/cancelled-update.json`. An unchanged
archive permits interrupted preparation to retry; a conflicting archive fails
closed. Cancellation and setup markers keep sessions blocked until full installed
identity verification completes. A setup interrupted after committing its exact
installed metadata may finish only that same signed setup; ordinary sequence
replay rejection resumes when the marker is gone. This implements the transition
but does not yet demonstrate native uninstall/reinstall.

For diagnostics, `Update-RecoveryTask.ps1 -InspectOnly` reads task ownership and
outputs a proposed identity without registering or removing a task. It does not
prove executable trust or successful registration. `test_update_recovery_task.ps1`
tests pure ownership rejection and cancellation files in an isolated temporary
directory, without native task changes or installer execution.

The company bundle includes the fixed read-only MSI mode inspector. Setup selects
repair for the exact already-registered product and installation for an unknown
product. Both the updater and setup resolve Windows Installer from the OS system
directory. This does not authorize another edition, publisher, version or package:
signed metadata, artifact identity and package checks still precede installation.

## Verification boundary

Customer EXE rollback now reinstalls only the retained previous hash/publisher-
verified package after complete snapshot and previous-agent capability checks.
It rejects an MSI upgrade-family registration, MSI product marker, wrong
uninstall registration, or another installation's service. The restored service
must run automatically; installed endpoint/agent files verify before metadata
and failed-release quarantine are published. Identity, keys and consent remain
current. Explicit customer EXE uninstall runs a fixed encoded cancellation
script inside its elevated uninstall batch before service/file removal, stopping
only its owned configuration task. Recovery rechecks cancellation before
completion.

Native service/process command generation now quotes stable names containing
spaces. SYSTEM installation uses the OS command processor directly. Silent
portable packages wait for the extracted install process and propagate its exit
status; GUI launch keeps its existing asynchronous behavior. The standalone wait
module's Windows child lifecycle tests pass, including failed exit and missing
executable handling. Native app/packer syntax parses, but current full native
compilation and signed installer execution have not yet passed.

Windows agent/worker compilation, agent component tests, read-only task planning,
Scheduled Task object creation and ownership/cancellation regressions pass.
Unsigned customer and technician packaging rehearsals using historical 1.4.9
native inputs compile; read-only MSI tables confirm rollback/cancellation/file
removal order and upgrade/recovery exclusions. These are authoring checks, not
the current native release or successful installation. Actual signed task
registration, startup/logon execution, reboot between removal and restoration,
uninstall/reinstall, task cleanup, customer EXE rollback and the full supported
Windows matrix remain acceptance gates.
