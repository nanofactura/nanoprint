/** Where the site is published. Canonical URLs, the sitemap and social previews use it. */
export const SITE_URL = 'https://nanoprint.nanofactura.com';

export const REPO_URL = 'https://github.com/nanofactura/nanoprint';
export const RELEASES_URL = `${REPO_URL}/releases`;
export const NANOFACTURA_URL = 'https://nanofactura.com';
export const ALTERNATIVETO_URL =
  'https://alternativeto.net/software/nanoprint/about/?utm_source=badge&utm_medium=referral';

export const docs = {
  integration: `${REPO_URL}/blob/main/docs/integration.md`,
  protocol: `${REPO_URL}/blob/main/docs/protocol.md`,
  license: `${REPO_URL}/blob/main/LICENSE`,
  security: `${REPO_URL}/blob/main/SECURITY.md`,
} as const;

/**
 * Installers of the newest published release. Their names carry no version (see
 * ../docs/releasing.md), so these links never change. Drafts and pre-releases are not "latest".
 */
const LATEST = `${REPO_URL}/releases/latest/download/`;
export const downloads = {
  windows: `${LATEST}NanoPrint_x64-setup.exe`,
  mac: `${LATEST}NanoPrint_universal.dmg`,
  deb: `${LATEST}NanoPrint_amd64.deb`,
  rpm: `${LATEST}NanoPrint_x86_64.rpm`,
  appImage: `${LATEST}NanoPrint_amd64.AppImage`,
} as const;

export type Platform = 'windows' | 'mac' | 'linux';

/** The visitor's desktop system, or `null` on phones, tablets and unknown systems. */
export function detectPlatform(): Platform | null {
  const nav = navigator as Navigator & { userAgentData?: { platform?: string; mobile?: boolean } };
  const ua = nav.userAgent;
  if (nav.userAgentData?.mobile || /Android|iPhone|iPad|iPod/i.test(ua)) return null;
  const platform = nav.userAgentData?.platform ?? ua;
  if (/Win/i.test(platform)) return 'windows';
  // iPads report a Mac platform; touch tells them apart.
  if (/Mac/i.test(platform)) return navigator.maxTouchPoints > 1 ? null : 'mac';
  if (/CrOS/i.test(ua)) return null;
  if (/Linux|X11/i.test(platform)) return 'linux';
  return null;
}
