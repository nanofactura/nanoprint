import { REPO_URL, SITE_URL } from '#lib/site.ts';

export const prerender = true;
export const trailingSlash = 'never';

const YEAR_MS = 365 * 24 * 60 * 60 * 1000;

/**
 * How to report a vulnerability (RFC 9116). It expires a year after each build: a deploy at least
 * once a year keeps it current.
 */
export function GET(): Response {
  const expires = new Date(Date.now() + YEAR_MS).toISOString().replace(/\.\d+Z$/, 'Z');
  const text = `Contact: ${REPO_URL}/security/advisories/new
Contact: mailto:hola@nanofactura.com
Expires: ${expires}
Policy: ${REPO_URL}/blob/main/SECURITY.md
Preferred-Languages: es, en
Canonical: ${SITE_URL}/.well-known/security.txt
`;
  return new Response(text, { headers: { 'content-type': 'text/plain; charset=utf-8' } });
}
