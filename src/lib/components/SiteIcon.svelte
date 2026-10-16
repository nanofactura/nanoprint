<script lang="ts">
  import GlobeIcon from '@lucide/svelte/icons/globe';
  import type { ClassValue } from 'svelte/elements';

  /** `icon` is a logo NanoPrint re-encoded itself (a `data:image/png` URL), never a remote URL. */
  let { icon, class: className }: { icon: string | null; class?: ClassValue } = $props();
</script>

{#if icon}
  <img src={icon} alt="" class={['shrink-0 rounded-md object-contain', className]} />
{:else}
  <div
    class={[
      'flex shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground',
      className,
    ]}
  >
    <GlobeIcon class="size-1/2" />
  </div>
{/if}
