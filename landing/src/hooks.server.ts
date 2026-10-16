import type { Handle } from '@sveltejs/kit';
import { languageOf } from '$lib/i18n';

/** Runs while prerendering: each page's <html lang> follows its URL (`/en/…` is English). */
export const handle: Handle = ({ event, resolve }) =>
  resolve(event, {
    transformPageChunk: ({ html }) => html.replace('%lang%', languageOf(event.url.pathname)),
  });
