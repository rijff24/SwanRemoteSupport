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

## Linux candidate deployment and integration gate

`Dockerfile` builds the patched rendezvous/relay from the same immutable public
server and shared-submodule pins. The public recipe is pinned independently to
project revision `650361413543baf7ef150f5960544154ec77ddeb`; changing the current
checkout cannot silently change those transport inputs. Rust and both base
images are pinned, Cargo uses its verified lockfile, and the image contains
`/usr/share/doc/swan-transport/transport-build-source.zip`, the exact patched
source, recipe, receipt and AGPL notice. This establishes input provenance;
apt repositories and compiler output are not claimed bit-for-bit reproducible.

For a **fresh isolated test deployment**, explicitly select the override:

```sh
docker compose -f deployment/linux/compose.yaml -f deployment/linux/compose.maintained-transport.yaml up --build -d
```

Do not apply it to existing company volumes before backup/migration verification.
The normal deployment remains on its previously recorded component pins pending
candidate acceptance. No project registry image or production release is
published by this build. The company-component CI now also builds this candidate
and runs the existing real HTTPS, enrollment and encrypted complete backup/restore
rehearsal against its immutable local image ID. It validates the embedded public
recipe receipt before creating the rehearsal stack. These integration checks
still do not prove native direct/relay sessions or public reachability.

After a successful rehearsal, CI retains
`MaintainedTransport-Linux-x64-UNSIGNED-TEST`: both Linux x86-64 binaries,
`transport-build-source.zip`, `SWAN-SOURCE.json`, AGPL license and a hash manifest
binding them to the immutable image ID and packaging revision. Extraction uses
an owned stopped container, checks executable architecture and the archived UDP
patch, and runs no candidate executable. This artifact remains a test input;
it is not a signed release or a published production container image.
