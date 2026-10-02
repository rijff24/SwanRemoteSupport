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
sessions. Logs remain in the private update folder. When the receipt is absent,
or uninstall cancellation is present, the task removes only its own verified
task registration and does not install software.

Explicit MSI uninstall writes `pending-uninstall` before file removal, blocking
new sessions and automatic recovery. Paired uninstall rollback restores the
previous cancellation state, including an earlier cancellation. Software
rollback narrowly uninstalls its verified failed MSI with `SWAN_RECOVERY=1`,
excluding user-uninstall cancellation actions. Only explicit verified setup or
repair clears the cancellation marker. A cancelled pending update receipt may
still require additional explicit setup/recovery handling; this is not yet a
verified uninstall/reinstall workflow.

For diagnostics, `Update-RecoveryTask.ps1 -InspectOnly` reads task ownership and
outputs a proposed identity without registering or removing a task. It does not
prove executable trust or successful registration. `test_update_recovery_task.ps1`
tests pure ownership rejection and cancellation files in an isolated temporary
directory, without native task changes or installer execution.

## Verification boundary

Windows agent/worker compilation, agent component tests, read-only task planning,
Scheduled Task object creation and ownership/cancellation regressions pass.
Unsigned customer and technician packaging rehearsals using historical 1.4.9
native inputs compile; read-only MSI tables confirm rollback/cancellation/file
removal order and upgrade/recovery exclusions. These are authoring checks, not
the current native release or successful installation. Actual signed task
registration, startup/logon execution, reboot between removal and restoration,
uninstall/reinstall, task cleanup, customer EXE rollback and the full supported
Windows matrix remain acceptance gates.
