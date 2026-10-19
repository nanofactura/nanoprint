import { t } from '$lib/i18n/index.svelte';

/** Mirrors `UiError` in `src-tauri/src/ui_error.rs`. */
interface UiError {
  code: string;
  detail: string | null;
}

const codes = [
  'certificate_not_ready',
  'trust_tool_missing',
  'trust_failed',
  'storage',
  'settings',
  'log_folder',
  'printers_unavailable',
  'printer_not_found',
  'print_failed',
  'virtual_printer',
  'easy_print_printer',
  'update_failed',
  'update_not_ready',
  'port_invalid',
  'ports_equal',
  'network_name_invalid',
  'network_host_invalid',
  'network_port_invalid',
  'printer_name_taken',
  'unexpected',
] as const;
type Code = (typeof codes)[number];

function isUiError(error: unknown): error is UiError {
  return typeof error === 'object' && error !== null && 'code' in error;
}

function isCode(code: string): code is Code {
  return (codes as readonly string[]).includes(code);
}

/**
 * A command's error in the user's language: NanoPrint's own text comes from the locales, and the
 * system's message (already localized by the system, or technical) follows in parentheses.
 */
export function describeError(error: unknown): string {
  if (!isUiError(error)) return String(error);
  const text = isCode(error.code) ? t(`error.code.${error.code}`) : t('error.code.unexpected');
  return error.detail ? `${text} (${error.detail})` : text;
}
