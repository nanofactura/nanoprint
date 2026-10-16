// Drives the real qz-tray.js client against a NanoPrint print server, the way a web page does.
// Used by the compatibility tests in src-tauri/src/server/tests.rs:
//   node qz-client.mjs <port>             ws://, as from an http:// page
//   node qz-client.mjs <port> <ca.pem>    wss://, as from an https:// page trusting the authority
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import qz from 'qz-tray';
import WebSocket from 'ws';

const port = Number(process.argv[2]);
const ca = process.argv[3] ? readFileSync(process.argv[3]) : undefined;

// Browsers always send an Origin header; Node only does when asked.
class BrowserWebSocket extends WebSocket {
  constructor(url, protocols) {
    super(url, protocols, { origin: 'https://shop.example', ca });
  }
}

qz.api.setWebSocketType(BrowserWebSocket);
qz.api.setPromiseType((resolver) => new Promise(resolver));
qz.api.setSha256Type((data) => createHash('sha256').update(data).digest('hex'));
// How a site presents itself to NanoPrint (see docs/integration.md).
qz.security.setCertificatePromise((resolve) => {
  resolve(JSON.stringify({ nanoprint: { name: 'Shop Example' } }));
});
// qz-tray.js logs connection progress; keep stdout for the result.
console.log = console.info = () => {};

await qz.websocket.connect({
  host: 'localhost',
  port: { secure: [port], insecure: [port] },
  usingSecure: ca !== undefined,
  retries: 0,
});
const printers = await qz.printers.find();
const printer = await qz.printers.find('pos');
const config = qz.configs.create(printer, { encoding: 'CP850', jobName: 'Ticket' });
await qz.print(config, [
  '\x1b@Año\n',
  { type: 'raw', format: 'command', flavor: 'base64', data: 'G0A=' },
]);
await qz.websocket.disconnect();
process.stdout.write(JSON.stringify({ printers, printer }));
// qz-tray.js keeps a keep-alive timer that would hold Node open for another minute.
process.exit(0);
