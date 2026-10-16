import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { thirdPartyLicenses } from './scripts/licenses.ts';

export default defineConfig({
  plugins: [tailwindcss(), sveltekit(), thirdPartyLicenses()],
  // Every TypeScript file, dependencies included, uses this project's settings. Otherwise files the
  // project excludes (node_modules) look further up and find the app's tsconfig in the repository
  // root, which needs the app's dependencies.
  tsconfig: 'tsconfig.json',
});
