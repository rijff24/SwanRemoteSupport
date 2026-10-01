# Local company-server testing

Each company runs its own management, rendezvous, relay and Windows build worker. The project does not operate a shared hosting or signing service.

Copy `local-test.env.example` to `local-test.env`, build `product/Cargo.toml`, then run `Start-LocalTest.ps1` from the repository root. The launcher reads named settings as data and binds management to loopback. `local-test.env` and `swan-data/` are ignored by Git. The data directory contains the database, private profile-signing key and setup token; its Windows ACL permits the current user, administrators and SYSTEM.

The default local administration address is http://127.0.0.1:18080. Use the one-time token in `swan-data/local-test/setup-token.txt` for initial setup. Do not paste the token into issues or build logs. Existing RustDesk services are independent of this test process.

This loopback HTTP listener is for administration smoke tests. Endpoint enrollment and profile synchronization require HTTPS with a trusted certificate; put a local TLS reverse proxy in front before testing those flows. Do not disable certificate validation. Public direct and relay testing requires a company-owned reachable endpoint and a separate external test machine.

Verified on this development machine: management starts, administration responds with HTTP 200, and the seven product-workspace tests pass. The native Flutter/RustDesk integration, production signatures, complete installers, automatic update execution and clean-machine Windows compatibility remain release requirements; this development build is not a production release.
