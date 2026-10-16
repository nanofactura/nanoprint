// Drives web-escpos-printer's QZ Tray transport against a NanoPrint print server, the way a web
// page using the library does. Used by the compatibility test in src-tauri/src/server/tests.rs:
//   node web-escpos-client.mjs <port>
import { createHash } from 'node:crypto';
import { register } from 'node:module';
import WebSocket from 'ws';

register('./qz-tray-esm.mjs', import.meta.url);
const { default: qz } = await import('./qz-tray-bridge.mjs');
const { default: WebEscposPrinter } = await import('web-escpos-printer');

const port = Number(process.argv[2]);

// Browsers always send an Origin header; Node only does when asked.
class BrowserWebSocket extends WebSocket {
  constructor(url, protocols) {
    super(url, protocols, { origin: 'https://shop.example' });
  }
}
qz.api.setWebSocketType(BrowserWebSocket);
qz.api.setPromiseType((resolver) => new Promise(resolver));
qz.api.setSha256Type((data) => createHash('sha256').update(data).digest('hex'));
// The library connects with qz-tray's defaults; point it at the test server.
const connect = qz.websocket.connect;
qz.websocket.connect = (options = {}) =>
  connect({
    ...options,
    host: 'localhost',
    port: { secure: [port], insecure: [port] },
    usingSecure: false,
    retries: 0,
  });
// How a site presents itself to NanoPrint: set on the same qz-tray module the library uses.
qz.security.setCertificatePromise((resolve) => {
  resolve(JSON.stringify({ nanoprint: { name: 'Web ESC/POS shop' } }));
});
// qz-tray.js and the library log progress; keep stdout for the result.
console.log = console.info = console.debug = () => {};

const printer = new WebEscposPrinter();
const printers = await printer.listQzPrinters();
const info = await printer.connect({ transport: 'qz', printerName: 'POS-58' });
await printer.printRaw([0x1b, 0x40, 0x48, 0x69, 0x0a]);
await printer.disconnect();
process.stdout.write(JSON.stringify({ printers, connected: Boolean(info) }));
// qz-tray.js keeps a keep-alive timer that would hold Node open for another minute.
process.exit(0);
