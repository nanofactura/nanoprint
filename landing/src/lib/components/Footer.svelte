<script lang="ts">
  import { messages, pages, type Lang } from '#lib/i18n/index.ts';
  import { docs, NANOFACTURA_URL, REPO_URL } from '#lib/site.ts';

  let { lang }: { lang: Lang } = $props();

  const t = $derived(messages(lang));
  const links = $derived([
    { href: REPO_URL, label: t.footer.links.github },
    { href: docs.license, label: t.footer.links.license },
    { href: docs.security, label: t.footer.links.security },
    { href: '/third-party-licenses.txt', label: t.footer.links.licenses },
    { href: pages.developers[lang], label: t.footer.links.developers },
    { href: `${pages.home[lang]}#code-signing-policy`, label: t.footer.links.signing },
  ]);
</script>

<footer class="mt-24 border-t border-line bg-surface/40">
  <div class="container-page grid gap-10 py-12 md:grid-cols-[1fr_auto]">
    <div class="flex flex-col gap-4">
      <img
        src="/brand/logo-nanoprint-dark.svg"
        alt="NanoPrint"
        width="123"
        height="50"
        class="h-10 w-fit"
      />
      <p class="text-sm text-muted">
        {t.footer.productOf}
        <a href={NANOFACTURA_URL} class="font-medium text-fg hover:underline">NanoFactura</a>.
      </p>
    </div>
    <nav aria-label="NanoPrint" class="flex flex-wrap gap-x-6 gap-y-2 text-sm md:justify-end">
      {#each links as link (link.href)}
        <a href={link.href} class="text-muted hover:text-fg">{link.label}</a>
      {/each}
    </nav>
  </div>
  <div class="container-page border-t border-line/60 py-8">
    <div class="flex max-w-4xl flex-col gap-2 text-xs leading-relaxed text-muted/80">
      {#each t.footer.legal as line (line)}
        <p>{line}</p>
      {/each}
      <p>{t.footer.privacy}</p>
      <p>© {new Date().getFullYear()} NanoFactura</p>
    </div>
  </div>
</footer>
