// Writes generated/third-party-notices.html: NanoPrint's own license (GPL-3.0, with a link to the
// source of this version) and the license of every third-party component inside the executable,
// as those licenses require. Release builds embed the page and the panel shows it ("Third-party
// licenses"); see `thirdPartyNotices` in vite.config.ts.
//
//   Rust crates: cargo-about with about.toml, for every platform NanoPrint ships on.
//   JavaScript: the packages in package.json "dependencies" and theirs, which is what the UI
//   bundles (tooling stays in "devDependencies"), plus CSS taken from `cssOnly` packages.
//
// Fails on a license that is not accepted or a package without license text.
// Usage: npm run notices   (needs cargo-about on PATH, or its path in CARGO_ABOUT)
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import process from 'node:process';

/** @typedef {{ name: string, text: string, users: string[] }} License */

const root = resolve(import.meta.dirname, '..');
const output = join(root, 'generated', 'third-party-notices.html');

/** Accepted JavaScript licenses: those of about.toml and deny.toml, plus the font license. */
const accepted = new Set([
  '0BSD',
  'Apache-2.0',
  'BSD-3-Clause',
  'CC0-1.0',
  'ISC',
  'MIT',
  'MPL-2.0',
  'OFL-1.1',
  'Zlib',
]);
/** Development packages whose CSS the UI imports (and nothing else). */
const cssOnly = ['shadcn-svelte'];
const licenseFile = /^(licen[cs]e|copying|notice)/i;

/** The MIT license text, for packages that declare MIT without publishing the text. */
const mit = (/** @type {string} */ holder) => `MIT License

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
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
`;

/** Packages with incomplete metadata, checked by hand against the package and its repository. */
/** @type {Partial<Record<string, { license: string, text?: string }>>} */
const clarified = {
  // Its LICENSE file is MIT; package.json has no "license" field.
  'svelte-toolbelt': { license: 'MIT' },
  // MIT in package.json, but no license text in the package or its repository.
  'is-reference': { license: 'MIT', text: mit('Rich Harris') },
  'locate-character': { license: 'MIT', text: mit('Rich Harris') },
};

/** @returns {License[]} */
function rustLicenses() {
  // Written to a file: cargo-about refuses to write to a redirected output under PowerShell (the
  // Windows runners' shell), because PowerShell would re-encode it.
  const dir = mkdtempSync(join(tmpdir(), 'nanoprint-about-'));
  const output = join(dir, 'about.json');
  let json;
  try {
    execFileSync(
      process.env.CARGO_ABOUT ?? 'cargo-about',
      [
        'generate',
        '--format',
        'json',
        '--locked',
        '--fail',
        '--config',
        join(root, 'about.toml'),
        '--manifest-path',
        join(root, 'src-tauri', 'Cargo.toml'),
        '--output-file',
        output,
      ],
      { stdio: ['ignore', 'ignore', 'inherit'] },
    );
    json = readFileSync(output, 'utf8');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
  // JSON.parse returns `any`; the JSDoc cast types it, and `tsc -p tsconfig.node.json` checks it.
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const about =
    /** @type {{ licenses: { name: string, text: string, used_by: { crate: { name: string, version: string } }[] }[] }} */ (
      JSON.parse(json)
    );
  return about.licenses.map((license) => ({
    name: license.name,
    text: license.text,
    users: license.used_by.map(({ crate }) => `${crate.name} ${crate.version}`),
  }));
}

/** Whether an SPDX expression is satisfied by the accepted licenses. */
function isAccepted(/** @type {string} */ expression) {
  const ids = (/** @type {string} */ alternatives) =>
    alternatives.replace(/[()]/g, '').split(/\s+AND\s+/);
  return expression
    .split(/\s+OR\s+/)
    .some((alternative) => ids(alternative).every((id) => accepted.has(id.trim())));
}

/** @returns {License[]} */
function javascriptLicenses() {
  // npm itself, run by Node without a shell (`npm run` sets npm_execpath).
  const npm = process.env.npm_execpath;
  if (!npm) throw new Error('Run this script with `npm run notices`.');
  const paths = execFileSync(process.execPath, [npm, 'ls', '--omit=dev', '--all', '--parseable'], {
    cwd: root,
    encoding: 'utf8',
  })
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(1); // The first line is NanoPrint itself.
  paths.push(...cssOnly.map((name) => join(root, 'node_modules', name)));

  /** @type {Map<string, License>} */
  const groups = new Map();
  /** @type {string[]} */
  const problems = [];
  for (const dir of new Set(paths)) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- typed by the JSDoc cast.
    const manifest = /** @type {{ name: string, version: string, license?: unknown }} */ (
      JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'))
    );
    const id = `${manifest.name} ${manifest.version}`;
    const clarification = clarified[manifest.name];
    const license =
      clarification?.license ?? (typeof manifest.license === 'string' ? manifest.license : '');
    if (!isAccepted(license)) problems.push(`${id}: license "${license}" is not accepted`);
    const texts = readdirSync(dir)
      .filter((file) => licenseFile.test(file))
      .sort()
      .map((file) => readFileSync(join(dir, file), 'utf8'));
    if (clarification?.text) texts.push(clarification.text);
    if (texts.length === 0) problems.push(`${id}: no license text`);
    for (const text of texts) {
      const group = groups.get(text) ?? { name: license, text, users: [] };
      group.users.push(id);
      groups.set(text, group);
    }
  }
  if (problems.length > 0) {
    throw new Error(`Third-party license problems:\n  ${problems.join('\n  ')}`);
  }
  return [...groups.values()];
}

const escape = (/** @type {string} */ text) =>
  text.replace(/[&<>"]/g, (c) => `&#${String(c.charCodeAt(0))};`);

function section(/** @type {string} */ title, /** @type {License[]} */ licenses) {
  const items = licenses
    .sort((a, b) => a.name.localeCompare(b.name) || a.users[0].localeCompare(b.users[0]))
    .map(
      (license) => `<section>
<h3>${escape(license.name)}</h3>
<p class="users">${escape(license.users.sort().join(', '))}</p>
<pre>${escape(license.text.trim())}</pre>
</section>`,
    );
  return `<h2>${escape(title)}</h2>\n${items.join('\n')}`;
}

/** The version being built: the single source is `[workspace.package]` in the root Cargo.toml. */
const version = /^\[workspace\.package\][^[]*?^version = "([^"]+)"/m.exec(
  readFileSync(join(root, 'Cargo.toml'), 'utf8'),
)?.[1];
if (!version) throw new Error('No [workspace.package] version in Cargo.toml.');
const repository = 'https://github.com/nanofactura/nanoprint';

const page = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Licenses</title>
<style>
:root { color-scheme: light dark; font: 14px/1.5 system-ui, sans-serif; }
body { margin: 24px; }
h1 { font-size: 20px; } h2 { font-size: 17px; margin-top: 32px; } h3 { font-size: 15px; margin-bottom: 4px; }
.users { margin: 0 0 8px; opacity: 0.75; }
pre { white-space: pre-wrap; font-size: 12px; padding: 12px; border-radius: 6px; background: rgb(127 127 127 / 12%); }
</style>
</head>
<body>
<h1>Licenses · Licencias</h1>
<p>NanoPrint ${escape(version)} is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License, version 3 (GPL-3.0-only), below. It comes with no
warranty. The complete source code of this version is at
${escape(repository)}/tree/v${escape(version)}.</p>
<p>NanoPrint includes the third-party components listed after the license, each under its own
license. The source code of each Rust crate is published at https://crates.io and of each
JavaScript package at https://www.npmjs.com, under the name and version listed.</p>
<h2>NanoPrint: GNU General Public License, version 3</h2>
<pre>${escape(readFileSync(join(root, 'LICENSE'), 'utf8').trim())}</pre>
${section('Rust crates', rustLicenses())}
${section('JavaScript packages and fonts', javascriptLicenses())}
</body>
</html>
`;

mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, page);
process.stdout.write(`Wrote ${relative(root, output)}\n`);
