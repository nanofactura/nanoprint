<script lang="ts">
  import {
    historyCategories,
    isHistoryCategory,
    isHistoryKind,
    listHistory,
    type HistoryEntry,
  } from '$lib/agent';
  import { Badge } from '$lib/components/ui/badge';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import * as Select from '$lib/components/ui/select';
  import { formatDateTime, formatNumber } from '$lib/format';
  import { describeError } from '$lib/errors';
  import { t } from '$lib/i18n/index.svelte';

  /** `refreshKey` changes when the panel regains focus, so the list reloads. */
  let { refreshKey }: { refreshKey: number } = $props();

  /** Typing waits this long before the history is queried. */
  const SEARCH_DELAY_MS = 250;
  const ALL = 'all';

  let search = $state('');
  let query = $state('');
  let category = $state(ALL);
  let entries = $state<HistoryEntry[]>([]);
  let error = $state('');
  /** Only the latest request updates the list, whatever order the answers arrive in. */
  let latest = 0;

  $effect(() => {
    const text = search;
    const timer = setTimeout(() => (query = text.trim()), SEARCH_DELAY_MS);
    return () => {
      clearTimeout(timer);
    };
  });

  /** `_refreshKey` is only read so the effect below runs again when it changes. */
  async function load(text: string, selected: string, _refreshKey: number): Promise<void> {
    const request = ++latest;
    try {
      const found = await listHistory(text || null, isHistoryCategory(selected) ? selected : null);
      if (request !== latest) return;
      entries = found;
      error = '';
    } catch (e) {
      error = t('error.load', { message: describeError(e) });
    }
  }

  $effect(() => {
    void load(query, category, refreshKey);
  });

  const categoryOptions = $derived([
    { value: ALL, label: t('settings.history.allEvents') },
    ...historyCategories.map((value) => ({ value, label: t(`history.category.${value}`) })),
  ]);
  const categoryLabel = $derived(
    categoryOptions.find((option) => option.value === category)?.label ?? '',
  );
  const filtered = $derived(query !== '' || category !== ALL);

  function kindLabel(kind: string): string {
    return isHistoryKind(kind) ? t(`history.kind.${kind}`) : kind;
  }

  /** Mirrors the codes `server.rs` (refused connections) and `agent.rs` (failed jobs) store. */
  const reasonCodes = [
    'missing_host',
    'foreign_host',
    'missing_origin',
    'invalid_origin',
    'network_printer',
    'printer_not_found',
    'printer_hidden',
  ] as const;
  type ReasonCode = (typeof reasonCodes)[number];

  /** NanoPrint stores its own reasons as codes; other errors come from the system as text. */
  function errorText(error: string): string {
    const isCode = (value: string): value is ReasonCode =>
      (reasonCodes as readonly string[]).includes(value);
    return isCode(error) ? t(`history.reason.${error}`) : error;
  }

  function isFailure(kind: string): boolean {
    return ['job_failed', 'permission_denied', 'connection_rejected', 'rate_limited'].includes(
      kind,
    );
  }

  function details(entry: HistoryEntry): string {
    const parts = [entry.printer, entry.jobName];
    if (entry.bytes !== null) {
      parts.push(t('settings.history.bytes', { size: formatNumber(entry.bytes) }));
    }
    if (entry.durationMs !== null) parts.push(`${formatNumber(entry.durationMs)} ms`);
    return parts.filter(Boolean).join(' · ');
  }

  /** Jobs without a site are test pages printed from the Printers tab. */
  function source(entry: HistoryEntry): string {
    if (entry.origin !== null) return entry.origin;
    return entry.kind === 'job_printed' || entry.kind === 'job_failed'
      ? t('settings.history.testPage')
      : t('settings.history.noSite');
  }
</script>

<Card.Root class="min-h-0">
  <Card.Header>
    <Card.Title>{t('settings.tabs.history')}</Card.Title>
    <Card.Description>{t('settings.history.description')}</Card.Description>
  </Card.Header>
  <Card.Content class="flex min-h-0 flex-col gap-3">
    <div class="flex gap-2">
      <Input
        type="search"
        bind:value={search}
        placeholder={t('settings.history.search')}
        aria-label={t('settings.history.search')}
        class="min-w-0 flex-1"
      />
      <Select.Root type="single" bind:value={category}>
        <Select.Trigger class="w-40 shrink-0" aria-label={t('settings.history.category')}>
          {categoryLabel}
        </Select.Trigger>
        <Select.Content>
          {#each categoryOptions as option (option.value)}
            <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    </div>

    {#if error}
      <p role="alert" class="text-sm text-destructive select-text">{error}</p>
    {:else if entries.length === 0}
      <p class="text-sm text-muted-foreground">
        {filtered ? t('settings.history.noMatches') : t('settings.history.empty')}
      </p>
    {:else}
      <ul class="-mr-2 flex min-h-0 flex-col divide-y overflow-y-auto pr-2">
        {#each entries as entry, index (index)}
          <li class="flex flex-col gap-0.5 py-2">
            <div class="flex items-center justify-between gap-2">
              <Badge variant={isFailure(entry.kind) ? 'destructive' : 'secondary'}>
                {kindLabel(entry.kind)}
              </Badge>
              <time class="text-xs text-muted-foreground">{formatDateTime(entry.at)}</time>
            </div>
            <p class="truncate text-xs" title={entry.origin ?? ''}>
              <span class={[entry.origin !== null && 'font-mono select-text']}>{source(entry)}</span
              >
              {#if entry.siteName}
                <span class="text-muted-foreground">· «{entry.siteName}»</span>
              {/if}
            </p>
            {#if details(entry)}
              <p class="text-xs text-muted-foreground">{details(entry)}</p>
            {/if}
            {#if entry.error}
              <p class="text-xs text-destructive select-text">{errorText(entry.error)}</p>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </Card.Content>
</Card.Root>
