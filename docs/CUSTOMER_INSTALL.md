# Swan Remote Support customer installation

This release is for an installation performed by Swan Computing with the computer owner's authorization. Do not proceed if the customer has not agreed to the support scope and whether access will be attended or unattended.

1. Extract the entire release ZIP to a normal local folder.
2. Confirm Swan has published the same release version and SHA-256 on its HTTPS download page.
3. Double-click `Install-SwanRemoteSupport.cmd` and approve the Windows administrator prompt.
4. Complete the manual Tailscale login when the browser opens. Swan may need to approve the restricted customer-device tag in the Tailscale admin console.
5. Wait while the installer verifies the network, installs Swan, and creates a unique device credential.
6. Swan stores the one-time receipt in its password manager and clears the clipboard.
7. Confirm the Swan tray icon is visible and ask Swan to demonstrate how to stop or uninstall the service.
8. Test one authorized connection, restart Windows, and test again at the sign-in screen before considering installation complete.

The installer must show a valid trusted publisher. Do not disable Microsoft Defender, SmartScreen, Windows Firewall, or another antivirus product to make the installation succeed. Stop and contact Swan through its verified business channel if the signature is missing, the publisher is unexpected, the hash differs, Tailscale requests the wrong account/tailnet, or any security product reports the release.

Swan Remote Support is based on RustDesk and its corresponding source is linked from the release. The official Tailscale client is downloaded separately from Tailscale and remains a Tailscale product. See the public [privacy statement](PRIVACY.md), [security policy](../SECURITY.md), and [source/licensing information](SOURCE_AND_LICENSE.md).
