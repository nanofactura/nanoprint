# NanoPrint website

The site at [nanoprint.nanofactura.com](https://nanoprint.nanofactura.com): a home page and a page
for developers, in Spanish (`/`, `/desarrolladores/`) and English (`/en/`, `/en/developers/`).
Built with SvelteKit, Svelte 5, Tailwind CSS 4 and Vite, rendered to static HTML at build time, and
served by Cloudflare Pages. It is a separate npm package from the app: its dependencies, including
Wrangler, never reach the app's build.

## Working on it

```sh
cd landing
npm ci
npm run dev       # http://localhost:5173
npm run check     # types and Svelte warnings
npm run build     # static site in build/
npm run preview   # serves build/ at http://localhost:4173
```

All text lives in `src/lib/i18n/es.ts` and `src/lib/i18n/en.ts`; TypeScript fails the build when one
language misses an entry. Text between backticks renders as inline code.

## Deploying

Deploys run from a maintainer's computer, not from CI.

Once:

1. `npx wrangler login` opens the browser to authorize Wrangler. Its credentials stay in your user
   profile, outside the repository. If your login reaches several Cloudflare accounts, copy
   `.env.example` to `.env` and set `CLOUDFLARE_ACCOUNT_ID` (the ID is not secret, but it does not
   belong in the repository either).
2. `npx wrangler pages project create nanoprint --production-branch main`
3. In the Cloudflare dashboard, open the `nanoprint` Pages project → Custom domains → add
   `nanoprint.nanofactura.com`. With nanofactura.com on Cloudflare, the DNS record is created for
   you.

Each time:

```sh
npm run deploy           # builds and publishes to production
npm run deploy:preview   # builds and publishes to a preview URL
```

`wrangler.toml` holds only the project name and the output folder. Never commit `.env` or an API
token; git ignores `.env`.

## Hero video

The home page's hero plays `static/media/hero.webm` and `static/media/hero.mp4` in a loop, muted, in
a 5:4 frame (`object-cover` crops other shapes). Until the files exist, a drawing of a web page
printing a receipt shows instead, and the build only warns. Keep each file under a few megabytes
(Cloudflare Pages accepts up to 25 MiB per file). From a source video:

```sh
ffmpeg -i source.mov -an -vf "scale=1200:-2" -c:v libvpx-vp9 -crf 34 -b:v 0 static/media/hero.webm
ffmpeg -i source.mov -an -vf "scale=1200:-2" -c:v libx264 -crf 24 -preset slow -pix_fmt yuv420p -movflags +faststart static/media/hero.mp4
```

Visitors who ask their system for reduced motion see the first frame only.

## Search engines and AI assistants

- Every page is complete HTML, readable without JavaScript.
- Each page links its other language with `hreflang` and names itself as canonical; English is the
  `x-default` for other languages. `sitemap.xml` repeats these pairs. Both come from `pages` in
  `src/lib/i18n/index.ts`: add a page there and in `src/routes/`.
- Structured data (schema.org): `SoftwareApplication` and `FAQPage` on the home page, `TechArticle`
  and `BreadcrumbList` on the developer page.
- `llms.txt` summarizes the product for AI assistants, built from the same text as the pages.
- Social previews: `static/og-es.png` and `static/og-en.png`.

## Icons and preview images

`npm run assets` renders the favicons, the touch icon, the demo's logo and the social preview images
from the brand masters in `../assets/brand`. Run it after changing a master or a tagline in
`scripts/generate-assets.mjs`; the results are committed.

## Security and licenses

- Each page carries a Content-Security-Policy with the hashes of its inline scripts
  (`svelte.config.js`); `static/_headers` adds the headers a `<meta>` tag cannot set. The only
  outside connections allowed are to NanoPrint on `localhost`, for the developer demo.
- No cookies, analytics or third-party requests: fonts and scripts are served by the site itself.
- The build writes `third-party-licenses.txt` (`scripts/licenses.ts`) with the license text of every
  package whose code reaches the browser. qz-tray (LGPL-2.1) is served unmodified in its own file;
  its license text is in `licenses/`.
