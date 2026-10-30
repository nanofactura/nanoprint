# Changelog

Notable changes to NanoPrint. The format follows [Keep a Changelog](https://keepachangelog.com/),
and versions follow [Semantic Versioning](https://semver.org/).

> QZ Tray and qz-tray.js are products of QZ Industries, LLC; QZ Tray is its trademark. NanoPrint is an independent project, not affiliated with or endorsed by QZ Industries, and mentions them only to describe compatibility.

## [Unreleased]

### Added

- Network printers: add a printer connected by cable or Wi-Fi in the Printers tab, by IP address
  or host name and port (9100 by default), with no driver. Sites see it next to the system's
  printers and print to it by name, or by `{host, port}` when it matches one the user added;
  NanoPrint never connects to an address a page chose. Jobs go one at a time per printer.

## [0.2.1] - 2026-10-30

### Security

- HTTPS connections, such as the update check, reject certificate chains that pass through the
  distrusted emSign intermediates (rustls-webpki 0.103.16, RUSTSEC-2026-0382).
- Constant-time helpers used by the hash functions no longer leak timing in `u128` comparisons
  (ctutils 0.4.5, RUSTSEC-2026-0385).

### Changed

- Updated Tauri to 2.12.5, the updater plugin to 2.14.0 and other dependencies.

## [0.2.0] - 2026-10-16

### Added

- Automatic updates: NanoPrint checks GitHub Releases every six hours, verifies each update's
  signature and installs it once no job has printed for five minutes. Preferences → Updates turns
  it off, checks on request and installs at once. Works on Windows, macOS and the Linux AppImage.

### Changed

- The welcome says NanoPrint starts with the computer and where to turn that off.

## [0.1.0] - 2026-10-16

First public release.

### Added

- Tray app for Windows, macOS and Linux with a settings panel in English and Spanish that follows
  the system language, and start with the system on by default. Started with the system it stays
  quietly in the tray; opened by the user it shows the welcome the first time and the panel after.
- Print server compatible with qz-tray.js clients on loopback only: `ws://` for `http://` pages and
  `wss://` with a per-install certificate for `https://` pages. Raw ESC/POS printing to printers
  installed in the operating system.
- Permission prompt per site, remembered for HTTPS and localhost sites, with a site list to revoke
  or unblock them. Sites can send a name and logo for the prompt (NanoPrint extension).
- Printers tab: the printers installed on the computer with their driver and port, a switch to
  offer each one to sites or not, and an ESC/POS test page. Printers are offered by default,
  except those that cannot print tickets: virtual ones (PDF and XPS writers, fax, OneNote) and
  Remote Desktop printers redirected through Easy Print.
- History of the last 30 days, searchable by site and filtered by event type. Printed content is
  never stored.
- Fixed ports for the print servers in Preferences, for computers that also run QZ Tray.
- First-run welcome and guided activation of secure sites, offered again when a browser refuses
  the certificate.
- Limits per site: 30 print jobs a minute, 16 connections, 10 copies per job.
- Third-party licenses window and a diagnostic log the user can open from the panel.
- `nanoprint://start` link: a page can start NanoPrint when the user has quit it. The browser asks
  first; NanoPrint starts quietly in the tray and the link carries nothing else.
- Windows uninstaller cleanup: removes the certificate from the trust store and the signing key
  when the user deletes NanoPrint's data.

[Unreleased]: https://github.com/nanofactura/nanoprint/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/nanofactura/nanoprint/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/nanofactura/nanoprint/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/nanofactura/nanoprint/releases/tag/v0.1.0
