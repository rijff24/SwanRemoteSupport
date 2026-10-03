# Company deployment privacy information

This is the configurable product's technical privacy description. Each hosting company publishes its own support contact, consent wording, retention policy and customer request process before distributing its packages. The current development build is not a production release.

The company server stores administrator/technician accounts, password hashes, TOTP authentication secrets, device enrollment records, group permissions, authorization/session history, audit events and update/build metadata. Devices keep their unique enrollment credentials and signed cached company configuration. Installer bootstrap files contain public company endpoint and trust information only.

Customer approval is the default session mode. Unattended support requires separate explicit consent and company policy permission. Revocation is saved locally before server synchronization; the receiver must deny subsequent unattended access when consent is absent. Customer-visible support status, stop controls and uninstall remain acceptance requirements. Session recording is disabled by default.

An authorized remote session may expose screen content, input, clipboard contents, transferred files, audio and system information according to its permissions. Companies must explain the support scope to their customers. Technician authorization is verified by signed grants and receiving devices; a company download is not itself permission to access a device.

Configuration and trust changes use signed profiles over HTTPS. Cached company branding may be shown offline, but expired or unverifiable policy cannot authorize a new managed session. No project-hosted shared company service is supplied, and the new package flow does not install Tailscale or collect legacy device passwords.

Companies protect database/configuration/transport keys, enrollment credentials, TOTP seeds, signing material and backups. Private keys and customer records never belong in public source or installation bundles. Support contacts and deletion/retention procedures are the hosting company's responsibility. Complete backup, restore and deletion behavior must be verified before production use.

The historical Swan-only privacy and password-manager flow is preserved in the tagged baseline. See [the acceptance report](ACCEPTANCE.md) for controls that still need native or clean-machine verification.
