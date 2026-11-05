# Check a company's public endpoints

Each company runs its own server and relay. Swan Remote Support supplies no
shared connectivity service.

In the administration interface, open **Network**, run **Check saved endpoints**,
and download **Network configuration for outside checks**. This file contains
public company information and trust keys. The `bootstrap.json` from a company
installer bundle works too.

Copy that file and the matching `swan-management` executable to a computer on a
separate Internet connection. Run it as an ordinary user:

```powershell
.\swan-management.exe check-network .\company-network-bootstrap.json --outside-network
```

On Linux:

```sh
./swan-management check-network ./company-network-bootstrap.json --outside-network
```

For a company using its own certificate authority, add `--ca-file company-root.crt`.
The checker still validates the HTTPS certificate, hostname, pinned profile key,
company identity and profile lifetime. Obtain fresh public configuration after
company endpoint or signing-key rotation.

The command prints JSON and exits successfully when its requested checks pass.
It verifies management HTTPS and TCP, the NAT-test/rendezvous/relay TCP ports, and
a native rendezvous UDP rejection response for a deliberately empty device ID. It uses no company account, device
enrollment, server data directory or private signing key. It creates no server
listener. The pinned transport rejects this diagnostic request before device lookup or storage; it registers no device. Native UDP responses are connectivity diagnostics, not authentication.

`--outside-network` records the operator's stated test location and additionally
requires the connected addresses to be public candidates. The location is not
independently attested. Private or special-use routes cannot pass that outside
check. Without this flag the same command can diagnose a private deployment;
those results describe connectivity from that computer only.

Before publishing customer downloads, also complete real customer-approved
sessions from the separate connection, testing direct traversal and relay
fallback. A successful port check does not qualify those sessions or every
HTTPS-only corporate proxy. Office hosting may need port forwarding; CGNAT may
require a company-controlled public relay. The JSON report identifies which
endpoint failed so the company can correct DNS, certificates or forwarding.
