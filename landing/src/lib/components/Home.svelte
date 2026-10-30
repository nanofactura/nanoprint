<script lang="ts">
  import ArrowRight from '@lucide/svelte/icons/arrow-right';
  import Check from '@lucide/svelte/icons/check';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import Download from '@lucide/svelte/icons/download';
  import Feather from '@lucide/svelte/icons/feather';
  import Monitor from '@lucide/svelte/icons/monitor';
  import ShieldCheck from '@lucide/svelte/icons/shield-check';
  import CodeBlock from '#lib/components/CodeBlock.svelte';
  import DownloadButton from '#lib/components/DownloadButton.svelte';
  import Footer from '#lib/components/Footer.svelte';
  import Header from '#lib/components/Header.svelte';
  import HeroMedia from '#lib/components/HeroMedia.svelte';
  import Rich from '#lib/components/Rich.svelte';
  import Seo from '#lib/components/Seo.svelte';
  import { messages, pages, type Lang } from '#lib/i18n/index.ts';
  import {
    ALTERNATIVETO_URL,
    downloads,
    NANOFACTURA_URL,
    RELEASES_URL,
    REPO_URL,
    SITE_URL,
  } from '#lib/site.ts';

  let { lang }: { lang: Lang } = $props();

  const t = $derived(messages(lang));
  const home = $derived(t.home);
  const featureIcons = [Monitor, Feather, ShieldCheck];
  const platforms = $derived([
    { id: 'download-windows', ...home.downloads.windows },
    { id: 'download-mac', ...home.downloads.mac },
    { id: 'download-linux', ...home.downloads.linux },
  ]);

  // What search engines and AI assistants read about the page (schema.org).
  const structuredData = $derived([
    {
      '@context': 'https://schema.org',
      '@type': 'SoftwareApplication',
      name: 'NanoPrint',
      description: home.description,
      url: `${SITE_URL}${pages.home[lang]}`,
      image: `${SITE_URL}/og-${lang}.png`,
      applicationCategory: 'BusinessApplication',
      operatingSystem: 'Windows 10, Windows 11, macOS 10.15+, Linux',
      downloadUrl: RELEASES_URL,
      license: 'https://www.gnu.org/licenses/gpl-3.0.html',
      isAccessibleForFree: true,
      offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
      inLanguage: ['es', 'en'],
      sameAs: [REPO_URL],
      publisher: { '@type': 'Organization', name: 'NanoFactura', url: NANOFACTURA_URL },
    },
    {
      '@context': 'https://schema.org',
      '@type': 'FAQPage',
      mainEntity: home.faq.items.map((item) => ({
        '@type': 'Question',
        name: item.q,
        acceptedAnswer: { '@type': 'Answer', text: item.a },
      })),
    },
  ]);
</script>

<Seo {lang} page="home" title={home.title} description={home.description} {structuredData} />
<Header {lang} page="home" />

<main id="main">
  <!-- Hero -->
  <section class="relative overflow-hidden">
    <div
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_80%_0%,color-mix(in_oklab,var(--color-brand)_22%,transparent),transparent_55%),radial-gradient(ellipse_at_0%_100%,color-mix(in_oklab,var(--color-accent)_12%,transparent),transparent_50%)]"
    ></div>
    <div
      class="relative container-page grid items-center gap-12 py-16 md:py-24 lg:grid-cols-[1fr_1.2fr] lg:items-start"
    >
      <div class="flex flex-col gap-7">
        <h1
          class="text-4xl font-semibold tracking-tight text-balance sm:text-5xl lg:text-[3.4rem] lg:leading-[1.08]"
        >
          {home.hero.title}
        </h1>
        <p class="max-w-xl text-lg leading-relaxed text-muted">{home.hero.lead}</p>
        <DownloadButton {lang} />
        <ul class="flex flex-wrap gap-2 text-xs text-muted">
          <li class="rounded-md border border-line px-2.5 py-1">{home.hero.platforms}</li>
          <li class="rounded-md border border-line px-2.5 py-1">{home.hero.free}</li>
          <li class="rounded-md border border-line px-2.5 py-1">{home.hero.openSource}</li>
        </ul>
      </div>
      <div class="lg:-mr-8 lg:pt-10 xl:-mr-16"><HeroMedia {lang} /></div>
    </div>
  </section>

  <!-- Features -->
  <section id="features" class="container-page py-20">
    <div class="max-w-2xl">
      <h2 class="text-3xl font-semibold tracking-tight sm:text-4xl">{home.features.title}</h2>
      <p class="mt-4 text-lg text-muted">{home.features.lead}</p>
    </div>
    <div class="mt-12 grid gap-6 md:grid-cols-3">
      {#each home.features.items as feature, index (feature.title)}
        {@const Icon = featureIcons[index]}
        <article class="flex flex-col gap-4 rounded-2xl border border-line bg-surface p-6">
          <span
            class="flex size-11 items-center justify-center rounded-xl bg-brand/15 text-brand-soft"
            aria-hidden="true"
          >
            <Icon class="size-5" />
          </span>
          <h3 class="text-xl font-semibold">{feature.title}</h3>
          <p class="text-muted">{feature.text}</p>
          <ul class="mt-auto flex flex-col gap-2 border-t border-line pt-4 text-sm">
            {#each feature.points as point (point)}
              <li class="flex gap-2">
                <Check class="mt-0.5 size-4 shrink-0 text-accent" aria-hidden="true" />{point}
              </li>
            {/each}
          </ul>
        </article>
      {/each}
    </div>
  </section>

  <!-- How it works -->
  <section class="border-y border-line bg-surface/40">
    <div class="container-page py-20">
      <h2 class="text-3xl font-semibold tracking-tight sm:text-4xl">{home.how.title}</h2>
      <ol class="mt-12 grid gap-8 md:grid-cols-3">
        {#each home.how.items as step, index (step.title)}
          <li class="flex flex-col gap-3">
            <span
              class="flex size-10 items-center justify-center rounded-full border border-accent/40 font-mono text-accent-soft"
              aria-hidden="true">{index + 1}</span
            >
            <h3 class="text-lg font-semibold">{step.title}</h3>
            <p class="text-muted">{step.text}</p>
          </li>
        {/each}
      </ol>
    </div>
  </section>

  <!-- For developers: other projects can offer NanoPrint to their users -->
  <section id="developers" class="container-page py-20">
    <div
      class="grid items-center gap-10 rounded-2xl border border-brand/30 bg-[linear-gradient(135deg,color-mix(in_oklab,var(--color-brand)_14%,transparent),transparent_60%)] p-6 sm:p-10 lg:grid-cols-[1fr_1.1fr]"
    >
      <div>
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{home.integrate.title}</h2>
        <p class="mt-4 text-lg text-muted">{home.integrate.text}</p>
        <ul class="mt-6 flex flex-col gap-2 text-sm">
          {#each home.integrate.points as point (point)}
            <li class="flex gap-2">
              <Check class="mt-0.5 size-4 shrink-0 text-accent" aria-hidden="true" />{point}
            </li>
          {/each}
        </ul>
        <a
          href={pages.developers[lang]}
          class="mt-8 inline-flex items-center gap-2 rounded-lg bg-brand px-5 py-2.5 font-semibold text-white transition-colors hover:bg-brand-hover"
        >
          {home.integrate.button}
          <ArrowRight class="size-4" aria-hidden="true" />
        </a>
      </div>
      <div class="min-w-0"><CodeBlock code={home.integrate.code} {lang} /></div>
    </div>
  </section>

  <!-- Printers and drivers -->
  <section class="container-page grid gap-12 py-12 lg:grid-cols-2">
    <div>
      <h2 class="text-3xl font-semibold tracking-tight sm:text-4xl">{home.printers.title}</h2>
      <p class="mt-4 text-lg text-muted">{home.printers.text}</p>
      <p class="mt-4 text-sm text-muted">{home.printers.note}</p>
    </div>
    <div class="rounded-2xl border border-line bg-surface p-6">
      <h3 class="text-lg font-semibold">{home.printers.driversTitle}</h3>
      <dl class="mt-4 flex flex-col divide-y divide-line">
        {#each home.printers.drivers as driver (driver.os)}
          <div class="flex flex-col gap-1 py-4 first:pt-0 last:pb-0">
            <dt class="font-medium">{driver.os}</dt>
            <dd class="text-sm text-muted">{driver.text}</dd>
          </div>
        {/each}
      </dl>
    </div>
  </section>

  <!-- Downloads -->
  <section id="download" class="container-page py-20">
    <div class="max-w-2xl">
      <h2 class="text-3xl font-semibold tracking-tight sm:text-4xl">{home.downloads.title}</h2>
      <p class="mt-4 text-lg text-muted">{home.downloads.lead}</p>
    </div>
    <div class="mt-12 grid gap-6 lg:grid-cols-3">
      {#each platforms as platform (platform.id)}
        <article
          id={platform.id}
          class="flex flex-col gap-5 rounded-2xl border border-line bg-surface p-6"
        >
          <div>
            <h3 class="text-xl font-semibold">{platform.name}</h3>
            <p class="mt-1 text-sm text-muted">
              <span class="sr-only">{home.downloads.requirements}:</span>{platform.requirements}
            </p>
          </div>
          <div class="flex flex-col gap-2">
            {#each platform.files as file, index (file.download)}
              <a
                href={downloads[file.download]}
                class={[
                  'inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-semibold transition-colors',
                  index === 0
                    ? 'bg-brand text-white hover:bg-brand-hover'
                    : 'border border-line text-fg hover:border-muted',
                ]}
              >
                <Download class="size-4" aria-hidden="true" />
                {file.label}
              </a>
            {/each}
          </div>
          <div class="border-t border-line pt-4">
            <h4 class="text-sm font-medium">{home.downloads.install}</h4>
            <ol
              class="mt-3 flex list-decimal flex-col gap-2 pl-5 text-sm text-muted marker:text-accent"
            >
              {#each platform.steps as step (step)}
                <li><Rich text={step} /></li>
              {/each}
            </ol>
          </div>
        </article>
      {/each}
    </div>

    <div class="mt-8 grid gap-6 lg:grid-cols-[2fr_1fr]">
      <div class="rounded-2xl border border-accent/30 bg-accent/5 p-6">
        <h3 class="font-semibold text-accent-soft">{home.downloads.afterTitle}</h3>
        <p class="mt-2 text-sm text-muted">{home.downloads.afterText}</p>
      </div>
      <div class="flex flex-col justify-center gap-3 p-2 text-sm">
        <a href={RELEASES_URL} class="link">{home.downloads.previous}</a>
        <details class="group text-muted">
          <summary
            class="flex cursor-pointer list-none items-center gap-1 hover:text-fg [&::-webkit-details-marker]:hidden"
          >
            {home.downloads.verifyTitle}
            <ChevronDown
              class="size-4 transition-transform group-open:rotate-180"
              aria-hidden="true"
            />
          </summary>
          <p class="mt-2 leading-relaxed"><Rich text={home.downloads.verifyText} /></p>
        </details>
      </div>
    </div>
  </section>

  <!-- Code signing policy: SignPath Foundation requires it on the home and download pages. -->
  <section id="code-signing-policy" class="container-page py-8">
    <div class="rounded-2xl border border-line bg-surface/60 p-6 sm:p-8">
      <h2 class="flex items-center gap-2 text-xl font-semibold">
        <ShieldCheck class="size-5 text-brand-soft" aria-hidden="true" />
        {home.signing.title}
      </h2>
      <ul class="mt-4 flex max-w-4xl flex-col gap-2 text-sm text-muted">
        {#each home.signing.items as item (item)}
          <li class="flex gap-2">
            <Check class="mt-0.5 size-4 shrink-0 text-accent" aria-hidden="true" />{item}
          </li>
        {/each}
      </ul>
      <p class="mt-4 flex flex-wrap gap-x-5 gap-y-1 text-sm">
        <a href="https://about.signpath.io/" class="link">SignPath.io</a>
        <a href="https://signpath.org/" class="link">SignPath Foundation</a>
        <a href={`${REPO_URL}#code-signing-policy`} class="link">GitHub</a>
      </p>
    </div>
  </section>

  <!-- Open source -->
  <section class="container-page py-12">
    <div
      class="flex flex-col items-start gap-6 rounded-2xl border border-line bg-surface p-8 sm:flex-row sm:items-center sm:justify-between sm:p-10"
    >
      <div class="max-w-2xl">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{home.openSource.title}</h2>
        <p class="mt-3 text-muted">{home.openSource.text}</p>
        <!-- AlternativeTo's dark compact badge, served from this site (img-src 'self'). -->
        <a href={ALTERNATIVETO_URL} class="mt-5 inline-block rounded-md">
          <img
            src="/badges/alternativeto.svg"
            alt={home.openSource.alternativeTo}
            width="117"
            height="40"
            class="h-10 w-auto"
          />
        </a>
      </div>
      <a
        href={REPO_URL}
        class="inline-flex shrink-0 items-center gap-2 rounded-lg border border-line px-5 py-2.5 font-semibold transition-colors hover:border-muted"
      >
        {home.openSource.button}
        <ArrowRight class="size-4" aria-hidden="true" />
      </a>
    </div>
  </section>

  <!-- FAQ -->
  <section id="faq" class="container-page py-20">
    <h2 class="text-3xl font-semibold tracking-tight sm:text-4xl">{home.faq.title}</h2>
    <div class="mt-10 flex max-w-3xl flex-col divide-y divide-line border-y border-line">
      {#each home.faq.items as item (item.q)}
        <details class="group py-5">
          <summary
            class="flex cursor-pointer list-none items-center justify-between gap-4 font-medium [&::-webkit-details-marker]:hidden"
          >
            <h3>{item.q}</h3>
            <ChevronDown
              class="size-5 shrink-0 text-muted transition-transform group-open:rotate-180"
              aria-hidden="true"
            />
          </summary>
          <p class="mt-3 leading-relaxed text-muted">{item.a}</p>
        </details>
      {/each}
    </div>
  </section>
</main>

<Footer {lang} />
