import { en } from './en';
import { es } from './es';
import type { Messages } from './types';

export type Lang = 'es' | 'en';
export const languages: readonly Lang[] = ['es', 'en'];

/**
 * Every page in both languages. Spanish lives at the root, English under /en/. Search engines
 * learn the pairs from the hreflang links each page carries (see Seo.svelte).
 */
export const pages = {
  home: { es: '/', en: '/en/' },
  developers: { es: '/desarrolladores/', en: '/en/developers/' },
} as const satisfies Record<string, Record<Lang, string>>;
export type Page = keyof typeof pages;

/** The version shown to visitors whose language is neither: English. */
export const FALLBACK_LANG: Lang = 'en';

export function languageOf(pathname: string): Lang {
  return pathname === '/en' || pathname.startsWith('/en/') ? 'en' : 'es';
}

export function messages(lang: Lang): Messages {
  return lang === 'en' ? en : es;
}

/** Replaces `{name}` placeholders with `params.name`. */
export function format(template: string, params: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (placeholder, name: string) => {
    const value = params[name];
    return value === undefined ? placeholder : String(value);
  });
}
