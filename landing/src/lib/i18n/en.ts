import type { Messages } from './types';

// Text between backticks renders as code.
export const en: Messages = {
  meta: { locale: 'en_US' },
  nav: {
    features: 'Features',
    download: 'Download',
    developers: 'Developers',
    faq: 'FAQ',
    github: 'GitHub',
    otherLanguage: 'Español',
    languageLabel: 'Leer en español',
    skip: 'Skip to content',
    menu: 'Menu',
  },
  download: {
    generic: 'Download NanoPrint',
    windows: 'Download for Windows',
    mac: 'Download for macOS',
    linux: 'Download for Linux',
    otherPlatforms: 'Other systems',
    previousVersions: 'Previous versions',
    desktopOnly: 'NanoPrint runs on Windows, macOS and Linux computers.',
  },
  home: {
    title: 'NanoPrint: print receipts from the web to your thermal printer',
    description:
      'Free, open-source app for Windows, macOS and Linux that prints ESC/POS receipts from your web app to thermal printers, with permission per site. Compatible with qz-tray.js.',
    hero: {
      eyebrow: 'Free and open source',
      title: 'Print receipts from the web, straight to your thermal printer',
      lead: 'NanoPrint is a lightweight app that connects your web app to the receipt printers of the computer. Every site asks for permission before it prints.',
      platforms: 'Windows · macOS · Linux',
      compatible: 'Compatible with qz-tray.js',
      videoLabel: 'Animation: a web app prints a receipt through NanoPrint.',
    },
    features: {
      title: 'Built for the counter',
      lead: 'What a shop needs: receipts that print fast, with no odd setup and no risk.',
      items: [
        {
          title: 'Cross-platform',
          text: 'The same web app prints the same way on Windows, macOS and Linux, with the printers you already have installed.',
          points: [
            'Windows 10 and 11',
            'macOS 10.15 or later, Intel and Apple silicon',
            'Ubuntu, Debian, Fedora and other distributions',
          ],
        },
        {
          title: 'Lightweight',
          text: 'An installer of a few megabytes. It lives in the system tray and starts with the computer.',
          points: [
            'No Java or other software needed',
            'Installs without administrator rights on Windows',
            'In English and Spanish',
          ],
        },
        {
          title: 'Secure',
          text: 'No site prints without your permission, and what you print never leaves your computer.',
          points: [
            'Permission per site, revocable at any time',
            'Accepts connections from this computer only',
            'Never stores what is printed',
          ],
        },
      ],
    },
    how: {
      title: 'How it works',
      items: [
        {
          title: 'Install NanoPrint',
          text: 'It sits in the system tray, next to the clock, and starts with the computer.',
        },
        {
          title: 'Open your web app',
          text: 'The first time a site wants to print, NanoPrint asks whether you allow it.',
        },
        {
          title: 'Print',
          text: "The receipt goes straight to the printer, without the browser's print dialog.",
        },
      ],
    },
    compat: {
      title: 'Does your app use QZ Tray?',
      text: 'NanoPrint speaks the same protocol as qz-tray.js for raw printing. If your web app already prints receipts with qz-tray.js, it usually works without code changes.',
      link: 'Details for developers',
      note: 'QZ Tray is a trademark of QZ Industries, LLC. NanoPrint is an independent project, not affiliated with QZ Industries.',
    },
    printers: {
      title: 'Supported printers',
      text: 'Thermal receipt printers that understand ESC/POS, 58 or 80 mm wide, connected by USB or over the network and installed in the system. ESC/POS is the language of most receipt printers.',
      driversTitle: 'Do I need a driver?',
      drivers: [
        {
          os: 'Windows',
          text: 'The printer must appear in Settings → Printers & scanners. The manufacturer\'s driver works, and so does the "Generic / Text Only" driver that comes with Windows: NanoPrint sends the commands as they are.',
        },
        {
          os: 'macOS',
          text: 'Add it in System Settings → Printers & Scanners with any driver. NanoPrint sends raw data, so the driver does not alter it.',
        },
        {
          os: 'Linux',
          text: "Add it in the printer settings (CUPS) with the manufacturer's driver or as a generic printer. NanoPrint prints without filters, in raw mode.",
        },
      ],
      note: 'NanoPrint does not install drivers, so it needs no administrator rights. Virtual printers (PDF, XPS, fax) are not offered to sites.',
    },
    downloads: {
      title: 'Download NanoPrint',
      lead: 'Free, no sign-up. Pick your system.',
      requirements: 'Requirements',
      install: 'Installation',
      windows: {
        name: 'Windows',
        requirements: 'Windows 10 or 11, 64-bit',
        files: [{ label: 'Installer (.exe)', download: 'windows' }],
        steps: [
          'Open the downloaded file.',
          'If "Windows protected your PC" appears, click "More info", then "Run anyway". The warning shows because the installer does not have a commercial digital signature yet.',
          'Follow the wizard. It does not ask for administrator rights.',
        ],
      },
      mac: {
        name: 'macOS',
        requirements: 'macOS 10.15 or later, Intel and Apple silicon',
        files: [{ label: 'Disk image (.dmg)', download: 'mac' }],
        steps: [
          'Open the .dmg file and drag NanoPrint to Applications.',
          'The first time, macOS says it cannot verify the developer. Open System Settings → Privacy & Security and click "Open Anyway".',
          'Confirm with your password. From then on it opens directly.',
        ],
      },
      linux: {
        name: 'Linux',
        requirements: '64-bit: Ubuntu 22.04, Debian 12, Fedora 38 or later',
        files: [
          { label: '.deb (Ubuntu, Debian)', download: 'deb' },
          { label: '.rpm (Fedora, openSUSE)', download: 'rpm' },
          { label: 'AppImage (others)', download: 'appImage' },
        ],
        steps: [
          'Ubuntu or Debian: `sudo apt install ./NanoPrint_amd64.deb`',
          'Fedora: `sudo dnf install ./NanoPrint_x86_64.rpm`',
          'AppImage: `chmod +x NanoPrint_amd64.AppImage`, then open it.',
        ],
      },
      afterTitle: 'After installing',
      afterText:
        'NanoPrint welcomes you and offers to enable secure (https) sites in one click. Then it stays in the system tray, next to the clock.',
      previous: "Previous versions and what's new in each",
      verifyTitle: 'Verify the installer',
      verifyText:
        "Every installer is built on GitHub from the published source and carries a provenance attestation. GitHub's `gh` tool checks it: `gh attestation verify NanoPrint_x64-setup.exe --repo nanofactura/nanoprint`",
    },
    openSource: {
      title: 'Open source',
      text: 'NanoPrint is free software under the GPL-3.0 license. All of its code is on GitHub: review it, report issues or contribute.',
      button: 'View on GitHub',
    },
    faq: {
      title: 'Frequently asked questions',
      items: [
        {
          q: 'Is NanoPrint free?',
          a: 'Yes, for personal and commercial use. It is free software under the GPL-3.0 license.',
        },
        {
          q: 'Does it work with apps built for QZ Tray?',
          a: "Yes, for raw receipt printing: NanoPrint implements the qz-tray.js protocol. QZ Tray's other features (files, serial ports, USB, PDF or HTML printing) are not available.",
        },
        {
          q: 'Which printers work?',
          a: "ESC/POS thermal receipt printers, 58 or 80 mm, installed in the system and connected by USB or over the network. NanoPrint's panel prints a test page and chooses which printers sites can see.",
        },
        {
          q: 'Do I need a special driver?',
          a: "No. The printer must be installed in the operating system, with the manufacturer's driver or a generic one. NanoPrint sends the commands as they are.",
        },
        {
          q: 'Can a web page print without me noticing?',
          a: 'No. Every site must ask for permission, and you decide. NanoPrint only accepts connections from this computer, and permissions can be revoked from its panel.',
        },
        {
          q: 'Does NanoPrint store what I print?',
          a: 'No. Its history keeps the date, site, printer and result of each job for 30 days, never the content. NanoPrint sends no data to the internet.',
        },
        {
          q: 'Why do Windows and macOS warn me when installing?',
          a: 'Because the installer does not have a commercial digital signature yet, as is common for new open-source projects. You can verify that each installer was built from the source published on GitHub.',
        },
        {
          q: 'Does it work over Remote Desktop?',
          a: 'NanoPrint prints to the printers installed on the computer it runs on. Printers redirected by Remote Desktop through "Easy Print" cannot receive receipts: install the printer\'s driver on the remote computer and turn off Easy Print.',
        },
        {
          q: 'Does it work on phones or tablets?',
          a: 'No. NanoPrint is for Windows, macOS and Linux computers.',
        },
      ],
    },
  },
  developers: {
    title: 'NanoPrint for developers: print ESC/POS receipts from JavaScript',
    description:
      'How to print receipts on thermal printers from a web page with NanoPrint and qz-tray.js: compatible libraries, a code example and a live demo.',
    hero: {
      eyebrow: 'For developers',
      title: 'Print ESC/POS receipts from JavaScript',
      lead: 'NanoPrint has no SDK of its own: your page uses qz-tray.js, the client of QZ Tray, and sends ESC/POS bytes built with any encoder. If your app already prints with qz-tray.js, nothing needs to change.',
      guide: 'Integration guide',
      protocol: 'Protocol',
    },
    libraries: {
      title: 'Compatible libraries',
      lead: 'The ones we recommend, tested with NanoPrint.',
      license: 'License',
      items: [
        {
          name: 'qz-tray',
          href: 'https://www.npmjs.com/package/qz-tray',
          license: 'LGPL-2.1',
          role: 'Connects to NanoPrint and sends the jobs.',
          note: 'Version 2.2 or later. Every NanoPrint build is tested with qz-tray.js 2.3.0.',
        },
        {
          name: '@point-of-sale/receipt-printer-encoder',
          href: 'https://www.npmjs.com/package/@point-of-sale/receipt-printer-encoder',
          license: 'MIT',
          role: 'Builds ESC/POS bytes: text, code pages, QR codes, images and cut.',
          note: 'Pair it with qz-tray.',
        },
        {
          name: 'web-escpos-printer',
          href: 'https://www.npmjs.com/package/web-escpos-printer',
          license: 'MIT',
          role: 'Encoder and transports in one package.',
          note: 'Use its QZ Tray transport, installed from npm.',
        },
      ],
    },
    example: {
      title: 'Minimal example',
      lead: 'Install both libraries with `npm install qz-tray @point-of-sale/receipt-printer-encoder` and send a receipt:',
      code: `import qz from 'qz-tray';
import ReceiptPrinterEncoder from '@point-of-sale/receipt-printer-encoder';

await qz.websocket.connect();
const printer = await qz.printers.getDefault(); // or qz.printers.find('POS-58')

const bytes = new ReceiptPrinterEncoder({ columns: 32 }) // 58 mm; 48 for 80 mm
  .initialize()
  .line('Corner Shop')
  .line('Total: $12.34')
  .qrcode('https://example.com/receipt/123')
  .cut()
  .encode();

const config = qz.configs.create(printer, { jobName: 'Receipt 123' });
await qz.print(config, [
  { type: 'raw', format: 'command', flavor: 'base64', data: toBase64(bytes) },
]);

function toBase64(bytes) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}`,
      note: 'If the system\'s default printer is a virtual one (such as "Microsoft Print to PDF"), `getDefault()` returns `null`: find the printer by name or let the user pick it.',
    },
    identity: {
      title: "Your site's name and logo",
      text: "Optional: NanoPrint shows the name and logo your page sends in its permission prompt, always next to the page's real address. The logo is a PNG, JPEG or WebP data URL of up to 32 KiB.",
      code: `// Before connecting
qz.security.setCertificatePromise((resolve) => {
  resolve(JSON.stringify({
    nanoprint: { name: 'Corner Shop', icon: 'data:image/png;base64,…' },
  }));
});`,
    },
    demo: {
      title: 'Try it now',
      lead: 'With NanoPrint installed and running, this page connects to it, lists your printers and prints a sample receipt. It uses only qz-tray.js and ReceiptPrinterEncoder.',
      steps: [
        'NanoPrint will ask whether this site may print.',
        'Chrome may ask to allow access to devices on your local network: allow it so the page can reach NanoPrint.',
      ],
      connect: 'Connect to NanoPrint',
      connecting: 'Connecting…',
      connected: 'Connected to NanoPrint',
      disconnect: 'Disconnect',
      printer: 'Printer',
      paper: 'Paper',
      qr: 'Include a QR code',
      print: 'Print a sample receipt',
      printing: 'Printing…',
      printed: 'Receipt sent to {printer}.',
      noPrinters:
        "NanoPrint offers no printer to this site. Check the Printers tab in NanoPrint's panel.",
      notRunning:
        'Could not connect to NanoPrint. Make sure it is installed and running (icon next to the clock) and that secure sites are enabled in Preferences.',
      blocked:
        "NanoPrint refused the connection. If you blocked this site, unblock it in NanoPrint's Sites tab.",
      failed: 'Could not print: {message}',
      download: 'Download NanoPrint',
      ticket: {
        title: 'Sample receipt',
        items: [
          ['Coffee with milk', 1, 3.5],
          ['Croissant', 3, 2.25],
          ['Mineral water', 2, 1.8],
        ],
        total: 'TOTAL',
        thanks: 'Thanks for trying NanoPrint!',
        jobName: 'Sample receipt',
      },
    },
    requirements: {
      title: 'Before you integrate',
      items: [
        'https pages: the user enables "secure sites" in NanoPrint once; NanoPrint offers it right after installing.',
        'Chrome may ask for local network access before the page can connect to NanoPrint.',
        'Ports: the same as qz-tray.js (`wss://localhost:8181` and `ws://localhost:8182`). If the user picks others, pass them in `qz.websocket.connect({ port: … })`.',
        'Limits: 30 jobs a minute per site, up to 10 copies per job and messages of up to 2 MiB.',
        'Raw printing only: files, serial ports, USB, HID and pixel, HTML or PDF printing are not available.',
      ],
    },
    more: 'Full documentation on GitHub',
  },
  footer: {
    productOf: 'NanoPrint is a product of',
    links: {
      github: 'Source code',
      license: 'GPL-3.0 license',
      security: 'Security',
      licenses: 'Third-party licenses',
      developers: 'Developers',
    },
    legal: [
      'NanoPrint comes with absolutely no warranty, under the terms of the GPL-3.0 license.',
      'QZ Tray is a trademark of QZ Industries, LLC. NanoPrint is an independent project, not affiliated with or endorsed by QZ Industries; it mentions QZ Tray only to describe compatibility.',
      'Windows is a trademark of Microsoft Corporation. macOS is a trademark of Apple Inc. Linux is a registered trademark of Linus Torvalds. Other trademarks belong to their owners.',
    ],
    privacy: 'This site uses no cookies and no tracking.',
  },
  code: { copy: 'Copy', copied: 'Copied' },
};
