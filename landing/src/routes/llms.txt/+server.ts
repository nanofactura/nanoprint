import { messages, pages } from '#lib/i18n/index.ts';
import { docs, downloads, RELEASES_URL, REPO_URL, SITE_URL } from '#lib/site.ts';

export const prerender = true;
export const trailingSlash = 'never';

/**
 * A plain summary for AI assistants and answer engines (https://llmstxt.org), built from the same
 * text as the pages so the two never disagree.
 */
export function GET(): Response {
  const en = messages('en');
  const faq = en.home.faq.items.map((item) => `### ${item.q}\n\n${item.a}`).join('\n\n');
  const text = `# NanoPrint

> ${en.home.description}

NanoPrint is made by NanoFactura and published under the GPL-3.0 license. It runs in the system
tray, listens only on this computer (wss://localhost:8181 and ws://localhost:8182, the ports
qz-tray.js uses) and asks the user before any website may print. It never stores printed content.

- Platforms: ${en.home.downloads.windows.requirements}; ${en.home.downloads.mac.requirements}; Linux ${en.home.downloads.linux.requirements}.
- Printers: ESC/POS thermal receipt printers (58 and 80 mm) installed in the operating system, or
  network printers the user adds in NanoPrint by IP address and port (raw TCP, usually 9100).
- Compatible client libraries: qz-tray (2.2 or later), @point-of-sale/receipt-printer-encoder, web-escpos-printer.
- Not supported: QZ Tray's file, serial, USB, HID, pixel, HTML and PDF printing calls.

## Pages

- [Home (English)](${SITE_URL}${pages.home.en}): features, downloads and installation steps.
- [Developers (English)](${SITE_URL}${pages.developers.en}): libraries, code example and live demo.
- [Inicio (español)](${SITE_URL}${pages.home.es}): la misma información en español.
- [Desarrolladores (español)](${SITE_URL}${pages.developers.es}): integración en español.

## Downloads

- [Windows installer](${downloads.windows})
- [macOS disk image, universal](${downloads.mac})
- [Linux .deb](${downloads.deb}), [.rpm](${downloads.rpm}), [AppImage](${downloads.appImage})
- [All releases](${RELEASES_URL})

## Documentation

- [Full text of the site](${SITE_URL}/llms-full.txt): every page, in English and Spanish.

- [Source code](${REPO_URL})
- [Integration guide](${docs.integration})
- [Protocol](${docs.protocol})
- [Security policy](${docs.security})

## FAQ

${faq}
`;
  return new Response(text, { headers: { 'content-type': 'text/plain; charset=utf-8' } });
}
