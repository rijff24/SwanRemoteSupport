# Candidate maintained transport

The Windows lab reproduced `WSAEFAULT` (10014) when sending an IPv4 sockaddr
through a dual-stack IPv6 UDP socket. Sending to the equivalent IPv4-mapped
IPv6 destination succeeded. The candidate patch normalizes that destination in
both shared `FramedSocket::send` and `send_raw`; IPv4 sockets, native IPv6
destinations and SOCKS transport keep their existing paths.

`source.json` pins the public server commit, shared submodule commit, upstream
Cargo lock, regression patch and implementation patch with input/output hashes.
No company profile supplies source, commands or patches. Existing installer
component pins and deployments are unchanged until candidate validation passes.

The `swan-transport.yml` workflow prepares a clean pinned checkout and adds the
regression first. Windows must reproduce error 10014 in the real Rust UDP path
while the IPv4 and IPv6 controls pass. Linux must pass the baseline. The workflow
then applies the implementation patch, requires all three network tests to pass,
and builds both executables using Rust 1.90.0 and `--locked`. Logs and results are
retained independently of build success. Candidate binaries and exact tracked
source, patched shared code and the public recipe are archived together.

These are unsigned test artifacts. Passing shared UDP tests does not prove an
authorized native session, Windows Server 2016 compatibility, Internet NAT
traversal or relay fallback. Validate those on the isolated lab before changing
distribution pins. Production publication still requires release approval,
valid signatures and corresponding public source; this workflow publishes no
production release and makes no claim of binary reproducibility.
