<script lang="ts">
  import PlusIcon from '@lucide/svelte/icons/plus';
  import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
  import {
    addNetworkPrinter,
    listPrinters,
    printTestPage,
    removeNetworkPrinter,
    setPrinterShared,
    type PrinterView,
  } from '$lib/agent';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { describeError } from '$lib/errors';
  import { t } from '$lib/i18n/index.svelte';

  /** The port of raw TCP printing, which nearly every network receipt printer uses. */
  const RAW_PORT = '9100';

  /** `refreshKey` changes when the panel regains focus, so the list reloads. */
  let { refreshKey, onerror }: { refreshKey: number; onerror: (message: string) => void } =
    $props();

  let printers = $state<PrinterView[]>();
  let loading = $state(false);
  /** The printer a test page is being sent to. */
  let testing = $state<string>();
  let notice = $state('');
  /** The form to add a network printer, while it is open. */
  let adding = $state<{ name: string; host: string; port: string }>();
  let addError = $state('');
  let saving = $state(false);

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

  async function add(): Promise<void> {
    if (!adding) return;
    const port = Number(adding.port.trim());
    if (!Number.isInteger(port) || port < 1 || port > 65535) {
      addError = t('error.code.network_port_invalid');
      return;
    }
    saving = true;
    addError = '';
    try {
      await addNetworkPrinter(adding.name, adding.host, port);
      adding = undefined;
      await load(false);
    } catch (e) {
      addError = describeError(e);
    } finally {
      saving = false;
    }
  }

  async function remove(name: string): Promise<void> {
    try {
      await removeNetworkPrinter(name);
      await load(false);
    } catch (e) {
      onerror(t('error.save', { message: describeError(e) }));
    }
  }

  function details(printer: PrinterView): string {
    if (printer.network) {
      return t('settings.printers.address', { address: printer.connection ?? '' });
    }
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
                {#if printer.network}
                  <Badge variant="outline">{t('settings.printers.network')}</Badge>
                {/if}
              </p>
              {#if details(printer)}
                <p class="truncate text-xs text-muted-foreground select-text">
                  {details(printer)}
                </p>
              {/if}
              {#if printer.limitation}
                <p class="text-xs text-muted-foreground">
                  {printer.limitation === 'virtual'
                    ? t('settings.printers.virtual')
                    : t('settings.printers.easyPrint')}
                </p>
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
              <div class="flex items-center gap-2">
                {#if printer.network}
                  <Button variant="ghost" size="sm" onclick={() => void remove(printer.name)}>
                    {t('settings.printers.remove')}
                  </Button>
                {/if}
                <!-- The test page's raw bytes would not reach the paper. -->
                {#if !printer.limitation}
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
            </div>
          </li>
        {/each}
      </ul>
      {#if notice}
        <p role="status" class="text-xs text-muted-foreground">{notice}</p>
      {/if}
      <p class="text-xs text-muted-foreground">{t('settings.printers.testHint')}</p>
    {/if}

    {#if adding}
      <form
        class="flex flex-col gap-3 rounded-md border p-3"
        onsubmit={(event) => {
          event.preventDefault();
          void add();
        }}
      >
        <p class="text-xs text-muted-foreground">{t('settings.printers.addDescription')}</p>
        <div class="flex flex-col gap-2">
          <Label for="network-name">{t('settings.printers.name')}</Label>
          <Input
            id="network-name"
            maxlength={64}
            placeholder={t('settings.printers.namePlaceholder')}
            bind:value={adding.name}
          />
        </div>
        <div class="grid grid-cols-[1fr_6rem] gap-3">
          <div class="flex flex-col gap-2">
            <Label for="network-host">{t('settings.printers.host')}</Label>
            <Input
              id="network-host"
              maxlength={253}
              placeholder="192.168.1.50"
              spellcheck={false}
              autocapitalize="off"
              bind:value={adding.host}
            />
          </div>
          <div class="flex flex-col gap-2">
            <Label for="network-port">{t('settings.printers.port')}</Label>
            <Input id="network-port" inputmode="numeric" maxlength={5} bind:value={adding.port} />
          </div>
        </div>
        {#if addError}
          <p role="alert" class="text-xs text-destructive">{addError}</p>
        {/if}
        <div class="flex justify-end gap-2">
          <Button
            variant="ghost"
            size="sm"
            onclick={() => {
              adding = undefined;
              addError = '';
            }}>{t('settings.printers.cancel')}</Button
          >
          <Button
            type="submit"
            size="sm"
            disabled={saving || !adding.name.trim() || !adding.host.trim()}
            >{t('settings.printers.confirmAdd')}</Button
          >
        </div>
      </form>
    {:else if printers}
      <Button
        variant="outline"
        size="sm"
        class="self-start"
        onclick={() => (adding = { name: '', host: '', port: RAW_PORT })}
      >
        <PlusIcon class="size-4" />
        {t('settings.printers.add')}
      </Button>
    {/if}
  </Card.Content>
</Card.Root>
