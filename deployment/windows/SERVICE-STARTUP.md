# Managed customer service startup

The managed Windows launcher connects to the Service Control Manager before
loading `librustdesk.dll`, reading company state or entering the Rust runtime.
Microsoft requires prompt dispatcher/handler registration and explicit status
reporting during initialization; see
[ServiceMain initialization](https://learn.microsoft.com/en-us/windows/win32/services/service-servicemain-function)
and [service state transitions](https://learn.microsoft.com/en-us/windows/win32/services/service-status-transitions).

The service name remains `Swan Remote Support`, independently of company display
branding. The launcher handles an exact `--service` invocation. Desktop arguments
continue through the existing desktop entry point; ambiguous service arguments
are rejected. The upstream launcher path is retained when `SWAN_BUILD` is unset.

Initialization reports StartPending with a 15-second wait hint and checkpoints
every five seconds. A heartbeat thread is confirmed running before DLL loading,
because DLL initialization can otherwise hold the loader lock and prevent a
new thread from entering its procedure. Initialization is bounded to 120 seconds.
If that bound expires, the service reports a stopped/error state and terminates
its own initializing process. No desktop or authorized session has started at
that point. The global Windows service timeout is not changed.

Only the core DLL beside the installed executable is selected, with application
DLL-directory and System32 dependency search. Both new service exports must be
present; a legacy core is rejected rather than starting another dispatcher.
The core remains loaded through process exit to avoid unloading while a final
control callback is in flight.

The Rust service uses the existing managed defaults and receiving-device
authorization. Its normal service loop reports Running/Stopped through the
launcher's status callback, and its existing control handler receives stop and
interrogate requests. Running denotes the service process entering that loop;
remote availability still requires a usable receiver, approved enrollment,
valid signed policy and a valid session grant. This bootstrap does not authorize
connections, change consent, reset identity or enable an upstream password path.

`test_service_bootstrap.ps1` tests the actual Windows service manager with isolated
simulated DLLs. Cases cover normal startup/control/stop, 35-second DLL loading
with pending checkpoints, absent DLL/exports, bounded loader timeout and command
argument handling. These tests establish the launcher boundary, not native Rust
integration, signing, cold-boot reliability or full product acceptance. Complete
native builds and enrolled-machine cold-boot tests remain required.
