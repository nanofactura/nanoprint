<script lang="ts">
  import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
  import { listPrinters, printTestPage, setPrinterShared, type PrinterView } from '$lib/agent';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { describeError } from '$lib/errors';
  import { t } from '$lib/i18n/index.svelte';

  /** `refreshKey` changes when the panel regains focus, so the list reloads. */
  let { refreshKey, onerror }: { refreshKey: number; onerror: (message: string) => void } =
    $props();

  let printers = $state<PrinterView[]>();
  let loading = $state(false);
  /** The printer a test page is being sent to. */
  let testing = $state<string>();
  let notice = $state('');

  /**
   * `refresh` asks the system again; otherwise a list from the last 30 seconds is reused.
   * `_refreshKey` is only read so the effect below runs again when it changes.
   */
  async function load(refresh: boolean, _refreshKey?: number): Promise<void> {
    loading = true;
    try {
      printers = await listPrinters(refresh);
    } catch (e) {
      onerror(t('error.load', { message: describeError(e) }));
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void load(false, refreshKey);
  });

  async function onSharedChange(printer: PrinterView, shared: boolean): Promise<void> {
    try {
      await setPrinterShared(printer.name, shared);
    } catch (e) {
      printer.shared = !shared;
      onerror(t('error.save', { message: describeError(e) }));
    }
  }

  async function test(name: string): Promise<void> {
    testing = name;
    notice = '';
    try {
      await printTestPage(name);
      notice = t('settings.printers.testSent', { printer: name });
    } catch (e) {
      onerror(describeError(e));
    } finally {
      testing = undefined;
    }
  }

  function details(printer: PrinterView): string {
    const parts = [];
    if (printer.driver) parts.push(t('settings.printers.driver', { driver: printer.driver }));
    if (printer.connection) {
      parts.push(t('settings.printers.connection', { connection: printer.connection }));
    }
    return parts.join(' · ');
  }
</script>

<Card.Root class="min-h-0">
  <Card.Header>
    <Card.Title>{t('settings.tabs.printers')}</Card.Title>
    <Card.Description>{t('settings.printers.description')}</Card.Description>
    <Card.Action>
      <Button variant="outline" size="sm" disabled={loading} onclick={() => void load(true)}>
        <RefreshCwIcon class={['size-4', loading && 'animate-spin']} />
        {t('settings.printers.refresh')}
      </Button>
    </Card.Action>
  </Card.Header>
  <Card.Content class="flex min-h-0 flex-col gap-3">
    {#if printers?.length === 0}
      <p class="text-sm text-muted-foreground">{t('settings.printers.empty')}</p>
    {:else if printers}
      <ul class="-mr-2 flex min-h-0 flex-col divide-y overflow-y-auto pr-2">
        {#each printers as printer, index (printer.name)}
          <li class="flex flex-col gap-2 py-3">
            <div class="flex min-w-0 flex-col gap-0.5">
              <p class="flex min-w-0 items-center gap-2">
                <span class="truncate text-sm font-medium select-text" title={printer.name}
                  >{printer.name}</span
                >
                {#if printer.default}
                  <Badge variant="secondary">{t('settings.printers.default')}</Badge>
                {/if}
              </p>
              {#if details(printer)}
                <p class="truncate text-xs text-muted-foreground select-text">
                  {details(printer)}
                </p>
              {/if}
              {#if printer.isVirtual}
                <p class="text-xs text-muted-foreground">{t('settings.printers.virtual')}</p>
              {/if}
            </div>
            <div class="flex min-h-8 items-center justify-between gap-4">
              <div class="flex items-center gap-2">
                <Switch
                  id={`printer-${String(index)}`}
                  bind:checked={printer.shared}
                  onCheckedChange={(shared: boolean) => void onSharedChange(printer, shared)}
                />
                <Label for={`printer-${String(index)}`} class="text-sm font-normal">
                  {printer.shared ? t('settings.printers.shared') : t('settings.printers.hidden')}
                </Label>
              </div>
              <!-- A virtual printer would save the test page's bytes as a broken file. -->
              {#if !printer.isVirtual}
                <Button
                  variant="outline"
                  size="sm"
                  disabled={testing !== undefined}
                  onclick={() => void test(printer.name)}
                >
                  {testing === printer.name
                    ? t('settings.printers.testing')
                    : t('settings.printers.test')}
                </Button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
      {#if notice}
        <p role="status" class="text-xs text-muted-foreground">{notice}</p>
      {/if}
      <p class="text-xs text-muted-foreground">{t('settings.printers.testHint')}</p>
    {/if}
  </Card.Content>
</Card.Root>
