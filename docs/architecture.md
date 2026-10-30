# Architecture

NanoPrint is a tray application built with [Tauri 2](https://v2.tauri.app/): a Rust core and a
Svelte UI rendered by the operating system's webview (WebView2, WKWebView, WebKitGTK). Most of the
time it runs with no window open, so its footprint is one Rust process and a tray icon.

> QZ Tray and qz-tray.js are products of QZ Industries, LLC; QZ Tray is its trademark. NanoPrint is an independent project, not affiliated with or endorsed by QZ Industries, and mentions them only to describe compatibility.

```
 web app (browser)                               NanoPrint (signed-in user)
 ┌──────────────────────┐   ws(s)://127.0.0.1   ┌───────────────────────────────────────────┐
 │ qz-tray.js-compatible│ ─────────────────────▶│ print server ─▶ permissions ─▶ printers   │
 │ client + ESC/POS     │   (QZ wire protocol)  │      │              │             │       │
 │ encoder              │                       │   protocol      prompt window   transports │
 └──────────────────────┘                       │                 (per origin)  queue/USB/  │
                                                │                               serial/TCP/ │
                                                │ tray + settings window        Bluetooth   │
                                                └───────────────────────────────────────────┘
```

## Components

| Component                       | Status          | Responsibility                                                                                                                                                                              |
| ------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tray and settings window        | Done            | Tray menu, settings window on demand, language, start with the system (quietly; opened by the user, it shows the welcome or the panel), `nanoprint://` link for pages to start it (quietly) |
| Print server                    | Done            | `ws://` and `wss://` WebSocket server on loopback; `Host`/`Origin` validation; message limits                                                                                               |
| Protocol layer                  | Done            | qz-tray.js wire protocol subset needed for raw printing ([protocol.md](protocol.md)), in its own crate (`crates/protocol`), fuzzed                                                          |
| Permission manager              | Done            | Per-origin consent prompt, signed allow/block store, revocation, 30-day history                                                                                                             |
| Printer registry and transports | Partial         | OS queue in raw mode (tested on Windows; CUPS on macOS and Linux untested). Pending: USB, serial, TCP 9100, BLE, Bluetooth SPP                                                              |
| Local TLS                       | Done (`wss://`) | Per-install certificate for `localhost`, `127.0.0.1` and `::1`, renewed while running; trust store install from the panel                                                                   |
| Onboarding                      | Done            | First-run welcome; offers to enable secure sites then, when a browser refuses the certificate, and from a panel banner                                                                      |
| Site identity                   | Done            | Optional name and logo sent by the page, sanitized and re-encoded ([integration.md](integration.md#site-name-and-logo))                                                                     |
| LAN mode                        | Planned         | Opt-in listener on the LAN with device pairing                                                                                                                                              |
| Updater                         | Done            | Signed automatic updates from GitHub Releases, installed when no job is printing; can be turned off                                                                                         |

## Compatibility with qz-tray.js clients

NanoPrint implements, in its own code, the part of the qz-tray.js wire protocol that raw printing
needs. The reference is the public qz-tray.js API (`qz.websocket.connect`, `qz.printers.find`,
`qz.printers.getDefault`, `qz.printers.details`, `qz.print` with raw data) plus black-box traces of
the real client connected to NanoPrint. The protocol notes in `docs/` are written in our own words
from those traces. Calls outside that subset (files, serial, USB, HID, sockets) return a protocol
error: they would widen the attack surface and NanoPrint does not need them.

Unmodified clients connect to the default QZ ports: secure `8181, 8282, 8383, 8484` and insecure
`8182, 8283, 8384, 8485`. On HTTPS pages they only try the secure ports, so serving `wss://` with a
trusted local certificate is what makes them work without code changes. NanoPrint therefore:

- uses those ports by default, taking the first free one of each list;
- lets the user fix other ports (Preferences → Connection ports, applied when NanoPrint restarts),
  for example when QZ Tray is installed on the same machine, and shows the active addresses in the
  settings window. Clients that do not let you configure the port (for example
  `web-escpos-printer`) only find NanoPrint on the default ports.

QZ Tray is named only to describe this compatibility. The protocol is reimplemented from public
documentation and observed behavior, without QZ code, logos or branding, and every place that
names QZ Tray carries the non-affiliation notice from the README.

## Permission model

1. A page opens the WebSocket. The browser sets `Origin`; NanoPrint rejects connections without
   one, with a malformed one, or from a blocked origin.
2. On the first request from an unknown origin, NanoPrint opens its own window: "`<origin>` wants
   to print", with the printers it may use, **Allow**, **Block** and **Remember this decision**.
   The window leads with the full origin (punycode for mixed scripts) and enables **Allow** only
   after a short delay to defeat click-jacking. A name and logo sent by the page appear only as a
   secondary "calls itself …" line: the name keeps letters, digits, spaces and common punctuation,
   and the logo (PNG, JPEG or WebP only: SVG references allow unbounded rendering work) is decoded
   in Rust with size limits and re-encoded as a 96 px PNG, so the webview never parses what the
   page sent and nothing is fetched (`src-tauri/src/identity.rs`). Logos are only decoded when they
   will be shown, never for blocked sites.
3. Several waiting sites are shown one at a time, and at most four wait at once: a fifth is refused
   without a prompt, so a page with many origins (subdomains) can neither flood the user nor hold
   connections while waiting. A request on screen for two minutes without an answer, or dismissed
   by closing the window, blocks that site for the session; the window then shows the next one.
   The window can only answer the request it shows.
4. Decisions are stored per exact origin (`scheme://host:port`). Only `https://` origins (and
   `http://localhost` for development) can be remembered; other origins get a session-only grant,
   and a stored decision replaces any earlier session one.
5. The settings window lists stored and session decisions with their last use. Revoking a site
   closes its open connections immediately.
6. Each origin may hold 16 connections (256 in total). A connection must ask for permission within
   30 seconds, and allowed ones idle for 3 minutes are closed, so no page can hold every slot.
   Refused handshakes reach the history at most once a minute per origin and reason.

The browser may add its own prompt on top: recent Chrome versions ask the user before a public
site can reach devices on the local network, this computer included. NanoPrint cannot pre-grant
it; the integration guide tells developers to expect it.

## Data storage

| Data                                                                    | Location                                                                                                                                                                                                        | Format                  |
| ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| Preferences (language, welcome shown, printers offered to sites, ports) | App config directory: `%APPDATA%\com.nanofactura.nanoprint` (Windows), `~/Library/Application Support/com.nanofactura.nanoprint` (macOS), `~/.config/com.nanofactura.nanoprint` (Linux)                         | `settings.json`         |
| Site permissions, paired devices, history                               | App local data directory: `%LOCALAPPDATA%\com.nanofactura.nanoprint` (Windows, never synced by roaming profiles), `~/Library/Application Support/…` (macOS), `~/.local/share/com.nanofactura.nanoprint` (Linux) | SQLite (`nanoprint.db`) |

**Who can read and write them.** Keeping per-user data in the user's profile is the standard
practice: browsers do the same (Firefox keeps site permissions in `permissions.sqlite`, Chrome in its
`Preferences` file). Files are created for the signed-in user only (the profile's ACLs on Windows;
mode `0600` files in a `0700` directory on macOS and Linux), so other users of the computer cannot
read or change them. Administrators and root can, and so can any program running as the same user,
as with any per-user file.

A program running as the user could therefore add an allowed site by editing the database. The
operating system does not isolate programs of the same user from each other, so no user-level app
can fully prevent this, and such a program could print through the operating system anyway.
NanoPrint still raises the bar:

- **Signed permissions.** Every row of the sites and paired-devices tables carries an HMAC-SHA256
  computed with a random key kept in the operating system's credential store (Windows Credential
  Manager, macOS Keychain, Linux Secret Service). A row whose signature does not verify is treated as
  "not allowed" and flagged in the settings window. On macOS the Keychain only gives that key to apps
  signed with NanoPrint's identity ([releasing.md](releasing.md#macos-without-an-apple-developer-account)); on Windows and Linux an attacker has to target NanoPrint specifically to
  extract it. Systems without a credential store keep the key in a user-only file instead. Once the
  credential store has held the key (`integrity.keyring` marks it), NanoPrint never switches to the
  file: if the store does not answer (a keyring still locked right after login) it retries every
  few seconds, and meanwhile decisions only last for the session. The store is asked off the main
  thread, since it may wait for the user to unlock it; connections in the first 15 seconds wait
  for the key so remembered sites are not asked again.
- No secrets in clear text: LAN device tokens are stored only as SHA-256 hashes.
- Everything read from disk is validated; a corrupt or unknown entry means "not allowed".
- The history never stores job contents: receipts contain personal data.
- Writes are atomic (SQLite transactions; temporary file + rename for `settings.json`).

**Sites table:** origin, decision (allowed or blocked), granted since, how it was granted (user
prompt or administrator policy), last use, job count, the printers it may use (planned), and the
name and logo the site sent. The name and logo are cosmetic, so they are not covered by the
signature. Schema changes are applied in order at startup and counted in `PRAGMA user_version`.

**History (30 days):** one row per event, with time, origin or paired device, event type
(connection accepted or rejected, permission granted, denied or revoked, job printed or failed,
rate limit hit), printer, transport, job name, size in bytes, copies, duration and error message.
Rows older than 30 days are deleted at startup and once a day. The settings window shows the
latest 500 matching rows, filtered by text (site origin or name) and by event type (prints,
errors, permissions). A CSV export is planned.

## Network modes

- **Loopback (default):** bound to `127.0.0.1` and `::1` only. Nothing on the network can reach it.
- **LAN (opt-in, off by default):** an extra listener on the selected interface. Because a
  non-browser client can forge `Origin`, every LAN device is paired:
  1. The settings window shows the address to use (IP and `<computer-name>.local`) and a short
     one-time pairing code that expires after a few minutes.
  2. The device connects with the code; the user approves it in a NanoPrint prompt.
  3. NanoPrint gives the device a long random token. The device stores it and presents it on every
     later connection. **The token is the device's identity**, not its IP or MAC address: IPs change
     with DHCP, MAC addresses are not visible across routers, can be spoofed, and phones randomize
     them per network.
  4. Paired devices appear in the same list as sites and can be revoked, which invalidates the
     token.

  The NanoPrint computer's own address should stay stable: use a DHCP reservation on the router, or
  connect through the `.local` name that NanoPrint announces with mDNS. A QR code with the address
  and code is only worth adding once a client app can scan it. Browsers on other devices need
  `wss://` with a certificate they trust, so LAN mode mainly targets backend services and native
  apps.

## Local TLS

On first start NanoPrint creates a certificate authority unique to that install and uses it to
issue a certificate for `localhost`, `127.0.0.1` and `::1` (`src-tauri/src/tls.rs`). The
authority's private key only lives in memory while it signs and is never written: without it,
nobody can mint certificates for other sites, even with the authority trusted. The authority also
carries X.509 name constraints limiting it to those loopback names and a path length of 0.

- Files, in `tls/` under the app local data directory: `ca.pem` (public), `localhost.pem`,
  `localhost-key.pem` (user-only), `certificate.json` (name, serial, expiry) and, after a renewal,
  `replaced.json`.
- Validity is 800 days (Apple rejects TLS certificates valid for more than 825). Thirty days before
  expiry a new authority and certificate replace them, checked at startup and every 12 hours: the
  running server switches to the new certificate without a restart (a POS computer may run
  NanoPrint for months), and the reminder to enable secure sites can show again. The old authority
  is recorded in `replaced.json` first and leaves the trust store with the next install; it stays
  recorded until its removal succeeds, across restarts.
- The `wss://` server uses rustls with the `ring` provider, TLS 1.3 only, on the secure ports
  `8181, 8282, 8383, 8484`.
- The panel shows whether the computer trusts the authority and installs or removes it on request
  (`src-tauri/src/trust.rs`), with the system's own confirmation: `certutil -user` on Windows,
  `security` on the macOS login keychain, NSS `certutil` on `~/.pki/nssdb` (Chrome) on Linux.
- Onboarding (`src-tauri/src/onboarding.rs`): the first-run welcome offers to enable secure sites.
  Later, when a browser ends a `wss://` handshake because it does not trust the certificate,
  NanoPrint opens the same window on top, once per run; the panel also shows a banner until the
  certificate is trusted. The installer does not do it: the system dialog would appear without
  context.
- Uninstalling (`src-tauri/src/uninstall.rs`): `nanoprint --uninstall` removes this
  installation's authorities from the trust store and the signing key from the credential store.
  The Windows uninstaller runs it when the user ticks "Delete the application data" (a hook in
  `src-tauri/windows/hooks.nsh`, skipped during updates); keeping the data keeps the certificate
  trusted for a reinstall. On macOS and Linux the user runs it before deleting the app.
- Still open: Firefox on Linux (own trust store).

## Webview hardening

- Content Security Policy (`tauri.conf.json`): scripts, styles, fonts and images from the app
  only (images also from `data:` URLs, which NanoPrint generates itself); IPC as the only
  connection; no plugins, frames, `<base>` or form submissions. The only relaxation is inline
  styles, which the UI components set for positioning.
- Each window gets its own capability file listing exactly the commands it may call, and the
  app's commands are declared in `build.rs`, so a window cannot reach anything else.
- `freezePrototype` stops scripts from tampering with built-in prototypes, no window loads remote
  content, and the webview's own context menu (reload, print, save as) is off in release builds.
- `npm run build` minifies the JavaScript and CSS (Vite), and Tauri embeds the assets in the
  executable compressed with Brotli.
- The licenses window (`third-party-notices.html`, generated for releases by `npm run notices`
  from cargo-about and the npm `dependencies`) is a static page with no capability at all.

## Least privilege

- **Installation:** per-user installer, no administrator rights. MSI is not built: it installs for
  all users and needs elevation. NanoPrint runs as the signed-in user and never asks for elevation.
- **System changes:** start with the system through the user's own entry (registry `Run` key,
  LaunchAgent, XDG autostart); the certificate in the user's trust store only; the signing key in
  the user's credential store; data in the user's profile, readable by that user only.
- **Clients:** loopback only, an explicit decision per origin, and only printing and printer
  listing, limited to the printers offered to sites: by default every printer that can print raw
  data (not PDF and XPS writers, fax, OneNote or Remote Desktop's Easy Print), and the Printers
  tab changes that per printer; see the [threat model](threat-model.md).
- **Windows of the app:** each has a capability with only the commands it calls (the settings
  window: its own commands, the app version and start with the system); the licenses window has
  none. No shell, file system, HTTP or opener plugin is included.
- **Processes started:** `certutil` or `security` to manage the certificate and the file manager to
  show the log folder, all with fixed argument vectors.
- **Logs** (`src-tauri/src/logs.rs`): their purpose is diagnosing a problem the user reports
  (Preferences → About → Diagnostic log opens the folder). Info level in releases, debug in
  development; the WebSocket and TLS libraries only log warnings, since their debug output carries
  message contents, so print data, site names and logos never reach the log. A new file per
  session, the previous session's kept, at most 128 KB each; archived files are deleted after 30
  days. On this computer only.
- **CI:** read-only token by default; the release job alone can write releases and sign provenance.

## Automatic updates

[`updates.rs`](../src-tauri/src/updates.rs), with
[`tauri-plugin-updater`](https://v2.tauri.app/plugin/updater/); key custody is described in
[releasing.md](releasing.md#update-signing-key):

1. Each release built by GitHub Actions publishes the installers, their signatures and a
   `latest.json` manifest on GitHub Releases, all signed with the update key (a CI secret).
2. A minute after it starts and every six hours, NanoPrint reads
   `releases/latest/download/latest.json`. A newer version is downloaded in the background and its
   signature checked against the public key embedded in the app (`tauri.conf.json`); a failed
   verification aborts the update.
3. It installs once no job has printed for five minutes, so it never interrupts printing, and
   starts again quietly in the tray (a flag in the settings marks the start after an update).
   Windows runs the per-user installer in passive mode; macOS replaces the app bundle; the Linux
   AppImage replaces itself. `.deb` and `.rpm` installs update through their package manager, and
   development builds never update.
   If the Windows installer fails (an antivirus blocking it, for example), it starts the installed
   version again instead of leaving NanoPrint closed ([hooks.nsh](../src-tauri/windows/hooks.nsh)).
   The settings record which version was being installed: a start that is still on the previous
   version waits six hours before trying again, so a blocked update does not retry every minute.
4. Preferences → Updates shows the state, checks on request and installs a downloaded update at
   once. Turned off, NanoPrint checks only when asked: it then makes no connection of its own.

Release artifacts are additionally signed for the operating system: Authenticode on Windows (for
example through SignPath, free for open-source projects), Developer ID plus notarization on macOS.
The updater key must be backed up: losing it strands installed copies on the current version.

## Packaging

Built by the [release workflow](../.github/workflows/release.yml); see
[releasing.md](releasing.md).

- Windows: per-user NSIS installer in English and Spanish (no administrator rights). The desktop
  shortcut is unchecked by default and kept when a new version is installed by hand over one that
  had it. An MSI for managed fleets would need a per-machine install and is left out for now.
- macOS: universal (Apple silicon and Intel) DMG, signed with a stable self-signed identity (or
  ad-hoc without it) until there is a Developer ID; tray-only (no Dock icon).
- Linux: `.deb` and `.rpm` (depending on the NSS tools for the certificate; udev rules for USB
  printers will come with direct USB) and AppImage. Snap and Flatpak are avoided because their
  sandboxes block serial and USB access.

## Landing page and downloads

The landing page is a static site in `site/` with its own `package.json`, deployed to Cloudflare by
a GitHub Actions workflow that runs only when a push changes `site/`. Releases never require
redeploying it:

- The release workflow also uploads each installer under a stable, version-less name, so download
  buttons point to `https://github.com/nanofactura/nanoprint/releases/latest/download/<name>`, which
  GitHub always resolves to the newest release.
- If the page shows the current version or release notes, it reads them in the browser from the
  GitHub API (`/repos/nanofactura/nanoprint/releases/latest`) and simply hides them if the request
  fails.

## Testing

| Level         | Tool                        | Covers                                                                                     | Status  |
| ------------- | --------------------------- | ------------------------------------------------------------------------------------------ | ------- |
| Unit (Rust)   | `cargo test`                | i18n, settings, protocol parsing, signed store and migrations, site name and logo checks   | Done    |
| Unit (web)    | Vitest                      | Locale catalogs share keys and placeholders, message formatting                            | Done    |
| Integration   | `cargo test` + tokio client | Print server: `Host`/`Origin` checks, handshake, permissions, limits, `wss://`, TLS alerts | Done    |
| Compatibility | `cargo test` + Node         | Real qz-tray.js over `ws://` and `wss://`, with a site name, and `web-escpos-printer`      | Done    |
| Fuzzing       | proptest + `cargo fuzz`     | Protocol parser: properties on every build, two minutes of coverage-guided fuzzing in CI   | Done    |
| End to end    | `tauri-driver` (WebDriver)  | Settings window and permission prompt (Windows and Linux; macOS has no WebDriver support)  | M3      |
| Manual        | Test page + real printers   | Printing on a 58 mm clone, an 80 mm printer and a network printer                          | Release |

CI runs every automated level on Windows, macOS and Linux.

## Roadmap to 1.0

**1.0 means a business can replace QZ Tray with NanoPrint** on Windows, macOS and Linux for
printers installed in the operating system or reachable on the network (TCP 9100).

| Milestone                                   | Scope                                                                                                                                                                                                  |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **M0, foundation** (done)                   | Tauri + Svelte app, tray, settings window, i18n, start with the system, lint and CI                                                                                                                    |
| **M1, print server** (done except TCP 9100) | Loopback WebSocket, `Host`/`Origin` validation, qz-tray.js protocol subset, permission prompt, SQLite store (sites, history), OS print queue on the three systems, TCP 9100 printers added by the user |
| **M2, local TLS** (done except Firefox)     | `wss://` with the per-install certificate, trust store integration including Firefox                                                                                                                   |
| **M3, settings UI** (done except roles)     | Sites (granted since, last use, revoke), printers (default, roles, test page), 30-day history, port, about                                                                                             |
| **M4, releases**                            | Signed installers, automatic updates, landing page with downloads                                                                                                                                      |
| **M5, hardening**                           | Fuzzing, security review, compatibility matrix, user and integration guides                                                                                                                            |

### 1.0 release criteria

- Compatibility suite green with unmodified qz-tray.js and the QZ transport of `web-escpos-printer`
  on Chrome, Edge and Firefox (Safari through `wss://`).
- Tested on real printers: a generic 58 mm clone, an 80 mm printer (for example Epson TM-T20) and a
  network printer.
- Threat model documented, protocol parser fuzzed, and an independent review of the print server,
  the permission flow and the updater with no open high-severity findings.
- Signed installers (Authenticode, Developer ID + notarization, Linux packages), update tested from
  the previous release, and a clean uninstall (autostart entry, trust entry, data).
- User guide (English and Spanish), integration guide for developers, and a privacy note on what is
  logged and for how long.

### After 1.0

Direct USB, serial and Bluetooth (BLE and SPP) transports, LAN mode with device pairing, printer
status (paper out, cover open).

## Versioning

The version is defined once, in `[workspace.package] version` of the root `Cargo.toml`, following
SemVer. `tauri.conf.json` and `package.json` deliberately have no `version`: Tauri falls back to
the Cargo version for the binaries, installers and updater, and the UI reads it at runtime
(`getVersion()` in the web UI, `package_info()` in Rust). MSI installers only accept numeric
pre-release identifiers, such as `1.2.0-1`.
