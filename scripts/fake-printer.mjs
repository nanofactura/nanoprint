// A network receipt printer for development: listens for raw TCP print jobs (port 9100, like a
// real Ethernet or Wi-Fi printer), shows each one in the terminal as a ticket preview, and can save
// its bytes. ESC/POS commands are named instead of printed; text is decoded with the code page the
// job selects (ESC t). It answers status requests (DLE EOT) as an online printer with paper.
//
//   npm run fake-printer                        # 127.0.0.1:9100
//   npm run fake-printer -- --port 9101 --save jobs --hex
//   npm run fake-printer -- --host 0.0.0.0      # reachable from other devices on the network
//
// Then add it in NanoPrint (Printers → Add network printer) with that address and port.
import { mkdirSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:net';
import { join } from 'node:path';
import process from 'node:process';
import { parseArgs } from 'node:util';

const { values: options } = parseArgs({
  options: {
    host: { type: 'string', default: '127.0.0.1' },
    port: { type: 'string', default: '9100' },
    save: { type: 'string' },
    hex: { type: 'boolean', default: false },
  },
});

/** Characters 0x80–0xFF of the DOS code pages receipt printers use. */
const CP437 = Array.from(
  'ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ',
);
const CP850 = Array.from(
  'ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜø£Ø×ƒáíóúñÑªº¿®¬½¼¡«»░▒▓│┤ÁÂÀ©╣║╗╝¢¥┐└┴┬├─┼ãÃ╚╔╩╦╠═╬¤ðÐÊËÈıÍÎÏ┘┌█▄¦Ì▀ÓßÔÒõÕµþÞÚÛÙýÝ¯´­±‗¾¶§÷¸°¨·¹³²■ ',
);
/** CP850 with the euro sign in place of the dotless i. */
const CP858 = CP850.map((char, index) => (index === 0x55 ? '€' : char));
for (const table of [CP437, CP850, CP858]) {
  if (table.length !== 128) throw new Error('a code page table must have 128 characters');
}
const windows1252 = new TextDecoder('windows-1252');

/**
 * The decoder of the code page table ESC t selects. Tables this preview does not know fall back
 * to CP437, the printers' default.
 * @param {number | undefined} table
 * @returns {(byte: number) => string}
 */
function decoder(table) {
  switch (table) {
    case 2:
      return (byte) => CP850[byte - 0x80] ?? '?';
    case 16:
      return (byte) => windows1252.decode(Uint8Array.of(byte));
    case 19:
      return (byte) => CP858[byte - 0x80] ?? '?';
    default:
      return (byte) => CP437[byte - 0x80] ?? '?';
  }
}

const ESC = 0x1b;
const GS = 0x1d;
const DLE = 0x10;
const FS = 0x1c;

/**
 * ESC commands with a fixed number of parameter bytes, by their second byte (-1: up to NUL).
 * @type {Partial<Record<number, number>>}
 */
const ESC_FIXED = {
  0x20: 1, // right-side spacing
  0x21: 1, // print mode
  0x24: 2, // absolute position
  0x25: 1, // user-defined characters on/off
  0x2d: 1, // underline
  0x32: 0, // default line spacing
  0x33: 1, // line spacing
  0x3d: 1, // peripheral device
  0x3f: 1, // cancel user-defined character
  0x40: 0, // initialize
  0x44: -1, // tab stops
  0x45: 1, // bold
  0x47: 1, // double strike
  0x4a: 1, // feed n dots
  0x4d: 1, // font
  0x52: 1, // international character set
  0x53: 0, // standard mode
  0x54: 1, // print direction
  0x56: 1, // rotate 90°
  0x5c: 2, // relative position
  0x61: 1, // justification
  0x63: 2, // panel buttons, paper sensors (ESC c n m)
  0x65: 1, // print and reverse feed
  0x70: 3, // drawer kick pulse
  0x74: 1, // code page
  0x7b: 1, // upside down
};
/** @type {Partial<Record<number, number>>} */
const GS_FIXED = {
  0x21: 1, // character size
  0x42: 1, // reverse white/black
  0x48: 1, // barcode text position
  0x49: 1, // printer ID request (answered)
  0x4c: 2, // left margin
  0x50: 2, // motion units
  0x57: 2, // print area width
  0x61: 1, // automatic status back
  0x62: 1, // smoothing
  0x66: 1, // barcode font
  0x68: 1, // barcode height
  0x72: 1, // status request (answered)
  0x77: 1, // barcode module width
};

/**
 * A ticket preview of `data`, the commands it names, and the bytes a real printer would answer.
 * @param {Uint8Array} data
 */
function preview(data) {
  /** @type {string[]} */
  const lines = [''];
  /** @type {string[]} */
  const marks = [];
  /** @type {number[]} */
  const replies = [];
  let decode = decoder(0);

  /** @param {string} text */
  const write = (text) => {
    lines[lines.length - 1] += text;
  };
  /** @param {string} text */
  const mark = (text) => {
    marks.push(text);
    write(`⟨${text}⟩`);
  };
  /** @param {number} at */
  const byte = (at) => data[at] ?? 0;
  /** @param {number} at */
  const u16 = (at) => byte(at) + byte(at + 1) * 256;
  /** @param {number} from @param {number} to */
  const ascii = (from, to) => String.fromCharCode(...data.subarray(from, to));

  /**
   * Reads the command at `at` and returns where the next byte starts.
   * @param {number} at
   * @returns {number}
   */
  const command = (at) => {
    const [first, second] = [byte(at), byte(at + 1)];
    if (first === DLE) {
      // DLE EOT n: real-time status; 0x12 means online, cover closed, paper present.
      if (second === 0x04) replies.push(0x12);
      // DLE DC4 fn m t: real-time requests (drawer pulse, buffer clear).
      return at + (second === 0x14 ? 5 : 3);
    }
    if (first === FS) {
      // FS p n m: a logo stored in the printer. Others (Kanji mode on and off) take nothing.
      return at + (second === 0x70 ? 4 : 2);
    }
    if (first === ESC) {
      if (second === 0x2a) {
        // ESC * m nL nH d…: bit image, 1 or 3 bytes per column.
        const columns = u16(at + 3);
        mark(`image ${String(columns)} dots wide`);
        return at + 5 + columns * (byte(at + 2) > 1 ? 3 : 1);
      }
      if (second === 0x64) {
        // ESC d n: print and feed n lines.
        for (let line = 0; line < byte(at + 2); line += 1) lines.push('');
        return at + 3;
      }
      if (second === 0x74) decode = decoder(byte(at + 2));
      if (second === 0x70) mark('open drawer');
      const length = ESC_FIXED[second];
      if (length === -1) {
        const end = data.indexOf(0, at + 2);
        return end === -1 ? data.length : end + 1;
      }
      if (length !== undefined) return at + 2 + length;
    }
    if (first === GS) {
      if (second === 0x56) {
        // GS V m [n]: cut; the m values 65 and 66 take a feed amount too.
        const m = byte(at + 2);
        mark(m === 1 || m === 49 || m === 66 ? 'partial cut' : 'cut');
        return at + 3 + (m === 65 || m === 66 ? 1 : 0);
      }
      if (second === 0x6b) {
        // GS k m …: barcode, its data up to NUL (m 0–6) or after its length (m 65–79).
        const m = byte(at + 2);
        if (m <= 6) {
          const end = data.indexOf(0, at + 3);
          const stop = end === -1 ? data.length : end;
          mark(`barcode ${ascii(at + 3, stop)}`);
          return stop + 1;
        }
        const length = byte(at + 3);
        mark(`barcode ${ascii(at + 4, at + 4 + length)}`);
        return at + 4 + length;
      }
      if (second === 0x28) {
        // GS ( fn pL pH …: QR codes (GS ( k), graphics (GS ( L) and other functions.
        const length = u16(at + 3);
        const body = data.subarray(at + 5, at + 5 + length);
        if (byte(at + 2) === 0x6b && body[0] === 49 && body[1] === 80) {
          mark(`QR ${new TextDecoder().decode(body.subarray(3))}`);
        } else if (byte(at + 2) === 0x4c && body[1] === 112) {
          mark(`image ${String(u16(at + 11))}×${String(u16(at + 13))}`);
        }
        return at + 5 + length;
      }
      if (second === 0x76 && byte(at + 2) === 0x30) {
        // GS v 0 m xL xH yL yH d…: raster image, x bytes (8 dots each) by y dots.
        const [width, height] = [u16(at + 4), u16(at + 6)];
        mark(`image ${String(width * 8)}×${String(height)}`);
        return at + 8 + width * height;
      }
      if (second === 0x38 && byte(at + 2) === 0x4c) {
        // GS 8 L p1–p4 …: graphics with a 32-bit length.
        mark('image');
        return at + 7 + u16(at + 3) + u16(at + 5) * 65536;
      }
      // GS I n: printer ID; GS r n: paper status (0: paper present).
      if (second === 0x49) replies.push(0x20);
      if (second === 0x72) replies.push(0x00);
      const length = GS_FIXED[second];
      if (length !== undefined) return at + 2 + length;
    }
    const hex = [first, second].map((b) => b.toString(16).padStart(2, '0')).join(' ');
    mark(`unknown ${hex}`);
    return at + 2;
  };

  let i = 0;
  while (i < data.length) {
    const next = byte(i);
    if (next === 0x0a) {
      lines.push('');
      i += 1;
    } else if (next === 0x0d) {
      i += 1;
    } else if (next === 0x09) {
      write('    ');
      i += 1;
    } else if (next === ESC || next === GS || next === FS || next === DLE) {
      i = command(i);
    } else if (next < 0x20) {
      mark(`0x${next.toString(16).padStart(2, '0')}`);
      i += 1;
    } else {
      write(next < 0x80 ? String.fromCharCode(next) : decode(next));
      i += 1;
    }
  }
  return { text: lines.join('\n').replace(/\n+$/, ''), marks, replies };
}

/** @param {Buffer} data @param {string} from @param {number} number */
function show(data, from, number) {
  const { text, marks } = preview(data);
  const lines = text.split('\n');
  const width = Math.max(32, ...lines.map((line) => Array.from(line).length));
  const rule = '─'.repeat(width + 2);
  console.log(`\nJob ${String(number)} from ${from}: ${String(data.length)} bytes`);
  console.log(`┌${rule}┐`);
  for (const line of lines) {
    console.log(`│ ${line}${' '.repeat(width - Array.from(line).length)} │`);
  }
  console.log(`└${rule}┘`);
  if (marks.length) console.log(`Commands: ${marks.join(', ')}`);
  if (options.hex) console.log(data.toString('hex').replace(/(..)(?!$)/g, '$1 '));
  if (options.save) {
    mkdirSync(options.save, { recursive: true });
    const file = join(options.save, `job-${String(number).padStart(3, '0')}.bin`);
    writeFileSync(file, data);
    console.log(`Saved to ${file}`);
  }
}

let jobs = 0;
const server = createServer((socket) => {
  const from = `${socket.remoteAddress ?? '?'}:${String(socket.remotePort ?? '?')}`;
  /** @type {Buffer[]} */
  const chunks = [];
  socket.on('data', (chunk) => {
    chunks.push(chunk);
    // Status requests are answered while the job arrives, as a real printer does.
    const { replies } = preview(chunk);
    if (replies.length) socket.write(Uint8Array.from(replies));
  });
  socket.on('error', (error) => {
    console.error(`${from}: ${error.message}`);
  });
  socket.on('end', () => {
    socket.end();
    const data = Buffer.concat(chunks);
    if (data.length) {
      jobs += 1;
      show(data, from, jobs);
    }
  });
});

server.on('error', (error) => {
  console.error(`Could not listen on ${options.host}:${options.port}: ${error.message}`);
  process.exit(1);
});
server.listen(Number(options.port), options.host, () => {
  console.log(`Fake network printer on ${options.host}:${options.port}. Ctrl+C stops it.`);
});
