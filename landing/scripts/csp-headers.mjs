// Runs after `vite build`: copies each page's Content-Security-Policy, which SvelteKit writes into a
// <meta> tag with the hashes of that page's inline scripts, into build/_headers as an HTTP header
// for the page's path. A header applies before the browser parses anything and can also carry
// frame-ancestors; the <meta> stays as a second layer.
import { appendFileSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

const BUILD = 'build';
const META = /<meta http-equiv="content-security-policy" content="([^"]+)"/;

/** Every HTML file under `dir`. */
function pages(/** @type {string} */ dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return pages(path);
    return name.endsWith('.html') ? [path] : [];
  });
}

const rules = [];
for (const file of pages(BUILD)) {
  const policy = META.exec(readFileSync(file, 'utf8'))?.[1];
  if (!policy) throw new Error(`${file}: no Content-Security-Policy <meta>`);
  const url = `/${relative(BUILD, file).split(sep).join('/')}`;
  // build/en/index.html is served at /en/ (and redirects from /en and /en/index.html).
  const paths = url.endsWith('/index.html') ? [url.slice(0, -'index.html'.length), url] : [url];
  const header = `${policy.replaceAll('&#39;', "'")}; frame-ancestors 'none'`;
  for (const path of paths) rules.push(`${path}\n  Content-Security-Policy: ${header}\n`);
}

appendFileSync(
  join(BUILD, '_headers'),
  `\n# Per-page policies, written by scripts/csp-headers.mjs.\n${rules.join('\n')}`,
);
console.log(`wrote the Content-Security-Policy of ${String(rules.length)} paths to build/_headers`);
