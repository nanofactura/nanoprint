import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import process from 'node:process';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import type { Plugin } from 'vite';
import { defineConfig } from 'vitest/config';

const host = process.env.TAURI_DEV_HOST;

/**
 * Serves `third-party-notices.html`, the licenses window: the page `npm run notices` generates
 * (release builds run it first), or a short placeholder in development.
 */
function thirdPartyNotices(): Plugin {
  const fileName = 'third-party-notices.html';
  const generated = fileURLToPath(new URL(`./generated/${fileName}`, import.meta.url));
  const page = () =>
    existsSync(generated)
      ? readFileSync(generated, 'utf8')
      : '<!doctype html><meta charset="utf-8"><title>Third-party licenses</title>' +
        '<p>The list of third-party licenses is generated for release builds (npm run notices).</p>';
  return {
    name: 'nanoprint:third-party-notices',
    configureServer(server) {
      server.middlewares.use(`/${fileName}`, (_request, response) => {
        response.setHeader('content-type', 'text/html; charset=utf-8');
        response.end(page());
      });
    },
    generateBundle() {
      this.emitFile({ type: 'asset', fileName, source: page() });
    },
  };
}

// Options tailored for `tauri dev` / `tauri build`. See https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [tailwindcss(), svelte(), thirdPartyNotices()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
      // UI strings shared with the Rust side (tray menu, notifications).
      $locales: fileURLToPath(new URL('./locales', import.meta.url)),
    },
  },
  // Keep Rust compiler errors visible in the terminal.
  clearScreen: false,
  server: {
    // Tauri expects a fixed port; fail instead of picking another one.
    port: 1420,
    strictPort: true,
    host: host ?? '127.0.0.1',
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    // The website (landing/) is a separate project: its builds must not reload the app.
    watch: { ignored: ['**/src-tauri/**', '**/target/**', '**/landing/**'] },
  },
  build: {
    // Tauri's webviews are evergreen (WebView2, WKWebView, WebKitGTK 4.1).
    target: 'es2022',
    // One page per window.
    rolldownOptions: {
      input: {
        settings: fileURLToPath(new URL('./index.html', import.meta.url)),
        permission: fileURLToPath(new URL('./permission.html', import.meta.url)),
        onboarding: fileURLToPath(new URL('./onboarding.html', import.meta.url)),
      },
    },
  },
  test: {
    include: ['src/**/*.test.ts'],
  },
});
