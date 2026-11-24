import { en } from './en';
import { es } from './es';
import { pt } from './pt';
import type { Messages } from './types';

export type Lang = 'es' | 'en' | 'pt';
export const languages: readonly Lang[] = ['es', 'en', 'pt'];

/**
 * Every page in every language. Spanish lives at the root, English under /en/ and Brazilian
 * Portuguese under /pt/. Portuguese is tagged `pt`, not `pt-BR`, so it also serves Portugal, as
 * the app does. Search engines learn the sets from the hreflang links each page carries (see
 * Seo.svelte).
 */
export const pages = {
  home: { es: '/', en: '/en/', pt: '/pt/' },
  developers: { es: '/desarrolladores/', en: '/en/developers/', pt: '/pt/desenvolvedores/' },
} as const satisfies Record<string, Record<Lang, string>>;
export type Page = keyof typeof pages;

const catalogs: Record<Lang, Messages> = { es, en, pt };

/** The version shown to visitors whose language is none of these: English. */
export const FALLBACK_LANG: Lang = 'en';

/** The first path segment names the language (`/en/…`, `/pt/…`); Spanish pages have none. */
export function languageOf(pathname: string): Lang {
  const segment = pathname.split('/')[1];
  return segment === 'en' || segment === 'pt' ? segment : 'es';
}

export function messages(lang: Lang): Messages {
  return catalogs[lang];
}

/** Replaces `{name}` placeholders with `params.name`. */
export function format(template: string, params: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (placeholder, name: string) => {
    const value = params[name];
    return value === undefined ? placeholder : String(value);
  });
}
