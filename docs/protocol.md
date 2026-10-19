# Protocol

NanoPrint speaks the subset of the qz-tray.js wire protocol that raw printing needs. This document
describes it in our own words, from the public qz-tray.js API and from black-box traces of the
real client (qz-tray.js 2.3.0) talking to a test server. `src-tauri/src/server/tests.rs` runs that
client against NanoPrint on every build.

> QZ Tray and qz-tray.js are products of QZ Industries, LLC; QZ Tray is its trademark. NanoPrint is an independent project, not affiliated with or endorsed by QZ Industries, and mentions them only to describe compatibility.

## Transport

- WebSocket on loopback only: `127.0.0.1` and `::1`.
- Insecure ports tried in order: `8182, 8283, 8384, 8485` (the first free one is used). Secure
  (`wss://`) ports `8181, 8282, 8383, 8484` come with local TLS, version 1.3 only.
- The upgrade request must carry `Host: localhost|127.0.0.1|[::1]:<port>` and an `Origin` that is
  a plain `http(s)` origin. Anything else gets HTTP 403 and is recorded in the history (once a
  minute per origin and reason, at most 20 a minute).
- Text frames only, at most 2 MiB each. Binary frames close the connection.
- At most 16 open connections per origin (256 in total); extra ones are closed after the
  handshake. A connection must start asking for permission within 30 seconds, and an allowed one
  that sends nothing (not even `"ping"`) for 3 minutes is closed.
- On the secure ports, a client that does not trust the local certificate ends the TLS handshake
  with an alert (`certificate_unknown` from Chromium browsers, `bad_certificate` from Firefox,
  `unknown_ca` from OpenSSL and rustls). NanoPrint takes it as the cue to offer installing the
  certificate, once per run.

## Messages

Requests are JSON objects with a short `uid` that the reply echoes:

```json
{
  "call": "printers.find",
  "params": { "query": "pos" },
  "uid": "s4h1js",
  "timestamp": 1,
  "position": {},
  "signature": "",
  "signAlgorithm": "SHA1"
}
```

Replies are `{ "uid": "…", "result": … }` or `{ "uid": "…", "error": "message" }`. `timestamp`,
`position`, `signature` and `signAlgorithm` are accepted and ignored. The text frames `"ping"`
(keep-alive) and `"getProgramName"` get no reply.

## Connection sequence

1. `getVersion` → `"2.3.0"`. This is the protocol level whose request format NanoPrint
   implements; older levels make qz-tray.js rewrite requests into legacy shapes.
   `getVersion` with `{"details": true}` returns NanoPrint's own name, version and URL.
2. The setup message `{"certificate": <string or null>, "uid": "…"}`. NanoPrint decides on the
   page's `Origin`, not on a certificate: a stored decision, or the user's answer to the permission
   prompt. Allowed → `{"uid", "result": null}`. Blocked →
   `{"uid", "error": "Connection blocked by client"}` and the connection closes.

   NanoPrint extension: when `certificate` is the JSON text
   `{"nanoprint": {"name": "…", "icon": "data:…"}}`, the name and logo are shown next to the
   origin. Any other value (`null`, a PEM certificate) is ignored. Rules and examples are in
   [integration.md](integration.md#site-name-and-logo).

3. Any later call also needs permission, so clients that skip step 2 are asked too.

## Calls

| Call                  | Params                     | Result                                                                                                                                 |
| --------------------- | -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `getVersion`          | `{details?}`               | Protocol level, or NanoPrint's details                                                                                                 |
| `printers.find`       | `{}`                       | Array of printer names                                                                                                                 |
| `printers.find`       | `{query}`                  | Name of the first printer matching case-insensitively (exact match first, then partial); error `Specified printer could not be found.` |
| `printers.getDefault` | none                       | Default printer name or `null`                                                                                                         |
| `printers.detail`     | none                       | Array of `{name, driver, connection, default}`                                                                                         |
| `print`               | `{printer, options, data}` | `null` once the job is in the printer queue                                                                                            |

Every other call (files, serial, USB, HID, sockets, networking, `websocket.stop`…) returns an error
naming the call. NanoPrint does not implement them.

Printers are those the operating system reports, followed by the network printers the user added
in the Printers tab (their `driver` is `null`, their `connection` is `host:port`, and they are
never the default). Sites only get the printers offered to them: by default every printer except those that cannot
print raw data, namely virtual ones (PDF and XPS writers, fax, OneNote) and printers redirected by
Remote Desktop through Easy Print, which drops raw data by design (Microsoft KB 2827141). The user
changes this per printer in the Printers tab. A printer not offered is left out of every list above, as if it were not installed,
and a `print` to it fails with `Specified printer could not be found.` On many Windows computers
the default printer is `Microsoft Print to PDF`: `printers.getDefault` then returns `null`.

### `print`

- `printer`: `{name}` of a printer the operating system reports or the user added in NanoPrint as
  a network printer. `{host, port}` (`port` defaults to 9100) must match the address of a network
  printer the user added (an IP address compares in its canonical form, a host name ignoring
  case); any other address fails with `Network printers must be added in NanoPrint first`.
- `options`: `jobName` (shown in the queue), `copies` (1–10), `encoding` for plain text: a name,
  or `{from, to}`, of which only `to` (the printer's encoding) matters because JSON text is already
  Unicode. An `encoding` of any other shape, or an object without `to`, refuses the job instead of
  printing in the wrong encoding. Other options are ignored.
- `data`: an array of elements, concatenated in order. A string is plain text. An object must be
  `{type: "raw", format: "command", flavor, data}` with `flavor`:
  - `plain`: text encoded with `options.encoding`: `UTF-8` (default), `ISO-8859-1`,
    `windows-1252`, `CP437`, `CP850`, `CP858`, with their Java aliases (`Cp850`, `IBM850`…).
    Characters the code page lacks become `?`.
  - `base64`, `hex`: bytes.
  - Other flavors (`file`, `xml`) and types (`pixel`, `html`, `pdf`, raw `image`) are refused.
- Each site may send 30 jobs per minute.
