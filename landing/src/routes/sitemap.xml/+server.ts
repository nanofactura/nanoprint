import { FALLBACK_LANG, languages, pages } from '$lib/i18n';
import { SITE_URL } from '$lib/site';

export const prerender = true;
export const trailingSlash = 'never';

/** Every page, with its other language and the version for everyone else (x-default). */
export function GET(): Response {
  const urls = Object.values(pages).flatMap((page) =>
    languages.map((lang) => {
      const alternates = [
        ...languages.map((other) => [other, page[other]] as const),
        ['x-default', page[FALLBACK_LANG]] as const,
      ]
        .map(
          ([hreflang, path]) =>
            `    <xhtml:link rel="alternate" hreflang="${hreflang}" href="${SITE_URL}${path}"/>`,
        )
        .join('\n');
      return `  <url>\n    <loc>${SITE_URL}${page[lang]}</loc>\n${alternates}\n  </url>`;
    }),
  );
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${urls.join('\n')}
</urlset>
`;
  return new Response(xml, { headers: { 'content-type': 'application/xml' } });
}
