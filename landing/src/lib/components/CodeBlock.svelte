<!-- A JavaScript snippet with light highlighting (comments, strings, keywords) and a copy button. -->
<script lang="ts">
  import Check from '@lucide/svelte/icons/check';
  import Copy from '@lucide/svelte/icons/copy';
  import { messages, type Lang } from '#lib/i18n/index.ts';

  let { code, lang, label = 'JavaScript' }: { code: string; lang: Lang; label?: string } = $props();

  const t = $derived(messages(lang));
  let copied = $state(false);

  const KEYWORDS = new Set([
    'import',
    'from',
    'const',
    'let',
    'await',
    'async',
    'function',
    'return',
    'new',
    'for',
    'of',
    'if',
  ]);

  /** Comments, strings and keywords get a color; everything else stays plain. */
  const tokens = $derived(
    Array.from(
      code.matchAll(/(\/\/[^\n]*)|('(?:[^'\\\n]|\\.)*'|`[^`]*`)|([A-Za-z_]\w*)|([^]+?)/g),
      (match) => {
        const [text, comment, string, word] = match;
        const kind = comment
          ? 'text-muted/80 italic'
          : string
            ? 'text-accent-soft'
            : word && KEYWORDS.has(word)
              ? 'text-brand-soft'
              : '';
        return { text, kind };
      },
    ),
  );

  async function copy(): Promise<void> {
    await navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="overflow-hidden rounded-xl border border-line bg-surface">
  <div class="flex items-center justify-between border-b border-line px-4 py-2">
    <span class="text-xs text-muted">{label}</span>
    <button
      type="button"
      class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted transition-colors hover:bg-surface-2 hover:text-fg"
      onclick={() => void copy()}
    >
      {#if copied}
        <Check class="size-3.5" aria-hidden="true" />
        {t.code.copied}
      {:else}
        <Copy class="size-3.5" aria-hidden="true" />
        {t.code.copy}
      {/if}
    </button>
  </div>
  <pre class="overflow-x-auto p-4 font-mono text-[0.8rem] leading-relaxed text-fg/90"><code
      >{#each tokens as token, index (index)}{#if token.kind}<span class={token.kind}
            >{token.text}</span
          >{:else}{token.text}{/if}{/each}</code
    ></pre>
</div>
