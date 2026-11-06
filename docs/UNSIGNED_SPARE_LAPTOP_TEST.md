# Unsigned two-laptop test

> Historical test procedure for `swan-single-company-baseline-1.4.9`. Its Tailscale and password steps apply only to the preserved single-company deployment. For the configurable company-hosted product, use the [product guide](../product/README.md) and [acceptance report](ACCEPTANCE.md).

This procedure is only for an isolated spare laptop owned or controlled by Swan Computing. The current 1.4.9 build is unsigned. Do not use it on a real customer computer, do not place customer data on the spare laptop, and do not weaken Microsoft Defender, SmartScreen, Windows Firewall, or another security control.

## Before starting

1. Keep the Swan server `desktop-7egpge2` online and connected to Tailscale.
2. Connect the technician laptop `rijff-thinkbook` to Tailscale. Confirm that it has only the `tag:swan-operator` tag.
3. Use a separate spare laptop as the simulated customer computer. Remove any old Swan test installation before starting if one is present.
4. Extract the entire test-kit ZIP to a normal local folder on both laptops. Do not run a file from inside the ZIP.
5. Verify the test-kit ZIP against the adjacent `.sha256.txt` file before extracting it.

## Simulated customer laptop

1. Open the Tailscale Machines page and identify the spare laptop by its Windows computer name. Assign only `tag:swan-customer` to that machine. Never assign the operator or server tag to it.
2. Run `Install-SwanRemoteSupport-UNSIGNED-TEST.cmd` from the extracted folder.
3. Accept the unsigned-test warning only on this spare laptop. When the consent prompt appears, type `YES` because you own or control this test machine.
4. If Tailscale opens a browser login, sign in to the intended tailnet. Approve or assign `tag:swan-customer` in the Tailscale admin console if requested. The installer waits for the exact tag before continuing.
5. When setup completes, paste the one-time credential receipt directly into the Swan password manager. Do not paste it into chat, email, this folder, or a screenshot. Clear the Windows clipboard after saving it.

## Technician laptop

1. Run `Run-Swan-Technician-UNSIGNED-TEST.cmd` from the extracted folder. It creates a byte-for-byte portable copy whose filename starts the technician interface instead of the one-page installer.
2. Use the numeric Swan Remote Support ID and unique password from the password manager to connect to the spare laptop.

## Required checks

1. Confirm the spare laptop receives a numeric Swan Remote Support ID.
2. Confirm the operator laptop can connect and control the spare laptop using its unique password.
3. Restart the spare laptop and reconnect at the Windows sign-in screen.
4. On the spare laptop, confirm Tailscale shows only `tag:swan-customer`. On the technician laptop, confirm only `tag:swan-operator`.
5. Confirm the customer-tagged spare cannot initiate a connection to the operator laptop. The operator-to-customer direction should work; the reverse direction should not.
6. Stop Tailscale on the spare laptop, then confirm the support connection fails closed. Start Tailscale on the spare again afterward; do not stop the Swan server for this check.
7. Record only pass/fail results and non-sensitive error text. Never record the unattended password, the server private key, browser login links, or Tailscale authentication material.

## Finish

Do not treat this unsigned build as customer-ready even if all tests pass. Remove the spare laptop from the tailnet or remove its customer tag when testing is complete, uninstall the test client if it is no longer needed, and keep the test credentials only as long as the test requires.
