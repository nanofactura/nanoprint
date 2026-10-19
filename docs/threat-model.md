# Threat model

What NanoPrint protects, from whom, how, and what it deliberately leaves out. Mitigations point to
the code or the document that implements them. Report anything that breaks these assumptions as
described in [SECURITY.md](../SECURITY.md).

## What is protected

- **The printers**: nothing prints unless the user allowed that site.
- **The computer**: a web page must not get any other capability through NanoPrint (files,
  processes, network access, other devices).
- **The user's privacy**: which sites print, and what. Printed content is never stored or logged.
- **The trust store**: the local certificate authority NanoPrint adds must not help anyone
  impersonate other sites.
- **Stored decisions**: a site cannot grant itself permission.

## Trust boundaries and actors

| Actor                                   | Trust                    | Reaches NanoPrint through                                              |
| --------------------------------------- | ------------------------ | ---------------------------------------------------------------------- |
| Any web page the user opens             | Untrusted                | The loopback WebSocket, via the browser (which sets `Origin`)          |
| A site the user allowed                 | Limited                  | The same socket, after consent: list printers and print, within limits |
| The user                                | Trusted                  | NanoPrint's own windows                                                |
| Network attackers (LAN, Wi-Fi, DNS)     | Untrusted                | Only indirectly: the server listens on loopback only                   |
| Other programs running as the same user | Out of scope (see below) | Everything the user can do                                             |
| Dependencies, CI and release pipeline   | Verified                 | The code and installers we ship                                        |

## Threats and mitigations

| Threat                                                            | Mitigation                                                                                                                                                                                                                                                                                                                                                                                                               |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| A page prints without the user's consent                          | Consent per exact origin in NanoPrint's own window; every call needs it ([permissions.rs](../src-tauri/src/permissions.rs)). Only HTTPS and localhost decisions are remembered.                                                                                                                                                                                                                                          |
| DNS rebinding (a page resolving its own name to 127.0.0.1)        | The `Host` header must name loopback and the server's port ([server.rs](../src-tauri/src/server.rs)).                                                                                                                                                                                                                                                                                                                    |
| Pages without an origin (`file://`, sandboxed frames, extensions) | Refused before the WebSocket opens; only plain `http(s)` origins pass.                                                                                                                                                                                                                                                                                                                                                   |
| Click-jacking or tricking the user into allowing                  | Always-on-top prompt with **Block** focused and **Allow** enabled after one second; the origin leads, in punycode.                                                                                                                                                                                                                                                                                                       |
| Impersonation through the site's name or logo                     | Shown only as "calls itself …" under the origin, in every window; the name keeps safe characters only (no `:`, `/` or `@`, so it cannot read like an address); the logo is re-encoded ([identity.rs](../src-tauri/src/identity.rs)).                                                                                                                                                                                     |
| Exploiting image decoders with a crafted logo                     | Decoded in memory-safe Rust with size and pixel limits, only for prompts and allowed sites; PNG, JPEG and WebP only (no SVG); never fetched from a URL; re-encoded before the webview sees it.                                                                                                                                                                                                                           |
| Reaching anything but printing                                    | The protocol implements only printer listing and raw printing; files, sockets, serial and USB are refused ([protocol.md](protocol.md)).                                                                                                                                                                                                                                                                                  |
| Using NanoPrint to reach hosts on the network                     | NanoPrint connects only to the network printers the user added in its own window; a page's `{host, port}` must match one exactly, and the host is checked to be an IP address or a host name only ([network.rs](../src-tauri/src/printers/network.rs)). One connection at a time per printer, with time limits.                                                                                                          |
| A web page starting NanoPrint (`nanoprint://` links)              | The browser asks the user before opening the link. The link only starts NanoPrint, quietly; the rest of it is ignored, it carries no command or data, and arrives as one argument that cannot match NanoPrint's own flags ([url_launch.rs](../src-tauri/src/url_launch.rs)). Printing still needs the site's permission.                                                                                                 |
| Injection into system commands                                    | No shell anywhere: argument vectors only, and printer names must match one the system reports exactly.                                                                                                                                                                                                                                                                                                                   |
| Denial of service (connections, prompts, history, paper)          | 16 connections per site and 256 in total, 30 s to ask for permission, 3 min idle limit, one prompt at a time and at most four sites waiting, a 2 min timeout per prompt, refusals and rate-limit hits recorded at most once a minute, 2 MiB messages, 30 jobs a minute, 10 copies.                                                                                                                                       |
| Misusing the local certificate authority                          | Unique per install, private key destroyed after issuing, X.509 name constraints to loopback, path length 0, user trust store only, TLS 1.3 ([tls.rs](../src-tauri/src/tls.rs)).                                                                                                                                                                                                                                          |
| Another program editing stored decisions                          | Rows are HMAC-signed with a key in the system credential store; unverifiable rows count as "not allowed" ([store.rs](../src-tauri/src/store.rs), [keys.rs](../src-tauri/src/keys.rs)).                                                                                                                                                                                                                                   |
| Script injection into NanoPrint's windows                         | Strict CSP, no remote content, untrusted strings rendered as text, per-window capabilities with only the commands each window uses ([architecture.md](architecture.md#webview-hardening)).                                                                                                                                                                                                                               |
| Compromised dependency or build                                   | Lockfiles, Dependabot with a 7-day cooldown, `cargo deny`, npm registry signatures, no install scripts, actions pinned by commit, zizmor, CodeQL, build provenance attestations and an SBOM per release.                                                                                                                                                                                                                 |
| Malicious update                                                  | Every update is signed with NanoPrint's update key and verified against the public key built into the app before it is installed; a bad signature aborts it ([updates.rs](../src-tauri/src/updates.rs)). The key is a secret of the protected `release` environment, used only by the release workflow; its custody is in [releasing.md](releasing.md#update-signing-key). Downloads go over HTTPS from GitHub Releases. |
| Leaking which sites print, or what                                | History and logs stay on the computer; printed content is never stored; no telemetry.                                                                                                                                                                                                                                                                                                                                    |

## Out of scope and residual risks

- **Programs running as the same user.** They can do whatever the user can, including printing
  through the operating system or editing NanoPrint's files. The signed decisions only raise the bar.
- **Administrators, root and physical access.**
- **Plain HTTP sites on a hostile network** can be impersonated; their permission lasts for the
  session only, and the prompt says so.
- **Jobs to network printers travel in clear text**, as raw TCP printing has no encryption or
  authentication: someone on the local network who can pose as the printer's address can read
  them. Printers on the OS queue over the network have the same exposure.
- **Unsigned installers.** Until code signing is in place, Windows SmartScreen and macOS Gatekeeper
  warn on first run, and users must verify downloads themselves (provenance attestations help).
- **The browser's own prompts**, such as Chrome's local network access permission, are outside
  NanoPrint's control.
- **A compromised maintainer account** could publish a malicious release; branch protection,
  required reviews and the protected `release` environment reduce that risk.

## Review status

- Two internal reviews with 15 findings each, all fixed (October 2026).
- The protocol parser, everything a page can send, lives in its own crate (`crates/protocol`) with
  property tests on every build and coverage-guided fuzzing in CI (`fuzz/`).
- Pending before 1.0: an independent security review.
