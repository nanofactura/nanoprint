import en from '$locales/en.json';
import es from '$locales/es.json';

export const locales = ['en', 'es'] as const;
export type Locale = (typeof locales)[number];
export type MessageKey = keyof typeof en;
type Catalog = Record<MessageKey, string>;

// Typing every catalog as a full `Catalog` turns a missing translation into a compile error.
const catalogs: Record<Locale, Catalog> = { en, es };

let current = $state<Locale>('en');

export function getLocale(): Locale {
  return current;
}

export function setLocale(locale: Locale): void {
  current = locale;
  document.documentElement.lang = locale;
}

/** Translates `key` into the current locale. Reactive: components re-render on `setLocale`. */
export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  return format(catalogs[current][key], params);
}

/** Replaces `{name}` placeholders with `params.name`; unknown placeholders are left as-is. */
export function format(template: string, params: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (placeholder, name: string) => {
    const value = params[name];
    return value === undefined ? placeholder : String(value);
  });
}
