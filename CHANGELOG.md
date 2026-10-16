# Changelog

Notable changes to NanoPrint. The format follows [Keep a Changelog](https://keepachangelog.com/),
and versions follow [Semantic Versioning](https://semver.org/).

> QZ Tray and qz-tray.js are products of QZ Industries, LLC; QZ Tray is its trademark. NanoPrint is an independent project, not affiliated with or endorsed by QZ Industries, and mentions them only to describe compatibility.

## [Unreleased]

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
- Windows uninstaller cleanup: removes the certificate from the trust store and the signing key
  when the user deletes NanoPrint's data.

[Unreleased]: https://github.com/nanofactura/nanoprint/commits/main
