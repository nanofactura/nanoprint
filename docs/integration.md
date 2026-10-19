# Printing from a web page

NanoPrint has no JavaScript SDK of its own. Pages talk to it through
[qz-tray.js](https://www.npmjs.com/package/qz-tray), the client of QZ Tray, and send raw ESC/POS
bytes built by any encoder. This guide covers what a web application needs; the wire protocol is
described in [protocol.md](protocol.md).

QZ Tray is a trademark of QZ Industries, LLC. NanoPrint is an independent project, not affiliated
with or endorsed by QZ Industries; it mentions QZ Tray only to describe compatibility.

## Recommended libraries

| Library                                                                                                          | Role                                                          | Notes                                                                                                                                             |
| ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`qz-tray`](https://www.npmjs.com/package/qz-tray) 2.2 or later                                                  | Connects to NanoPrint and sends the jobs                      | Tested on every build against qz-tray.js 2.3.0                                                                                                    |
| [`@point-of-sale/receipt-printer-encoder`](https://www.npmjs.com/package/@point-of-sale/receipt-printer-encoder) | Builds ESC/POS bytes: text, code pages, QR codes, images, cut | Pair it with `qz-tray`                                                                                                                            |
| [`web-escpos-printer`](https://www.npmjs.com/package/web-escpos-printer)                                         | Encoder and transports in one package                         | Use its QZ Tray transport. Install it from npm: the CDN bundle carries its own copy of qz-tray.js (see [Site name and logo](#site-name-and-logo)) |

Only raw printing is supported. Pixel, HTML and PDF printing, files, serial, USB and sockets
calls are refused: encode images into the ESC/POS stream instead (both encoders above can).

## Example

```js
import qz from 'qz-tray';
import ReceiptPrinterEncoder from '@point-of-sale/receipt-printer-encoder';

// Optional: how the site appears in NanoPrint's permission prompt and site list.
qz.security.setCertificatePromise((resolve) => {
  resolve(
    JSON.stringify({ nanoprint: { name: 'Kiosco Don Pepe', icon: 'data:image/png;base64,…' } }),
  );
});

await qz.websocket.connect();
const printer = await qz.printers.getDefault(); // or qz.printers.find('POS-58')

const bytes = new ReceiptPrinterEncoder({ columns: 32 }) // 58 mm paper; 48 for 80 mm
  .initialize()
  .line('Kiosco Don Pepe')
  .line('Total: $ 1.234,00')
  .qrcode('https://example.com/ticket/123')
  .cut()
  .encode();

const config = qz.configs.create(printer, { jobName: 'Ticket 123' });
await qz.print(config, [
  { type: 'raw', format: 'command', flavor: 'base64', data: toBase64(bytes) },
]);

function toBase64(bytes) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}
```

`qz.websocket.connect()` finds NanoPrint on its default ports. From an `http://` page it connects
to `ws://localhost:8182`; from an `https://` page it only tries `wss://localhost:8181` (see
[HTTPS pages](#https-pages)).

The user can fix other ports in NanoPrint (Preferences → Connection ports), for example when QZ
Tray runs on the same computer. Pages then have to name them:

```js
await qz.websocket.connect({ port: { secure: [9181], insecure: [9182] } });
```

Libraries that do not pass connection options through, such as `web-escpos-printer`, only find
NanoPrint on the default ports.

## When NanoPrint is not running

NanoPrint starts with the computer, but a user may have quit it. A page can start it again by
opening the `nanoprint://start` link: the browser asks the user whether to open NanoPrint, and the
system starts it quietly in the tray. Then connect again for a few seconds:

```js
async function openNanoPrint() {
  window.location.href = 'nanoprint://start';
  // NanoPrint needs a moment to start its servers.
  await qz.websocket.connect({ retries: 15, delay: 1 });
}
```

- Offer it as a button ("Open NanoPrint") after a failed connection, rather than opening the link
  on your own: browsers only open links to apps after a click. When NanoPrint is not installed,
  Chrome and Edge do nothing and the page stays as it is; other browsers may show a message. If
  the connection still fails, suggest downloading NanoPrint.
- The link can only start NanoPrint: whatever follows `nanoprint:` is ignored. If NanoPrint is
  already running, nothing happens. The site still needs the user's permission to print.
- It works once NanoPrint has been installed, on Windows, macOS and Linux (the installers register
  the link; the AppImage registers it the first time it runs).

## Permission

The first time a site connects, NanoPrint asks the user whether it may print. Until the user
answers, the connection waits (the prompt gives up two minutes after it appears; when several sites
ask at once they are shown one after another). When the user blocks the site, closes the prompt or
does not answer, the connection fails with the error `Connection blocked by client` and closes;
show a message explaining that printing was not allowed, and do not retry in a loop. The same error
comes back at once, without a prompt, while four other sites are already waiting for an answer; a
later attempt gets its prompt.

The decision is remembered for `https://` sites and for `localhost` (development). Other `http://`
sites are asked again every time NanoPrint starts, because on plain HTTP another machine on the
network could impersonate them. Every decision, remembered or not, appears in the NanoPrint panel,
where the user can revoke it at any time; the site's open connections then close.

## HTTPS pages

Browsers only let `https://` pages open `wss://` connections, so NanoPrint serves `wss://localhost`
with a certificate created on each computer. The computer must trust it once: NanoPrint offers this
on first run, again when a browser refuses the certificate, and in the panel ("Secure sites"). The
user confirms in a system dialog; no file has to be handled by hand.

If an `https://` page cannot connect while NanoPrint is running, ask the user to open the NanoPrint
panel and enable secure sites, then reload the page. Recent versions of Chrome may also ask the user
to allow the page to reach devices on the local network; qz-tray.js reports a refusal as an error.

## Site name and logo

A page can tell NanoPrint how to present it: a name and a square logo, shown next to its origin in
the permission prompt, the site list and the history. Both are optional, and both are claims made by
the page. NanoPrint always shows the origin first ("`https://shop.example` calls itself …"), and the
user's decision applies to the origin.

The page sends them in the field qz-tray.js uses for QZ Tray's signing certificate, which NanoPrint
does not otherwise need. Call this before connecting:

```js
qz.security.setCertificatePromise((resolve) => {
  resolve(JSON.stringify({ nanoprint: { name: 'Kiosco Don Pepe', icon: logoDataUrl } }));
});
```

- `name`: up to 40 characters. NanoPrint keeps letters, digits, single spaces and
  `. , - _ & ' ’ ( ) ! ? + #`, and drops everything else (control and invisible characters, text
  direction overrides, symbols, emoji, and `:`, `/` or `@`, so a name cannot read like an
  address).
- `icon`: a `data:` URL with base64 content, at most 32 KiB and 1024 pixels per side:
  `image/png`, `image/jpeg` or `image/webp`. Square works best; other shapes are centered on a
  transparent square. NanoPrint decodes the logo itself and stores a 96 × 96 PNG.
- SVG is not accepted: a few kilobytes of SVG can demand unbounded drawing work. Rasterize an SVG
  logo in the page first (see below).
- Links to images (`https://…/logo.png`) are ignored: loading them would tell the image's server
  when the user opens NanoPrint, and from which address.
- A value that fails these rules is ignored and the site appears with its origin only. The
  connection is not affected.
- The name and logo are stored with remembered decisions and updated whenever an allowed site
  connects again.

To send an SVG logo, let the browser draw it into a PNG:

```js
async function logoDataUrl(svgUrl) {
  const image = new Image();
  image.src = svgUrl;
  await image.decode();
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 96;
  canvas.getContext('2d').drawImage(image, 0, 0, 96, 96);
  return canvas.toDataURL('image/png');
}
```

Compatibility:

| Client                                                    | Can send a name and logo                                                                  |
| --------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `qz-tray` from npm, alone or with an encoder              | Yes                                                                                       |
| `web-escpos-printer` from npm (ES module)                 | Yes: it imports the same `qz-tray` module, so set the promise before the library connects |
| `web-escpos-printer` CDN bundle (`web-escpos-printer.js`) | No: its copy of qz-tray.js is internal                                                    |
| A site that signs requests for QZ Tray                    | No: the field carries its QZ certificate, and NanoPrint shows its origin only             |

## Limits

| Limit                       | Value                                                                                                                                                                                           |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Print jobs per site         | 30 per minute; more fail with `Too many print jobs; try again in a minute`                                                                                                                      |
| Copies per job              | 1 to 10                                                                                                                                                                                         |
| Message size                | 2 MiB per message                                                                                                                                                                               |
| Connections per site        | 16 at a time; more are closed right away                                                                                                                                                        |
| Asking for permission       | Within 30 seconds of connecting (qz-tray.js does it immediately), or the connection closes                                                                                                      |
| Idle connections            | Closed after 3 minutes without messages. qz-tray.js pings every 60 seconds unless `keepAlive` is 0                                                                                              |
| Printers                    | Installed in the operating system, or added in NanoPrint as network printers, and offered to sites: by default all but virtual ones (PDF, XPS, fax) and Remote Desktop printers with Easy Print |
| Network printers by address | `{host, port}` must match a network printer the user added in NanoPrint (Printers tab); others fail with `Network printers must be added in NanoPrint first`. Prefer the printer's name         |

NanoPrint never stores what is printed. Its history keeps the time, site, printer, job name, size
and result of each job for 30 days. A job succeeds once the system's print queue accepts it, or a
network printer takes all of its bytes: NanoPrint cannot tell whether paper came out.

A network receipt printer (Ethernet or Wi-Fi) works in two ways: installed in the operating system
like any other printer, or added in NanoPrint by IP address and port (usually 9100). Added in
NanoPrint, it needs no driver, and NanoPrint sends it one job at a time.

Over Remote Desktop, a printer redirected with Easy Print (driver "Remote Desktop Easy Print")
cannot receive raw data. Install the printer's own driver on the remote computer and turn off the
policy "Use Remote Desktop Easy Print printer driver first".
