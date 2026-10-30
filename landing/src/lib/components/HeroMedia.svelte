<!--
  The hero animation (static/media/, see README.md). It plays once, muted, when the page opens and
  stays on its last frame; a click plays it again. Its background was graded to the page's color and
  its edges fade out, so it blends in without a frame.
-->
<script lang="ts">
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import { onMount } from 'svelte';
  import { messages, type Lang } from '#lib/i18n/index.ts';

  let { lang }: { lang: Lang } = $props();

  const t = $derived(messages(lang));
  let video = $state<HTMLVideoElement>();
  let ended = $state(false);

  function play(): void {
    if (!video) return;
    ended = false;
    video.currentTime = 0;
    // A browser may refuse to play; the still image stays.
    video.play().catch(() => {
      ended = true;
    });
  }

  onMount(() => {
    const element = video;
    if (!element) return;
    // Those who ask for less motion get the final frame. A click still plays it.
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      element.pause();
      const showEnd = () => {
        element.currentTime = Math.max(0, element.duration - 0.05);
        ended = true;
      };
      if (element.readyState >= HTMLMediaElement.HAVE_METADATA) showEnd();
      else element.addEventListener('loadedmetadata', showEnd, { once: true });
    }
  });
</script>

<button
  type="button"
  class="group relative block w-full cursor-pointer rounded-3xl"
  aria-label={t.home.hero.videoLabel}
  onclick={play}
>
  <video
    bind:this={video}
    class="aspect-video w-full [mask-image:radial-gradient(closest-side,#000_62%,transparent_100%)]"
    width="1280"
    height="720"
    autoplay
    muted
    playsinline
    preload="auto"
    poster="/media/hero-poster.webp"
    aria-hidden="true"
    onended={() => (ended = true)}
  >
    <source src="/media/hero.webm" type="video/webm" />
    <source src="/media/hero.mp4" type="video/mp4" />
  </video>
  <span
    class={[
      'absolute right-[12%] bottom-[12%] flex size-9 items-center justify-center rounded-full border border-line bg-surface/80 text-muted opacity-0 transition-opacity',
      ended && 'group-hover:opacity-100 group-focus-visible:opacity-100',
    ]}
    aria-hidden="true"
  >
    <RotateCcw class="size-4" />
  </span>
</button>
