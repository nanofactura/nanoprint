import { invoke } from '@tauri-apps/api/core';
import { locales } from '$lib/i18n/index.svelte';

export const languagePreferences = ['system', ...locales] as const;
export type LanguagePreference = (typeof languagePreferences)[number];

export function isLanguagePreference(value: string): value is LanguagePreference {
  return (languagePreferences as readonly string[]).includes(value);
}

/** Mirrors `SettingsView` in `src-tauri/src/settings.rs`. */
export interface SettingsView {
  language: LanguagePreference;
  /** Locale the UI uses: the explicit preference, or the system language. */
  resolvedLocale: (typeof locales)[number];
  /** Detected system language, shown next to the "System" option. */
  systemLocale: (typeof locales)[number];
  /** Fixed ports, or `null` for the first free one of those qz-tray.js looks for. */
  insecurePort: number | null;
  securePort: number | null;
}

export function getSettings(): Promise<SettingsView> {
  return invoke<SettingsView>('get_settings');
}

export function setLanguage(language: LanguagePreference): Promise<SettingsView> {
  return invoke<SettingsView>('set_language', { language });
}

/** Takes effect when NanoPrint restarts (see {@link restart}). */
export function setPorts(
  insecurePort: number | null,
  securePort: number | null,
): Promise<SettingsView> {
  return invoke<SettingsView>('set_ports', { insecurePort, securePort });
}

/** Closes NanoPrint and starts it again. */
export function restart(): Promise<void> {
  return invoke('restart');
}
