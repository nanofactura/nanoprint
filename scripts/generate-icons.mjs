// Renders every raster icon from the SVG masters in assets/brand. Run `npm run icons` after
// changing a master; the generated files are committed.
import { execFileSync } from 'node:child_process';
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { Resvg } from '@resvg/resvg-js';

const BRAND = 'assets/brand';
const ICONS = 'src-tauri/icons';
const APP_ICON = `${BRAND}/isologo-nanoprint-1024.png`;

/**
 * Returns `svg` with a square viewBox centered on the original, keeping `padding` (0–0.5) free.
 * @param {string} svg
 * @param {number} [padding]
 * @returns {string}
 */
function squared(svg, padding = 0) {
  const match = /viewBox="([^"]+)"/.exec(svg);
  if (!match) throw new Error('SVG master without a viewBox');
  const [x, y, width, height] = match[1].trim().split(/\s+/).map(Number);
  const side = Math.max(width, height) / (1 - 2 * padding);
  const box = [x - (side - width) / 2, y - (side - height) / 2, side, side];
  return svg.replace(match[0], `viewBox="${box.map((n) => n.toFixed(3)).join(' ')}"`);
}

/**
 * @param {string} svg
 * @param {number} size Width and height in pixels.
 * @param {string} file
 */
function render(svg, size, file) {
  const png = new Resvg(svg, { fitTo: { mode: 'width', value: size } }).render().asPng();
  writeFileSync(file, png);
  console.log(`${file} (${String(size)}×${String(size)})`);
}

const isologo = readFileSync(`${BRAND}/isologo-nanoprint.svg`, 'utf8');
const mono = readFileSync(`${BRAND}/isologo-nanoprint-mono.svg`, 'utf8');

// Tray, Windows (16–48 px for 100–300% display scale) and Linux panels (22–48 px). The mark fills
// the whole height: at these sizes every pixel counts.
mkdirSync(`${ICONS}/tray`, { recursive: true });
for (const size of [16, 20, 22, 24, 32, 40, 48]) {
  render(squared(isologo), size, `${ICONS}/tray/tray-${String(size)}.png`);
}

// Tray, macOS: template images (black + alpha) that the menu bar tints for light and dark modes.
render(squared(mono), 18, `${ICONS}/tray/tray-template.png`);
render(squared(mono), 36, `${ICONS}/tray/tray-template@2x.png`);

// App icon master, then `tauri icon` derives the .ico, .icns and PNG sizes from it.
render(squared(isologo, 0.06), 1024, APP_ICON);
const tauri = createRequire(import.meta.url).resolve('@tauri-apps/cli/tauri.js');
execFileSync(process.execPath, [tauri, 'icon', APP_ICON, '--output', ICONS], { stdio: 'inherit' });
// NanoPrint is desktop-only and is not distributed through the Microsoft Store.
for (const dir of ['android', 'ios']) rmSync(`${ICONS}/${dir}`, { recursive: true, force: true });
for (const file of readdirSync(ICONS)) {
  if (/^(Square\d+x\d+Logo|StoreLogo)\.png$/.test(file)) rmSync(`${ICONS}/${file}`);
}
