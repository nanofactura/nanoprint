import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { thirdPartyLicenses } from './scripts/licenses.ts';

/** NanoPrint listens on these ports; qz-tray.js tries them in order (see ../docs/protocol.md). */
const nanoprint = ['wss://localhost:*', 'ws://localhost:*'] as const;

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit({
      preprocess: vitePreprocess(),
      // Every page is rendered to HTML at build time: Cloudflare Pages serves files only.
      adapter: adapter({ strict: true }),
      // Each prerendered page carries a CSP with the hashes of its own inline scripts. Headers that
      // a <meta> cannot set (frame-ancestors) are in static/_headers.
      csp: {
        mode: 'hash',
        directives: {
          'default-src': ['self'],
          'script-src': ['self'],
          // SvelteKit's route announcer (for screen readers) is created with a style attribute.
          // Scripts stay limited to the site's own files and hashed inline scripts.
          'style-src': ['self', 'unsafe-inline'],
          'img-src': ['self'],
          'font-src': ['self'],
          'media-src': ['self'],
          // The developer page's demo talks to NanoPrint on this computer.
          'connect-src': ['self', ...nanoprint],
          'object-src': ['none'],
          'base-uri': ['self'],
          'form-action': ['none'],
        },
      },
    }),
    thirdPartyLicenses(),
  ],
  // Every TypeScript file, dependencies included, uses this project's settings. Otherwise files the
  // project excludes (node_modules) look further up and find the app's tsconfig in the repository
  // root, which needs the app's dependencies.
  tsconfig: 'tsconfig.json',
});
