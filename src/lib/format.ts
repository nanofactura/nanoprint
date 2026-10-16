import { getLocale } from '$lib/i18n/index.svelte';

/** A Unix timestamp (seconds) as a localized date and time. */
export function formatDateTime(seconds: number): string {
  return new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(
    seconds * 1000,
  );
}

/** A Unix timestamp (seconds) as a localized date. */
export function formatDate(seconds: number): string {
  return new Intl.DateTimeFormat(getLocale(), { dateStyle: 'long' }).format(seconds * 1000);
}

export function formatNumber(value: number): string {
  return new Intl.NumberFormat(getLocale()).format(value);
}
