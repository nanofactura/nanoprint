<script lang="ts">
  import Menu from '@lucide/svelte/icons/menu';
  import { messages, pages, type Lang, type Page } from '$lib/i18n';
  import { REPO_URL } from '$lib/site';

  let { lang, page }: { lang: Lang; page: Page } = $props();

  const t = $derived(messages(lang));
  const other = $derived<Lang>(lang === 'es' ? 'en' : 'es');
  // Section links work from every page: they lead back to the home page when needed.
  const home = $derived(page === 'home' ? '' : pages.home[lang]);
  const links = $derived([
    { href: `${home}#features`, label: t.nav.features },
    { href: `${home}#download`, label: t.nav.download },
    { href: `${home}#faq`, label: t.nav.faq },
    { href: pages.developers[lang], label: t.nav.developers },
  ]);

  /** A section link scrolls within the page: close the menu that covers it. */
  function close(event: MouseEvent): void {
    (event.currentTarget as HTMLElement).closest('details')?.removeAttribute('open');
  }
</script>

<a
  href="#main"
  class="sr-only z-50 rounded-md bg-brand px-4 py-2 text-white focus:not-sr-only focus:fixed focus:top-3 focus:left-3"
>
  {t.nav.skip}
</a>

<header class="sticky top-0 z-40 border-b border-line/70 bg-bg/80 backdrop-blur-md">
  <div class="container-page flex h-16 items-center justify-between gap-4">
    <a href={pages.home[lang]} class="shrink-0" aria-label="NanoPrint">
      <img
        src="/brand/logo-nanoprint-dark.svg"
        alt="NanoPrint"
        width="123"
        height="50"
        class="h-11 w-auto"
      />
    </a>

    <nav aria-label={t.nav.menu} class="hidden items-center gap-1 md:flex">
      {#each links as link (link.href)}
        <a
          href={link.href}
          class={[
            'rounded-md px-3 py-2 text-sm text-muted transition-colors hover:text-fg',
            link.href === pages[page][lang] && 'text-fg',
          ]}
          aria-current={link.href === pages[page][lang] ? 'page' : undefined}>{link.label}</a
        >
      {/each}
      <a
        href={REPO_URL}
        class="rounded-md px-3 py-2 text-sm text-muted transition-colors hover:text-fg"
        >{t.nav.github}</a
      >
    </nav>

    <div class="flex items-center gap-2">
      <a
        href={pages[page][other]}
        hreflang={other}
        lang={other}
        aria-label={t.nav.languageLabel}
        class="rounded-md border border-line px-3 py-1.5 text-sm text-muted transition-colors hover:border-muted hover:text-fg"
        >{t.nav.otherLanguage}</a
      >
      <!-- No JavaScript needed: the menu is a disclosure element. -->
      <details class="group relative md:hidden">
        <summary
          class="flex size-9 cursor-pointer list-none items-center justify-center rounded-md border border-line text-muted hover:text-fg [&::-webkit-details-marker]:hidden"
          aria-label={t.nav.menu}
        >
          <Menu class="size-5" />
        </summary>
        <nav
          aria-label={t.nav.menu}
          class="absolute right-0 mt-2 flex w-56 flex-col rounded-lg border border-line bg-surface p-2 shadow-xl"
        >
          {#each links as link (link.href)}
            <a
              href={link.href}
              class="rounded-md px-3 py-2 text-sm hover:bg-surface-2"
              onclick={close}>{link.label}</a
            >
          {/each}
          <a href={REPO_URL} class="rounded-md px-3 py-2 text-sm hover:bg-surface-2"
            >{t.nav.github}</a
          >
        </nav>
      </details>
    </div>
  </div>
</header>
