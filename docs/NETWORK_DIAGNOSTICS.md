# Company network diagnostics

The administration interface's Company network screen publishes the management
HTTPS URL, rendezvous/relay endpoints and public transport trust key through the
signed company profile. DNS, HTTPS provisioning, transport processes and firewall
configuration must match those settings. Publishing a URL does not move the
server or configure a reverse proxy.

The Check saved endpoints button calls administrator-only
`POST /api/v1/network/check`. It probes only the saved profile; request-body
addresses are not accepted. Checks resolve rendezvous/relay DNS, connect to the
ID TCP port, its preceding NAT-test TCP port, and the relay TCP port, and fetch
the public management profile over certificate-validated HTTPS. The returned
profile must validate against this company's signing key, identity and revision.
The operation is rate-limited and audited.

Results explicitly identify `company_server` as the vantage point and never
claim external or UDP reachability. Private/local/shared-address classifications
are diagnostic hints. A public-looking DNS address and successful TCP connection
from the hosting server do not prove that clients elsewhere can reach it.

Run actual native direct and relay tests from an outside network before
distribution. Office hosting needs suitable port forwarding; CGNAT may require
a company-controlled public relay. Test UDP on the ID port separately. No project
VPS is assumed and compatibility with every HTTPS-only proxy is not promised.

Component tests cover a real loopback TCP listener/closed port, address parsing,
zero-port rejection and administrator access restrictions. Full external
reachability, native transport trust, NAT traversal and relay behavior remain
acceptance requirements.

A fresh-company HTTPS lifecycle run at `d6f1665` also passed against actual built
agent and management executables. It verifies the management HTTPS certificate
and company signature, rejects unauthenticated checks, ignores request-body
targets in favor of saved configuration, and reports external/UDP reachability
as unverified. The debug server uses an explicit local DER certificate through
the private `SWAN_TEST_CA_FILE` setting; release builds ignore this test setting.
Exact executable hashes and clean source state are recorded privately. Its test
transport ID is not a native remote-session test.
