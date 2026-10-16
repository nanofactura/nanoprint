# NanoPrint website

The site at [nanoprint.nanofactura.com](https://nanoprint.nanofactura.com): a home page and a page
for developers, in Spanish (`/`, `/desarrolladores/`) and English (`/en/`, `/en/developers/`).
Built with SvelteKit, Svelte 5, Tailwind CSS 4 and Vite, rendered to static HTML at build time, and
served by Cloudflare (Pages, now part of Workers as static assets). It is a separate npm package
from the app: its dependencies, including Wrangler, never reach the app's build.

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

Deploys run from a maintainer's computer, not from CI. Cloudflare Pages now runs as part of
Workers ("static assets"): `wrangler.toml` names the Worker (`nanoprint`), the folder it serves,
how it answers unknown paths (`404.html`, status 404) and trailing slashes, and its one custom
domain. The first deploy created the Worker, the `nanoprint.nanofactura.com` DNS record and its
certificate; nothing else in the `nanofactura.com` zone is touched.

Once per computer: `npx wrangler login` opens the browser to authorize Wrangler. Its credentials stay
in your user profile, outside the repository. If your login reaches several Cloudflare accounts,
copy `.env.example` to `.env` and set `CLOUDFLARE_ACCOUNT_ID` (not secret, but it does not belong in
the repository either).

Each time:

```sh
npm run deploy   # builds and publishes to nanoprint.nanofactura.com
```

Check the result with `npm run preview` before deploying. Never commit `.env` or an API token; git
ignores `.env`.

## Hero video

The home page's hero plays a 16:9 animation once, muted and without controls, and stops on its last
frame; clicking it (or pressing Enter on it) plays it again. Visitors who ask their system for
reduced motion see the last frame and can still play it.

The files live in the repository, in `static/media/`: `hero.webm` (VP9, for most browsers),
`hero.mp4` (H.264, for the rest) and `hero-poster.webp` (the first frame, shown until the video
starts). None has an audio track. The source video has no transparency, so instead its dark
background is graded to the page's color (`#0b0d14`) and the edges fade out with a CSS mask.
Keep each file small (Cloudflare Pages accepts up to 25 MiB per file). From a new source video:

```sh
GRADE="curves=all='0/0 0.12/0.055 0.5/0.47 1/1',colorbalance=rs=-0.02:bs=0.035"
ffmpeg -i source.mp4 -an -vf "$GRADE" -c:v libvpx-vp9 -crf 34 -b:v 0 -row-mt 1 static/media/hero.webm
ffmpeg -i source.mp4 -an -vf "$GRADE" -c:v libx264 -crf 24 -preset slow -pix_fmt yuv420p -movflags +faststart static/media/hero.mp4
ffmpeg -i source.mp4 -frames:v 1 -vf "$GRADE" -c:v libwebp -quality 82 static/media/hero-poster.webp
```

The grade suits a background near `#1c1c1c`; adjust the curve for another one.

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
