// Renders the site's icons and social preview images from the brand masters in ../assets/brand.
// Run `npm run assets` after changing a master or a tagline; the generated files are committed.
// The taglines use the system's sans-serif font (Segoe UI on Windows): check the result by eye.
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { Resvg } from '@resvg/resvg-js';

const BRAND = '../assets/brand';
const STATIC = 'static';
const BACKGROUND = '#0b0d14';

/** The markup inside an SVG file's root element, and its viewBox. */
function parts(/** @type {string} */ file) {
  const svg = readFileSync(`${BRAND}/${file}`, 'utf8');
  const viewBox = /viewBox="([^"]+)"/.exec(svg)?.[1];
  const inner = /<svg[^>]*>([\s\S]*)<\/svg>/.exec(svg)?.[1];
  if (!viewBox || !inner) throw new Error(`${file}: no viewBox or content`);
  return { viewBox, inner };
}

function render(/** @type {string} */ svg, /** @type {number} */ width, /** @type {string} */ out) {
  const png = new Resvg(svg, {
    fitTo: { mode: 'width', value: width },
    font: { loadSystemFonts: true, defaultFontFamily: 'Segoe UI' },
  })
    .render()
    .asPng();
  writeFileSync(`${STATIC}/${out}`, png);
  console.log(`wrote ${STATIC}/${out}`);
}

const isologo = parts('isologo-nanoprint.svg');
const wordmark = parts('logo-nanoprint-dark.svg');

mkdirSync(`${STATIC}/brand`, { recursive: true });
copyFileSync(`${BRAND}/logo-nanoprint-dark.svg`, `${STATIC}/brand/logo-nanoprint-dark.svg`);
copyFileSync(`${BRAND}/isologo-nanoprint.svg`, `${STATIC}/brand/isologo-nanoprint.svg`);

// The app's own icon file already holds 16 to 256 px versions of the mark.
copyFileSync('../src-tauri/icons/icon.ico', `${STATIC}/favicon.ico`);

// The isologo fits a square with a little room around it.
const [, , w, h] = isologo.viewBox.split(/\s+/).map(Number);
const side = h * 1.08;
const squareBox = `${String((w - side) / 2)} ${String((h - side) / 2)} ${String(side)} ${String(side)}`;
writeFileSync(
  `${STATIC}/favicon.svg`,
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${squareBox}">${isologo.inner}</svg>\n`,
);
console.log(`wrote ${STATIC}/favicon.svg`);

// iOS fills transparent areas with black: draw the mark on the site's background.
render(
  `<svg xmlns="http://www.w3.org/2000/svg" width="180" height="180" viewBox="0 0 180 180">
    <rect width="180" height="180" fill="${BACKGROUND}"/>
    <svg x="30" y="22" width="120" height="136" viewBox="${isologo.viewBox}">${isologo.inner}</svg>
  </svg>`,
  180,
  'apple-touch-icon.png',
);

// The logo the developer demo sends to NanoPrint (shown in its permission prompt).
render(
  `<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96" viewBox="${squareBox}">${isologo.inner}</svg>`,
  96,
  'brand/demo-logo.png',
);

/** Social preview, 1200 × 630, one per language. */
function socialImage(/** @type {string} */ tagline, /** @type {string} */ detail) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
    <defs>
      <radialGradient id="glow" cx="0.82" cy="0.18" r="0.7">
        <stop offset="0" stop-color="#4b5ff7" stop-opacity="0.45"/>
        <stop offset="1" stop-color="#4b5ff7" stop-opacity="0"/>
      </radialGradient>
      <radialGradient id="warm" cx="0.1" cy="1" r="0.6">
        <stop offset="0" stop-color="#d5892f" stop-opacity="0.22"/>
        <stop offset="1" stop-color="#d5892f" stop-opacity="0"/>
      </radialGradient>
    </defs>
    <rect width="1200" height="630" fill="${BACKGROUND}"/>
    <rect width="1200" height="630" fill="url(#glow)"/>
    <rect width="1200" height="630" fill="url(#warm)"/>
    <svg x="80" y="90" width="420" height="170" viewBox="${wordmark.viewBox}">${wordmark.inner}</svg>
    <text x="96" y="370" font-family="Segoe UI, sans-serif" font-size="54" font-weight="600" fill="#e9ebf3">${tagline[0]}</text>
    <text x="96" y="438" font-family="Segoe UI, sans-serif" font-size="54" font-weight="600" fill="#e9ebf3">${tagline[1]}</text>
    <text x="96" y="530" font-family="Segoe UI, sans-serif" font-size="30" fill="#e9a957">${detail}</text>
    <svg x="900" y="250" width="220" height="272" viewBox="${isologo.viewBox}">${isologo.inner}</svg>
  </svg>`;
}

render(
  socialImage(
    ['Imprimí tickets desde la web', 'en tu impresora térmica'],
    'Gratis y de código abierto · Windows, macOS y Linux',
  ),
  1200,
  'og-es.png',
);
render(
  socialImage(
    ['Print receipts from the web', 'to your thermal printer'],
    'Free and open source · Windows, macOS and Linux',
  ),
  1200,
  'og-en.png',
);
