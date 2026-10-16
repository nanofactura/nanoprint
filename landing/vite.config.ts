import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { thirdPartyLicenses } from './scripts/licenses.ts';

export default defineConfig({
  plugins: [tailwindcss(), sveltekit(), thirdPartyLicenses()],
});
