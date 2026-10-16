<!--
  The main download button. The HTML built ahead of time points to the downloads section; once the
  page runs, it offers the installer for the visitor's system.
-->
<script lang="ts">
  import Download from '@lucide/svelte/icons/download';
  import { onMount } from 'svelte';
  import { messages, type Lang } from '$lib/i18n';
  import { detectPlatform, downloads, RELEASES_URL, type Platform } from '$lib/site';

  let { lang }: { lang: Lang } = $props();

  const t = $derived(messages(lang));
  /** `undefined` until detected; `null` for phones, tablets and unknown systems. */
  let platform = $state<Platform | null>();

  onMount(() => {
    platform = detectPlatform();
  });

  const button = $derived.by(() => {
    switch (platform) {
      case 'windows':
        return {
          href: downloads.windows,
          label: t.download.windows,
          detail: t.home.downloads.windows.requirements,
        };
      case 'mac':
        return {
          href: downloads.mac,
          label: t.download.mac,
          detail: t.home.downloads.mac.requirements,
        };
      // Several package formats: the visitor picks one in the downloads section.
      case 'linux':
        return {
          href: '#download-linux',
          label: t.download.linux,
          detail: t.home.downloads.linux.requirements,
        };
      default:
        return { href: '#download', label: t.download.generic, detail: t.home.hero.platforms };
    }
  });
</script>

<div class="flex flex-col gap-3">
  <a
    href={button.href}
    class="inline-flex w-fit items-center gap-3 rounded-xl bg-brand px-6 py-3.5 text-base font-semibold text-white shadow-lg shadow-brand/25 transition-colors hover:bg-brand-hover"
  >
    <Download class="size-5" aria-hidden="true" />
    {button.label}
  </a>
  <p class="text-sm text-muted">{button.detail}</p>
  {#if platform === null}
    <p class="text-sm text-accent-soft">{t.download.desktopOnly}</p>
  {/if}
  <p class="flex flex-wrap gap-x-4 gap-y-1 text-sm">
    <a href="#download" class="link">{t.download.otherPlatforms}</a>
    <a href={RELEASES_URL} class="text-muted hover:text-fg">{t.download.previousVersions}</a>
  </p>
</div>
