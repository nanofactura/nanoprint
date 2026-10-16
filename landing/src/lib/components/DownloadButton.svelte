<!--
  The main download button. The HTML built ahead of time points to the downloads section; once the
  page runs, it offers the installer for the visitor's system. After a click it says the download is
  starting, then shows the installation steps, including how to get past the system's warning.
-->
<script lang="ts">
  import Download from '@lucide/svelte/icons/download';
  import LoaderCircle from '@lucide/svelte/icons/loader-circle';
  import { onMount } from 'svelte';
  import Rich from '$lib/components/Rich.svelte';
  import { messages, type Lang } from '$lib/i18n';
  import { detectPlatform, downloads, RELEASES_URL, type Platform } from '$lib/site';

  let { lang }: { lang: Lang } = $props();

  /** How long "your download is starting" shows before the installation steps. */
  const STARTING_MS = 3000;

  const t = $derived(messages(lang));
  /** `undefined` until detected; `null` for phones, tablets and unknown systems. */
  let platform = $state<Platform | null>();
  let phase = $state<'idle' | 'starting' | 'steps'>('idle');
  let timer: ReturnType<typeof setTimeout> | undefined;

  onMount(() => {
    platform = detectPlatform();
    return () => {
      clearTimeout(timer);
    };
  });

  const button = $derived.by(() => {
    switch (platform) {
      case 'windows':
        return {
          href: downloads.windows,
          label: t.download.windows,
          detail: t.home.downloads.windows.requirements,
          steps: t.home.downloads.windows.steps,
        };
      case 'mac':
        return {
          href: downloads.mac,
          label: t.download.mac,
          detail: t.home.downloads.mac.requirements,
          steps: t.home.downloads.mac.steps,
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

  /** The browser downloads the file; the page explains what comes next. */
  function started(): void {
    if (!button.steps) return;
    clearTimeout(timer);
    phase = 'starting';
    timer = setTimeout(() => (phase = 'steps'), STARTING_MS);
  }
</script>

<div class="flex flex-col gap-3">
  <a
    href={button.href}
    class="inline-flex w-fit items-center gap-3 rounded-xl bg-brand px-6 py-3.5 text-base font-semibold text-white shadow-lg shadow-brand/25 transition-colors hover:bg-brand-hover"
    onclick={started}
  >
    <Download class="size-5" aria-hidden="true" />
    {button.label}
  </a>

  <div aria-live="polite">
    {#if phase === 'starting'}
      <p class="flex items-center gap-2 text-sm text-fg">
        <LoaderCircle class="size-4 animate-spin text-brand-soft" aria-hidden="true" />
        {t.download.starting}
      </p>
    {:else if phase === 'steps' && button.steps}
      <div class="max-w-xl rounded-xl border border-accent/30 bg-accent/5 p-4">
        <p class="text-sm font-medium text-accent-soft">{t.download.nextSteps}</p>
        <ol
          class="mt-2 flex list-decimal flex-col gap-1.5 pl-5 text-sm text-muted marker:text-accent"
        >
          {#each button.steps as step (step)}
            <li><Rich text={step} /></li>
          {/each}
        </ol>
        <a href={button.href} class="mt-3 inline-block text-sm link">{t.download.retry}</a>
      </div>
    {:else}
      <p class="text-sm text-muted">{button.detail}</p>
    {/if}
  </div>
  {#if platform === null}
    <p class="text-sm text-accent-soft">{t.download.desktopOnly}</p>
  {/if}
  <p class="flex flex-wrap gap-x-4 gap-y-1 text-sm">
    <a href="#download" class="link">{t.download.otherPlatforms}</a>
    <a href={RELEASES_URL} class="text-muted hover:text-fg">{t.download.previousVersions}</a>
  </p>
</div>
