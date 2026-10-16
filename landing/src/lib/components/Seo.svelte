<!--
  Head tags of a page: title and description, the canonical URL, the same page in the other
  language (hreflang, with English for every other language), social previews and structured data.
-->
<script lang="ts">
  import { FALLBACK_LANG, languages, messages, pages, type Lang, type Page } from '$lib/i18n';
  import { SITE_URL } from '$lib/site';

  let {
    lang,
    page,
    title,
    description,
    structuredData = [],
  }: {
    lang: Lang;
    page: Page;
    title: string;
    description: string;
    structuredData?: object[];
  } = $props();

  const url = (language: Lang) => `${SITE_URL}${pages[page][language]}`;
  const other = $derived<Lang>(lang === 'es' ? 'en' : 'es');
  // `<` is escaped so no text can close the script element early.
  const blocks = $derived(
    structuredData.map(
      (data) =>
        `<script type="application/ld+json">${JSON.stringify(data).replaceAll('<', '\\u003c')}</` +
        'script>',
    ),
  );
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={url(lang)} />
  {#each languages as language (language)}
    <link rel="alternate" hreflang={language} href={url(language)} />
  {/each}
  <link rel="alternate" hreflang="x-default" href={url(FALLBACK_LANG)} />

  <meta property="og:type" content="website" />
  <meta property="og:site_name" content="NanoPrint" />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:url" content={url(lang)} />
  <meta property="og:image" content={`${SITE_URL}/og-${lang}.png`} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="630" />
  <meta property="og:image:alt" content={title} />
  <meta property="og:locale" content={messages(lang).meta.locale} />
  <meta property="og:locale:alternate" content={messages(other).meta.locale} />
  <meta name="twitter:card" content="summary_large_image" />

  <!-- eslint-disable-next-line svelte/no-at-html-tags -- JSON built above, with `<` escaped. -->
  {@html blocks.join('')}
</svelte:head>
