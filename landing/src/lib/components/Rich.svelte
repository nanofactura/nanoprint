<!-- Text from the message catalogs: anything between backticks renders as inline code. -->
<script lang="ts">
  let { text }: { text: string } = $props();

  const parts = $derived(text.split('`'));
</script>

{#each parts as part, index (index)}{#if index % 2 === 1}<code
      class="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[0.9em] break-words text-fg"
      >{part}</code
    >{:else}{part}{/if}{/each}
