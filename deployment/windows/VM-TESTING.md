# Isolated Windows acceptance lab

Use official Microsoft evaluation media, within its evaluation period. Never
disable activation checks or redistribute evaluation images with project releases.
Media, VM disks, local credentials, screenshots and raw test logs stay outside Git.

Choose the local lab, QEMU and ISO paths in a private local environment file.
The initial resource budget is one guest at a time, 4 GiB RAM, two virtual CPUs
and a sparse 64 GiB disk. User-mode
NAT does not change the host bridge, firewall or existing RustDesk services.
VNC (port 5920) and the QEMU monitor (4444) listen on loopback only. These sockets
provide full guest access; close QEMU when the guest is not under test.

`New-TestVm.ps1` requires an already verified QEMU installation and ISO. It
refuses existing VM directories. Prefer WHPX after checking it starts on the
host; TCG is a slower fallback. The host must retain 1.5 GiB available memory.
The scripts select QEMU's `max` CPU model. A local QMP capability probe confirmed
SSE4.2 and POPCNT are exposed by that model and absent from the default `qemu64`
model. Server 2025 requires these instructions; this supplies supported emulated
instructions rather than bypassing the operating system's hardware checks.
`New-LocalTestVm.ps1 -EnvironmentFile PATH_TO_PRIVATE_ENV` reads the six
`SWAN_VM_*` settings from the private environment file without executing them.
It requires `ISO_PATH.download-complete.json` containing `bytes` and `sha256`,
recorded only after the official HTTPS download exits successfully. It verifies
the recorded size and hash before creating a guest. This record proves local
download integrity; it does not replace verification of the Microsoft origin.
Do not write the completion record while a downloader is still running.
For a preserved guest, add `-Resume` and set `SWAN_VM_ACCELERATOR=tcg` in the
private environment file to try the software-emulation fallback. The resume
helper verifies the ISO again, retains the existing disk, reconstructs fixed
loopback-only QEMU options, and refuses a disk already in use. TCG performance
and successful Windows installation still require actual verification.
Windows 11 also requires supported UEFI/Secure Boot and TPM configuration:
the generic starter script is for Server guests and does not establish Windows
11 compatibility. Do not bypass Windows 11 hardware requirements for acceptance.

Choose **Desktop Experience** for every Server installation. Record edition,
build, patch level, ISO origin and SHA-256, accelerator and product commit.
Keep a clean installation checkpoint before testing each package.

## Required matrix

Windows 10 and 11 x64; Windows Server 2016, 2019, 2022 and 2025 Desktop Experience.
Each row remains **unverified** until actual installation and upgrade evidence
exists for that guest. Unit tests and host installation are insufficient.

For each guest, capture installation, repair, restart, uninstall, branding sync
and offline cache, attended approval, unattended opt-in/revocation, authorized
and denied access, UAC/logon screen, clipboard, transfer, multiple monitors,
interrupted upgrade recovery and identity preservation. Test direct and relay
connections between separate guests. Public Internet traversal needs a separate
external network; local NAT success does not prove public reachability.

Official media starting points:

- <https://www.microsoft.com/en-us/evalcenter/evaluate-windows-11-enterprise>
- <https://www.microsoft.com/en-us/evalcenter/evaluate-windows-server-2025>
- <https://www.qemu.org/download/>

Evaluation expiration or missing supported media must be reported as an
unverified matrix row, never as a passing compatibility test.

The first local WHPX probe reported `Unexpected VP exit code 4`; the isolated
guest was stopped. Its sparse disk is preserved outside Git. No operating
system installation or accelerator compatibility is established by that probe.
The official Server 2025 ISO subsequently downloaded successfully and its size
and SHA-256 were recorded privately. A TCG guest reached the Windows boot loader.
The initial TCG attempt used the insufficient default CPU and was restarted
with the corrected model. No completed Windows installation is recorded yet.
