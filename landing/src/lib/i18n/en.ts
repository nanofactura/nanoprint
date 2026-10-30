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
    starting: 'Your download is starting…',
    nextSteps: 'When the download finishes:',
    retry: "Didn't start? Download again",
  },
  home: {
    title: 'NanoPrint: print receipts and kitchen orders from the web to your thermal printer',
    description:
      'Free, open-source app for Windows, macOS and Linux that prints receipts and kitchen orders from your web app to ESC/POS thermal printers. Each site asks for permission once, not on every receipt.',
    hero: {
      title: 'Print receipts from the web, straight to your thermal printer',
      lead: 'NanoPrint is a free, lightweight app that connects your web app to the receipt printers of the computer. Each site asks for permission once, not on every receipt.',
      platforms: 'Windows · macOS · Linux',
      free: 'Free',
      openSource: 'Open source',
      videoLabel: 'Play the animation again: a thermal printer prints a NanoPrint receipt.',
    },
    features: {
      title: 'Built for the counter',
      lead: 'What a shop or a kitchen needs: receipts and orders that print fast, with no odd setup and no risk.',
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
          title: 'Lightweight and free',
          text: 'An installer of a few megabytes, free and with no sign-up. It lives in the system tray and starts with the computer.',
          points: [
            'No cost, subscriptions or usage limits',
            'No Java or other software needed',
            'Installs without administrator rights on Windows',
          ],
        },
        {
          title: 'Secure, not annoying',
          text: 'No site prints without your permission. You give it once per site, and it does not ask again on every receipt.',
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
          text: 'The first time a site wants to print, NanoPrint asks. You answer once: it does not ask on every receipt.',
        },
        {
          title: 'Print',
          text: "The receipt or kitchen order goes straight to the printer, without the browser's print dialog.",
        },
      ],
    },
    integrate: {
      title: 'Does your web app print receipts or kitchen orders?',
      text: 'Offer NanoPrint to your users: they install it for free, allow your site once, and your receipts go straight to their printer.',
      points: [
        'A few lines of JavaScript, with well-known libraries',
        'If your app already uses qz-tray.js, it works unchanged',
        'Free for you and your users, with no per-seat licenses',
        'Your app can have any license: NanoPrint is a separate program',
      ],
      button: 'Developer guide',
      code: `import qz from 'qz-tray';

await qz.websocket.connect();
const config = qz.configs.create('POS-58');
await qz.print(config, [{ type: 'raw', format: 'command', flavor: 'base64', data }]);`,
    },
    printers: {
      title: 'Supported printers',
      text: 'Thermal receipt and kitchen printers that understand ESC/POS, 58 or 80 mm wide, connected by USB or over the network. ESC/POS is the language of most receipt printers.',
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
        {
          os: 'Network printer',
          text: "It needs no driver. Add it in NanoPrint's Printers tab by IP address and port (usually 9100), and give it a fixed IP address in your router so it does not change. It can also be installed in the system like any other printer.",
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
    signing: {
      title: 'Code signing policy',
      items: [
        'Windows installers are going to be signed through the SignPath Foundation program for open-source projects. The application is in progress: current releases are not signed yet. "Free code signing provided by SignPath.io, certificate by SignPath Foundation".',
        'Every installer is built by GitHub Actions from the published source, and a project maintainer approves each signing.',
        'Author, reviewer and approver: Mariano Paulin (@marianopaulin02).',
        'Privacy: NanoPrint does not transfer any information to other networked systems unless specifically requested by the person using it. The one exception, from version 0.2.0, is checking GitHub for updates, which can be turned off in Preferences.',
      ],
    },
    openSource: {
      title: 'Open source',
      text: 'NanoPrint is free software under the GPL-3.0 license. All of its code is on GitHub: review it, report issues or contribute.',
      button: 'View on GitHub',
      alternativeTo: 'NanoPrint on AlternativeTo',
    },
    faq: {
      title: 'Frequently asked questions',
      items: [
        {
          q: 'Is NanoPrint free?',
          a: 'Yes, completely: for personal and commercial use, with no sign-up, subscriptions or limits. It is free software under the GPL-3.0 license.',
        },
        {
          q: 'Will it ask for permission every time I print?',
          a: 'No. NanoPrint asks once per site. For secure (https) sites the answer is kept; for sites without a padlock (http) it asks again when NanoPrint restarts, because another computer on the network could pose as them.',
        },
        {
          q: 'Which printers work?',
          a: "ESC/POS thermal receipt and kitchen printers, 58 or 80 mm, connected by USB or over the network: installed in the system or, for network printers, added in NanoPrint by IP address. NanoPrint's panel prints a test page and chooses which printers sites can see.",
        },
        {
          q: 'Do I need a special driver?',
          a: "No. A network printer can be added in NanoPrint by IP address, with no driver. Otherwise the printer must be installed in the operating system, with the manufacturer's driver or a generic one. NanoPrint sends the commands as they are.",
        },
        {
          q: 'Can a web page print without me noticing?',
          a: 'No. Every site must ask for permission, and you decide. NanoPrint only accepts connections from this computer, and permissions can be revoked from its panel.',
        },
        {
          q: 'Do I have to keep NanoPrint open?',
          a: 'NanoPrint starts with the computer and stays in the system tray, out of the way. If you quit it, sites that integrate it can open it again: your browser asks you first.',
        },
        {
          q: 'What does NanoPrint change on my computer?',
          a: 'It adds an entry to start with the computer (turn it off in Preferences) and the nanoprint:// link so sites can open it, always after you confirm. If you enable secure sites, it installs its own certificate for your user, after the system asks you. It keeps its settings and history in your profile. The Windows uninstaller removes everything when you choose to delete the application data.',
        },
        {
          q: 'Does NanoPrint store what I print?',
          a: 'No. Its history keeps the date, site, printer and result of each job for 30 days, never the content. NanoPrint only connects to GitHub to look for updates, and you can turn that off.',
        },
        {
          q: 'Why do Windows and macOS warn me when installing?',
          a: 'Because the installer does not have a commercial digital signature yet, as is common for new open-source projects. You can verify that each installer was built from the source published on GitHub.',
        },
        {
          q: 'Does it work with apps that print through QZ Tray today?',
          a: "Usually yes: NanoPrint understands the qz-tray.js protocol for printing receipts, so those apps tend to work unchanged. QZ Tray's features beyond receipt printing are not included.",
        },
        {
          q: 'Can I build NanoPrint into my commercial app?',
          a: 'Yes. Your app only talks to NanoPrint, which is a separate program, so it can have any license. The developer page explains how to integrate it.',
        },
        {
          q: 'Does it work over Remote Desktop?',
          a: 'NanoPrint prints to the printers installed on the computer it runs on. Printers redirected by Remote Desktop through "Easy Print" cannot receive receipts: install the printer\'s driver on the remote computer and turn off Easy Print.',
        },
        {
          q: 'Can I print from a phone or a tablet?',
          a: 'Not yet. NanoPrint installs on Windows, macOS and Linux computers and only accepts connections from that same computer. Printing from other devices on the local network, each one approved by you, is planned for a future version.',
        },
      ],
    },
  },
  developers: {
    title: 'NanoPrint for developers: print ESC/POS receipts and kitchen orders from JavaScript',
    description:
      'How to print receipts and kitchen orders on thermal printers from your web app with NanoPrint: compatible JavaScript libraries, a code example and a live demo.',
    hero: {
      eyebrow: 'For developers',
      title: 'Print ESC/POS receipts from JavaScript',
      lead: 'Your page connects to NanoPrint through qz-tray.js and sends it ESC/POS receipts built with the encoder you prefer. If your app already uses qz-tray.js, it works unchanged.',
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
    wake: {
      title: 'When NanoPrint is closed',
      text: 'NanoPrint starts with the computer, but the user may have quit it. Your page can start it again with the `nanoprint://start` link: the browser asks before opening it, and NanoPrint starts in the tray, with no window.',
      code: `async function openNanoPrint() {
  window.location.href = 'nanoprint://start';
  // NanoPrint needs a moment to start
  await qz.websocket.connect({ retries: 15, delay: 1 });
}`,
      note: 'Offer it as an "Open NanoPrint" button after a failed connection: browsers only open apps after a click. The link only starts NanoPrint, carries no data, and the site still needs permission to print.',
    },
    demo: {
      title: 'Try it now',
      lead: 'With NanoPrint installed and running, this page connects to it, lists your printers and prints a sample receipt. It uses only qz-tray.js and ReceiptPrinterEncoder.',
      steps: [
        'NanoPrint will ask whether this site may print.',
        'Chrome may ask to allow access to devices on your local network: allow it so the page can reach NanoPrint.',
      ],
      connect: 'Connect to NanoPrint',
      wake: 'Open NanoPrint',
      wakeFailed:
        "NanoPrint didn't open. If it isn't installed, download it; if your browser asked and you declined, try again.",
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
        'Could not connect to NanoPrint. If it is installed but closed, open it with the button (your browser will ask you to confirm). If it is running, check that secure sites are enabled in Preferences.',
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
    adopt: {
      title: 'Offer it to your users',
      lead: 'NanoPrint is meant to be used by other projects.',
      items: [
        "Link to `nanoprint.nanofactura.com` for the download: it detects each user's system and walks them through the installation.",
        'It is free for you and your users: no per-seat licenses, subscriptions or usage limits.',
        'Your app only talks to NanoPrint, a separate program, so it can have any license, including a proprietary one.',
        'If something is missing or broken, tell us on GitHub: real integrations help us decide what comes next.',
      ],
      issues: 'Open an issue on GitHub',
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
      signing: 'Code signing policy',
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
