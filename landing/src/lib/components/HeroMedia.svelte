<!--
  The hero's looping video (static/media/hero.webm and hero.mp4), over a drawn stand-in: a web page
  printing a receipt. The stand-in shows while the video loads, and stays when it is missing.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { messages, type Lang } from '$lib/i18n';

  let { lang }: { lang: Lang } = $props();

  const t = $derived(messages(lang));
  let video = $state<HTMLVideoElement>();
  let failed = $state(false);

  onMount(() => {
    // Autoplay cannot be turned off from CSS: stop on the first frame for those who ask.
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) video?.pause();
  });

  const lines = [
    'Kiosco Don Pepe',
    '------------------',
    'Café      $ 2.500',
    'Medialunas $ 2.700',
  ];
</script>

<div
  class="relative aspect-[5/4] w-full overflow-hidden rounded-2xl border border-line bg-surface shadow-2xl shadow-brand/10"
>
  <!-- Stand-in: a browser window and the receipt coming out of the printer. -->
  <div aria-hidden="true" class="absolute inset-0 flex items-center justify-center p-8">
    <div
      class="absolute inset-0 bg-[radial-gradient(circle_at_70%_20%,color-mix(in_oklab,var(--color-brand)_25%,transparent),transparent_60%)]"
    ></div>
    <div class="relative w-[58%] rounded-lg border border-line bg-surface-2 shadow-xl">
      <div class="flex gap-1.5 border-b border-line px-3 py-2">
        <span class="size-2 rounded-full bg-line"></span>
        <span class="size-2 rounded-full bg-line"></span>
        <span class="size-2 rounded-full bg-line"></span>
      </div>
      <div class="flex flex-col gap-2 p-4">
        <span class="h-2 w-2/3 rounded bg-line"></span>
        <span class="h-2 w-1/2 rounded bg-line"></span>
        <span
          class="mt-2 w-fit rounded-md bg-brand px-3 py-1.5 text-[0.65rem] font-semibold text-white"
        >
          {lang === 'es' ? 'Imprimir ticket' : 'Print receipt'}
        </span>
      </div>
    </div>
    <div
      class="relative mt-16 -ml-6 w-[40%] min-w-0 rotate-3 bg-[#f4f1ea] px-2 pt-3 pb-5 font-mono text-[0.45rem] leading-relaxed text-neutral-800 shadow-xl [clip-path:polygon(0_0,100%_0,100%_100%,90%_96%,80%_100%,70%_96%,60%_100%,50%_96%,40%_100%,30%_96%,20%_100%,10%_96%,0_100%)] sm:px-3 sm:text-[0.55rem]"
    >
      {#each lines as line (line)}
        <p class="whitespace-pre">{line}</p>
      {/each}
      <p class="mt-1 font-bold">TOTAL $ 5.200</p>
    </div>
  </div>

  {#if !failed}
    <video
      bind:this={video}
      class="absolute inset-0 size-full object-cover"
      autoplay
      muted
      loop
      playsinline
      preload="metadata"
      aria-label={t.home.hero.videoLabel}
    >
      <source src="/media/hero.webm" type="video/webm" />
      <source
        src="/media/hero.mp4"
        type="video/mp4"
        onerror={() => {
          failed = true;
        }}
      />
    </video>
  {/if}
</div>
