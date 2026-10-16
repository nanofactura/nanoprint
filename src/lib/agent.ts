import { invoke } from '@tauri-apps/api/core';

/** Mirrors `Listener` in `src-tauri/src/servers.rs`. */
export type Listener =
  | { state: 'starting' }
  | { state: 'listening'; port: number }
  | { state: 'failed'; message: string };

/** Mirrors `StatusView` in `src-tauri/src/servers.rs`. */
export interface ServerStatus {
  insecure: Listener;
  secure: Listener;
  certificate: { trusted: boolean; expiresAt: number } | null;
}

/** Mirrors `SiteView` in `src-tauri/src/permissions.rs`. Times are Unix seconds. */
export interface Site {
  origin: string;
  /** `null` when the stored decision was changed outside NanoPrint and is ignored. */
  decision: 'allow' | 'block' | null;
  grantedAt: number;
  lastUsedAt: number | null;
  jobCount: number;
  /** The name the site gave itself. A claim by the site: always show the origin too. */
  name: string | null;
  /** The site's logo, re-encoded by NanoPrint as a `data:image/png` URL. */
  icon: string | null;
  /** The decision lasts until NanoPrint closes; it is not stored. */
  sessionOnly: boolean;
}

export const historyKinds = [
  'job_printed',
  'job_failed',
  'permission_granted',
  'permission_denied',
  'permission_revoked',
  'connection_rejected',
  'rate_limited',
] as const;
export type HistoryKind = (typeof historyKinds)[number];

/** Mirrors `HistoryCategory` in `src-tauri/src/store.rs`. */
export const historyCategories = ['prints', 'errors', 'permissions'] as const;
export type HistoryCategory = (typeof historyCategories)[number];

/** Mirrors `HistoryEntry` in `src-tauri/src/store.rs`. Times are Unix seconds. */
export interface HistoryEntry {
  at: number;
  kind: string;
  origin: string | null;
  siteName: string | null;
  printer: string | null;
  jobName: string | null;
  bytes: number | null;
  durationMs: number | null;
  error: string | null;
}

/** Mirrors `PrinterView` in `src-tauri/src/printer_panel.rs`. */
export interface PrinterView {
  name: string;
  driver: string | null;
  /** The port or device the printer is connected to, when the system says. */
  connection: string | null;
  default: boolean;
  /** Whether web pages may see it and print to it. */
  shared: boolean;
  /**
   * Why it cannot print raw tickets, if it cannot: a virtual printer (PDF, XPS, fax) or a Remote
   * Desktop printer with Easy Print. Not offered to sites by default, and no test page.
   */
  limitation: 'virtual' | 'easyPrint' | null;
}

/** Mirrors `Status` in `src-tauri/src/updates.rs`. Times are Unix seconds. */
export type UpdateStatus =
  | { state: 'unsupported'; reason: 'development' | 'package' }
  | { state: 'idle' }
  | { state: 'checking' }
  | { state: 'upToDate'; checkedAt: number }
  | { state: 'downloading'; version: string }
  | { state: 'ready'; version: string }
  | { state: 'failed'; message: string };

/** Mirrors `UpdateView` in `src-tauri/src/updates.rs`. */
export interface UpdateView {
  status: UpdateStatus;
  /** Checks and installs on its own; otherwise only when the user asks. */
  automatic: boolean;
}

/** Mirrors `PermissionRequest` in `src-tauri/src/permissions.rs`. */
export interface PermissionRequest {
  origin: string;
  rememberable: boolean;
  name: string | null;
  icon: string | null;
}

/** Mirrors `Reason` in `src-tauri/src/onboarding.rs`. */
export type OnboardingReason = 'welcome' | 'certificateRefused';

/** Event emitted to the permission window when the request it should show changes. */
export const PERMISSION_REQUEST_CHANGED = 'permission-request-changed';

export function isHistoryKind(kind: string): kind is HistoryKind {
  return (historyKinds as readonly string[]).includes(kind);
}

export function isHistoryCategory(value: string): value is HistoryCategory {
  return (historyCategories as readonly string[]).includes(value);
}

export function getServerStatus(): Promise<ServerStatus> {
  return invoke<ServerStatus>('get_server_status');
}

/** Adds the local certificate authority to the system's trust store; the system asks to confirm. */
export function installCertificate(): Promise<void> {
  return invoke('install_certificate');
}

export function removeCertificate(): Promise<void> {
  return invoke('remove_certificate');
}

export function listSites(): Promise<Site[]> {
  return invoke<Site[]>('list_sites');
}

export function revokeSite(origin: string): Promise<void> {
  return invoke('revoke_site', { origin });
}

/** The last 500 history entries, newest first, whose site origin or name contains `search`. */
export function listHistory(
  search: string | null,
  category: HistoryCategory | null,
): Promise<HistoryEntry[]> {
  return invoke<HistoryEntry[]>('list_history', { search, category });
}

/** The system's printers. `refresh` asks the system again instead of reusing the recent list. */
export function listPrinters(refresh: boolean): Promise<PrinterView[]> {
  return invoke<PrinterView[]>('list_printers', { refresh });
}

/** Offers a printer to web pages, or hides it from them. */
export function setPrinterShared(name: string, shared: boolean): Promise<void> {
  return invoke('set_printer_shared', { name, shared });
}

/** Prints NanoPrint's ESC/POS test page, recorded in the history. */
export function printTestPage(name: string): Promise<void> {
  return invoke('print_test_page', { name });
}

export function getPermissionRequest(): Promise<PermissionRequest | null> {
  return invoke<PermissionRequest | null>('get_permission_request');
}

export function answerPermissionRequest(
  origin: string,
  allow: boolean,
  remember: boolean,
): Promise<void> {
  return invoke('answer_permission_request', { origin, allow, remember });
}

/** Opens the third-party licenses window. */
export function showLicenses(): Promise<void> {
  return invoke('show_licenses');
}

/** Shows the log folder in the system's file manager. */
export function openLogFolder(): Promise<void> {
  return invoke('open_log_folder');
}

export function getUpdateStatus(): Promise<UpdateView> {
  return invoke<UpdateView>('get_update_status');
}

/** Checks GitHub for a newer version now and downloads it; resolves when done. */
export function checkForUpdates(): Promise<UpdateView> {
  return invoke<UpdateView>('check_for_updates');
}

/** Installs the downloaded update; NanoPrint closes and starts again. */
export function installUpdate(): Promise<void> {
  return invoke('install_update');
}

export function setAutomaticUpdates(automatic: boolean): Promise<UpdateView> {
  return invoke<UpdateView>('set_automatic_updates', { automatic });
}

export function getOnboardingReason(): Promise<OnboardingReason> {
  return invoke<OnboardingReason>('get_onboarding_reason');
}
