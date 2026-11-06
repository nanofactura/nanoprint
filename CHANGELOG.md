# Changelog

Notable changes to NanoPrint. The format follows [Keep a Changelog](https://keepachangelog.com/),
and versions follow [Semantic Versioning](https://semver.org/).

> QZ Tray and qz-tray.js are products of QZ Industries, LLC; QZ Tray is its trademark. NanoPrint is an independent project, not affiliated with or endorsed by QZ Industries, and mentions them only to describe compatibility.

## [Unreleased]

### Added

- With automatic updates off, a new version puts a dot on the tray icon, its number in the
  tooltip and a banner at the top of the panel, with a button that downloads and installs it.

### Changed

- With automatic updates off, NanoPrint now checks GitHub for new versions too, a minute after it
  starts and every six hours, instead of connecting to nothing. It stops checking once it finds
  one, and downloads it only when you install it.
- Windows: updates install without showing the installer. If one cannot install, NanoPrint starts
  again with the installed version and tries later, with no message to dismiss. This applies to
  updates installed from this version on.

### Fixed

- Windows: an update no longer fails with "could not close NanoPrint" while something still holds
  NanoPrint's program file, such as an antivirus scanning it right after the download or NanoPrint
  itself finishing closing. The installer waits up to 30 seconds for the file to be free.

## [0.3.0] - 2026-11-06

### Added

- Network printers: add a printer connected by cable or Wi-Fi in the Printers tab, by IP address
  or host name and port (9100 by default), with no driver. Sites see it next to the system's
  printers and print to it by name, or by `{host, port}` when it matches one the user added;
  NanoPrint never connects to an address a page chose. Jobs go one at a time per printer.

### Changed

- Windows installer: the desktop shortcut is unchecked by default. Installing a new version by hand
  keeps the shortcut if there was one; updates never add it.
- Updated SQLite to 3.54.0, the updater and single-instance plugins and other dependencies.

### Fixed

- Windows: when an update fails to install (an antivirus blocking the installer, for example),
  NanoPrint starts again with the installed version instead of staying closed, and tries the
  update again later.
- Windows: installing a new version by hand no longer turns off start with the system.
- Linux: async-executor 1.14.0, withdrawn for a use-after-free when an executor is dropped, is
  replaced with 1.13.3.

### Security

- Updates are extracted with zip 9.1.3, which fixes three extraction flaws (RUSTSEC-2026-0367,
  RUSTSEC-2026-0368, RUSTSEC-2026-0369). Updates were already verified against NanoPrint's update
  key before extraction.

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

[Unreleased]: https://github.com/nanofactura/nanoprint/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/nanofactura/nanoprint/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/nanofactura/nanoprint/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/nanofactura/nanoprint/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/nanofactura/nanoprint/releases/tag/v0.1.0
