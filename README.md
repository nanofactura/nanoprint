<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/logo-nanoprint-dark.svg">
    <img src="assets/brand/logo-nanoprint.svg" alt="NanoPrint" width="320">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/nanofactura/nanoprint/actions/workflows/ci.yml"><img src="https://github.com/nanofactura/nanoprint/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/nanofactura/nanoprint/actions/workflows/codeql.yml"><img src="https://github.com/nanofactura/nanoprint/actions/workflows/codeql.yml/badge.svg" alt="CodeQL"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--only-blue" alt="License: GPL-3.0-only"></a>
</p>

NanoPrint is an open-source print agent for thermal receipt printers. It lives in the system tray
and lets web applications print raw ESC/POS jobs, with a permission prompt per site instead of
signing certificates. It is compatible with qz-tray.js clients.

> **Status:** early development, no release yet. Printing to printers installed in the operating
> system works over `ws://` and `wss://` (tested on Windows); see the
> [roadmap](docs/architecture.md#roadmap).

## Highlights

- Works with existing qz-tray.js clients and libraries such as `web-escpos-printer`.
- Sites ask for permission once; the user approves or blocks them and can revoke access later.
- Listens only on the local machine. Opt-in LAN access with device pairing is planned.
- Small footprint: a Rust core with native webviews (Tauri 2) instead of a bundled browser.
- English and Spanish UI that follows the system language.
- Windows, macOS and Linux, with signed automatic updates (the AppImage on Linux).

## Installing

Download the installer for your system from the
[latest release](https://github.com/nanofactura/nanoprint/releases/latest). NanoPrint installs for
the current user only, without administrator rights. It starts with the system, quietly in the tray
(you can turn that off in Preferences); opening it yourself shows the welcome the first time and the
panel afterwards.

Installers are not code-signed yet, so the system warns on first launch:

- **Windows:** "Windows protected your PC" → **More info** → **Run anyway**.
- **macOS:** open NanoPrint once, then **System Settings → Privacy & Security → Open Anyway**.
- **Linux:** `.deb`, `.rpm` or AppImage. Enabling secure sites uses `certutil` (`libnss3-tools` or
  `nss-tools`, installed with the `.deb` and `.rpm` packages).

You can check that a download was built by this repository with
`gh attestation verify <file> --repo nanofactura/nanoprint`.

To uninstall on Windows, use **Settings → Apps** and tick "Delete the application data" to also
remove its certificate and keys. On macOS and Linux, run `nanoprint --uninstall` before deleting the
app.

### What NanoPrint changes on your computer

- **Start with the system:** a startup entry for your user (Windows registry `Run` key, macOS
  LaunchAgent, Linux XDG autostart). Turn it off in Preferences.
- **`nanoprint://` links:** a handler so web pages can start NanoPrint when it is closed; the
  browser asks you first, and the link can do nothing else.
- **Secure sites**, only when you enable them and confirm in the system's dialog: NanoPrint's own
  certificate authority in your user's trust store (and in Firefox's on Linux).
- **Its own data:** the key that signs your permissions, in the system credential store; settings,
  permissions, history and logs in your user profile.

The Windows uninstaller removes the startup entry and the link handler, and, with "Delete the
application data", the certificate, the key and the data. On macOS and Linux, `nanoprint --uninstall`
removes the certificate and the key; turn off "Start with the system" first.

## Code signing policy

Windows releases are going to be signed through the SignPath Foundation program for open-source
projects (application in progress; current releases are not signed yet): free code signing provided
by [SignPath.io](https://about.signpath.io/), certificate by
[SignPath Foundation](https://signpath.org/).

- Every installer is built by GitHub Actions from this repository's source; a maintainer approves
  each signing request.
- Committers and reviewers: Mariano Paulin ([@marianopaulin02](https://github.com/marianopaulin02)).
- Approvers: Mariano Paulin ([@marianopaulin02](https://github.com/marianopaulin02)).
- Privacy: see [Privacy](#privacy). NanoPrint does not transfer any information to other networked
  systems unless specifically requested by the person using it. The one exception, from version
  0.2.0, is checking GitHub for updates, which can be turned off in Preferences.

## Printing from your web application

Use [qz-tray.js](https://www.npmjs.com/package/qz-tray) to connect and send raw jobs, with an
ESC/POS encoder such as
[`@point-of-sale/receipt-printer-encoder`](https://www.npmjs.com/package/@point-of-sale/receipt-printer-encoder),
or [`web-escpos-printer`](https://www.npmjs.com/package/web-escpos-printer) with its QZ Tray
transport. Sites can also send a name and logo for NanoPrint's permission prompt. See the
[integration guide](docs/integration.md).

## Development

Prerequisites: Node.js 24, Rust (stable) and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system.

```sh
npm install
npm run tauri dev
```

Quality checks: `npm run format`, `npm run lint`, `npm run check`, `npm test`, and for Rust
`cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
(run `npm run build` first). See [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request,
[CLAUDE.md](CLAUDE.md) for the project rules, [docs/architecture.md](docs/architecture.md) for the
design and [docs/releasing.md](docs/releasing.md) for releases. The website lives in
[landing/](landing/README.md).

## Privacy

NanoPrint has no telemetry, analytics or crash reports. The only connection it makes on its own
is checking GitHub Releases for a newer version, which, like visiting any web page, shows GitHub
your IP address; turn it off in Preferences → Updates, and NanoPrint connects to nothing. Web
pages reach it only on this computer. Its history (which site
printed, when and on which printer, never the content) and its diagnostic log stay on the computer
and are deleted after 30 days.

## Security

Please report vulnerabilities privately, as described in [SECURITY.md](SECURITY.md). The
[threat model](docs/threat-model.md) explains what NanoPrint protects and how.

## License

Copyright © NanoFactura®. NanoPrint is free software: you can redistribute it and/or modify it
under the terms of the [GNU General Public License, version 3](LICENSE) (GPL-3.0-only).

QZ Tray is a trademark of QZ Industries, LLC. NanoPrint is an independent project, not affiliated
with or endorsed by QZ Industries; it mentions QZ Tray only to describe compatibility.
