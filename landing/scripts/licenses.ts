// Vite plugin: writes third-party-licenses.txt next to the built site, with the license and full
// text of every package whose code reaches the browser. It reads the client bundle itself, so the
// list matches what ships: build tools and server-only code are left out.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import type { Plugin } from 'vite';

const root = resolve(import.meta.dirname, '..');

/** Shipped through CSS (the fonts) or bundled inside another package's build (the encoder's). */
const ALSO_SHIPPED = ['@fontsource-variable/geist', '@fontsource-variable/geist-mono'];
const PREBUNDLED = ['@point-of-sale/receipt-printer-encoder'];

const mit = (holder: string) => `MIT License

Copyright (c) ${holder}

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or
substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.`;

/** License texts for packages that declare a license but ship no license file. */
const TEXTS: Record<string, () => string> = {
  // From https://www.gnu.org/licenses/old-licenses/lgpl-2.1.txt
  'qz-tray': () => readFileSync(join(root, 'licenses', 'LGPL-2.1.txt'), 'utf8'),
  '@canvas/image-data': () => mit('the authors of https://github.com/node-gfx/image-data'),
  'resize-image-data': () => mit('Linus Unnebäck'),
};

/** The package directory a module belongs to, from its path. */
function packageDir(id: string): string | undefined {
  const path = id.replace(/^\0/, '').split('?')[0].replaceAll('\\', '/');
  const match = /^(.*\/node_modules\/(?:@[^/]+\/)?[^/]+)\//.exec(path);
  return match?.[1];
}

/** The directory of package `name` as Node would resolve it from `dir`. */
function resolvePackage(name: string, dir: string): string {
  for (let current = dir; ; current = resolve(current, '..')) {
    const candidate = join(current, 'node_modules', name);
    if (existsSync(join(candidate, 'package.json'))) return candidate;
    if (resolve(current, '..') === current) throw new Error(`cannot find ${name} from ${dir}`);
  }
}

/** `dir` and every package it depends on at run time, recursively. */
function dependencyTree(dir: string, found = new Set<string>()): Set<string> {
  if (found.has(dir)) return found;
  found.add(dir);
  const manifest = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8')) as {
    dependencies?: Record<string, string>;
  };
  for (const name of Object.keys(manifest.dependencies ?? {})) {
    dependencyTree(resolvePackage(name, dir), found);
  }
  return found;
}

function notice(dirs: Set<string>): string {
  const packages = new Map<string, { license: string; text: string }>();
  const missing: string[] = [];
  for (const dir of dirs) {
    const manifest = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8')) as {
      name: string;
      version: string;
      license?: string;
    };
    const file = readdirSync(dir).find((name) => /^(licen[cs]e|copying)(\.(md|txt))?$/i.test(name));
    const text = file ? readFileSync(join(dir, file), 'utf8') : TEXTS[manifest.name]?.();
    if (!manifest.license || !text) {
      missing.push(manifest.name);
      continue;
    }
    packages.set(`${manifest.name} ${manifest.version}`, {
      license: manifest.license,
      text: text.trim(),
    });
  }
  if (missing.length > 0) throw new Error(`no license text for: ${missing.join(', ')}`);

  const ids = [...packages.keys()].sort();
  const rule = '='.repeat(78);
  return [
    'Third-party software in nanoprint.nanofactura.com',
    'Software de terceros en nanoprint.nanofactura.com',
    '',
    "The site's own code is part of NanoPrint, GPL-3.0-only: https://github.com/nanofactura/nanoprint",
    "qz-tray is served unmodified, in its own file, and only loads in the developer page's demo.",
    '',
    ...ids.map((id) => `- ${id} (${packages.get(id)?.license ?? ''})`),
    '',
    ...ids.flatMap((id) => [
      rule,
      `${id} (${packages.get(id)?.license ?? ''})`,
      rule,
      '',
      packages.get(id)?.text ?? '',
      '',
    ]),
  ].join('\n');
}

export function thirdPartyLicenses(): Plugin {
  return {
    name: 'nanoprint-third-party-licenses',
    apply: 'build',
    generateBundle(_options, bundle) {
      // The server build only renders pages ahead of time; none of it reaches visitors.
      if (this.environment.name !== 'client') return;
      const dirs = new Set<string>();
      for (const output of Object.values(bundle)) {
        if (output.type !== 'chunk') continue;
        for (const id of output.moduleIds) {
          const dir = packageDir(id);
          if (dir) dirs.add(dir);
        }
      }
      for (const name of ALSO_SHIPPED) dirs.add(join(root, 'node_modules', name));
      for (const name of PREBUNDLED) {
        for (const dir of dependencyTree(join(root, 'node_modules', name))) dirs.add(dir);
      }
      this.emitFile({ type: 'asset', fileName: 'third-party-licenses.txt', source: notice(dirs) });
    },
  };
}
