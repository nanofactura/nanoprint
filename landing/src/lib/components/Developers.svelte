<script lang="ts">
  import ArrowUpRight from '@lucide/svelte/icons/arrow-up-right';
  import Check from '@lucide/svelte/icons/check';
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import Footer from '$lib/components/Footer.svelte';
  import Header from '$lib/components/Header.svelte';
  import PrintDemo from '$lib/components/PrintDemo.svelte';
  import Rich from '$lib/components/Rich.svelte';
  import Seo from '$lib/components/Seo.svelte';
  import { messages, pages, type Lang } from '$lib/i18n';
  import { docs, NANOFACTURA_URL, REPO_URL, SITE_URL } from '$lib/site';

  let { lang }: { lang: Lang } = $props();

  const t = $derived(messages(lang));
  const dev = $derived(t.developers);

  const structuredData = $derived([
    {
      '@context': 'https://schema.org',
      '@type': 'TechArticle',
      headline: dev.hero.title,
      description: dev.description,
      url: `${SITE_URL}${pages.developers[lang]}`,
      inLanguage: lang,
      about: { '@type': 'SoftwareApplication', name: 'NanoPrint', url: SITE_URL },
      publisher: { '@type': 'Organization', name: 'NanoFactura', url: NANOFACTURA_URL },
    },
    {
      '@context': 'https://schema.org',
      '@type': 'BreadcrumbList',
      itemListElement: [
        {
          '@type': 'ListItem',
          position: 1,
          name: 'NanoPrint',
          item: `${SITE_URL}${pages.home[lang]}`,
        },
        {
          '@type': 'ListItem',
          position: 2,
          name: t.nav.developers,
          item: `${SITE_URL}${pages.developers[lang]}`,
        },
      ],
    },
  ]);
</script>

<Seo {lang} page="developers" title={dev.title} description={dev.description} {structuredData} />
<Header {lang} page="developers" />

<main id="main">
  <section class="relative overflow-hidden">
    <div
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_85%_0%,color-mix(in_oklab,var(--color-brand)_20%,transparent),transparent_55%)]"
    ></div>
    <div class="relative container-page flex max-w-4xl flex-col gap-6 py-16 md:py-20">
      <p
        class="w-fit rounded-full border border-accent/30 bg-accent/10 px-3 py-1 text-sm text-accent-soft"
      >
        {dev.hero.eyebrow}
      </p>
      <h1 class="text-4xl font-semibold tracking-tight text-balance sm:text-5xl">
        {dev.hero.title}
      </h1>
      <p class="text-lg leading-relaxed text-muted">{dev.hero.lead}</p>
      <p class="flex flex-wrap gap-x-6 gap-y-2">
        <a href={docs.integration} class="inline-flex items-center gap-1 link"
          >{dev.hero.guide}<ArrowUpRight class="size-4" aria-hidden="true" /></a
        >
        <a href={docs.protocol} class="inline-flex items-center gap-1 link"
          >{dev.hero.protocol}<ArrowUpRight class="size-4" aria-hidden="true" /></a
        >
      </p>
    </div>
  </section>

  <section class="container-page max-w-4xl py-12">
    <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{dev.libraries.title}</h2>
    <p class="mt-3 text-muted">{dev.libraries.lead}</p>
    <div class="mt-8 flex flex-col gap-4">
      {#each dev.libraries.items as library (library.name)}
        <article class="rounded-xl border border-line bg-surface p-5">
          <div class="flex flex-wrap items-baseline justify-between gap-2">
            <h3 class="font-mono text-base font-medium">
              <a href={library.href} class="hover:text-brand-soft hover:underline">{library.name}</a
              >
            </h3>
            <span class="text-xs text-muted">{dev.libraries.license}: {library.license}</span>
          </div>
          <p class="mt-2">{library.role}</p>
          <p class="mt-1 text-sm text-muted">{library.note}</p>
        </article>
      {/each}
    </div>
  </section>

  <section class="container-page max-w-4xl py-12">
    <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{dev.example.title}</h2>
    <p class="mt-3 text-muted"><Rich text={dev.example.lead} /></p>
    <div class="mt-6"><CodeBlock code={dev.example.code} {lang} /></div>
    <p class="mt-4 text-sm text-muted"><Rich text={dev.example.note} /></p>
  </section>

  <section class="container-page max-w-4xl py-12">
    <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{dev.identity.title}</h2>
    <p class="mt-3 text-muted">{dev.identity.text}</p>
    <div class="mt-6"><CodeBlock code={dev.identity.code} {lang} /></div>
  </section>

  <section id="demo" class="container-page max-w-4xl py-12">
    <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{dev.demo.title}</h2>
    <p class="mt-3 mb-6 text-muted">{dev.demo.lead}</p>
    <PrintDemo {lang} />
  </section>

  <section id="adopt" class="container-page max-w-4xl py-12">
    <div
      class="rounded-2xl border border-brand/30 bg-[linear-gradient(135deg,color-mix(in_oklab,var(--color-brand)_14%,transparent),transparent_60%)] p-6 sm:p-8"
    >
      <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{dev.adopt.title}</h2>
      <p class="mt-3 text-muted">{dev.adopt.lead}</p>
      <ul class="mt-6 flex flex-col gap-3">
        {#each dev.adopt.items as item (item)}
          <li class="flex gap-3">
            <Check class="mt-1 size-4 shrink-0 text-accent" aria-hidden="true" />
            <span><Rich text={item} /></span>
          </li>
        {/each}
      </ul>
      <a href={`${REPO_URL}/issues`} class="mt-6 inline-flex items-center gap-1 link"
        >{dev.adopt.issues}<ArrowUpRight class="size-4" aria-hidden="true" /></a
      >
    </div>
  </section>

  <section class="container-page max-w-4xl py-12">
    <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{dev.requirements.title}</h2>
    <ul class="mt-6 flex flex-col gap-3">
      {#each dev.requirements.items as item (item)}
        <li class="flex gap-3 text-muted">
          <Check class="mt-1 size-4 shrink-0 text-accent" aria-hidden="true" />
          <span><Rich text={item} /></span>
        </li>
      {/each}
    </ul>
    <a href={docs.integration} class="mt-8 inline-flex items-center gap-1 link"
      >{dev.more}<ArrowUpRight class="size-4" aria-hidden="true" /></a
    >
  </section>
</main>

<Footer {lang} />
