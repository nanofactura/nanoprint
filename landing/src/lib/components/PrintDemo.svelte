<!--
  Live demo: connects to NanoPrint on the visitor's computer with qz-tray.js, lists the printers it
  offers and prints a sample receipt built with ReceiptPrinterEncoder. Both libraries load on the
  first click, so the page stays light for everyone else.
-->
<script lang="ts">
  import Plug from '@lucide/svelte/icons/plug';
  import Printer from '@lucide/svelte/icons/printer';
  import type { Qz } from 'qz-tray';
  import { onDestroy } from 'svelte';
  import { format, messages, pages, type Lang } from '#lib/i18n/index.ts';
  import { SITE_URL } from '#lib/site.ts';

  let { lang }: { lang: Lang } = $props();

  const t = $derived(messages(lang).developers.demo);
  const locale = $derived(lang === 'es' ? 'es-AR' : 'en-US');

  let status = $state<'idle' | 'connecting' | 'connected'>('idle');
  let printers = $state<string[]>([]);
  let printer = $state('');
  let columns = $state(32);
  let withQr = $state(true);
  let printing = $state(false);
  let result = $state<{ ok: boolean; text: string; offerDownload?: boolean }>();

  let qz: Qz | undefined;

  async function loadQz(): Promise<Qz> {
    const { default: client } = await import('qz-tray');
    client.api.setPromiseType((resolver) => new Promise(resolver));
    // If NanoPrint quits or the user revokes the permission, start over.
    client.websocket.setClosedCallbacks(() => {
      status = 'idle';
      printers = [];
    });
    return client;
  }

  /** The demo's logo, sent to NanoPrint for its permission prompt (see the identity example). */
  async function logo(): Promise<string> {
    const blob = await (await fetch('/brand/demo-logo.png')).blob();
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        resolve(String(reader.result));
      };
      reader.onerror = () => {
        reject(reader.error ?? new Error('could not read the logo'));
      };
      reader.readAsDataURL(blob);
    });
  }

  function describe(error: unknown): { ok: boolean; text: string; offerDownload?: boolean } {
    const text = error instanceof Error ? error.message : String(error);
    if (/blocked/i.test(text)) return { ok: false, text: t.blocked };
    if (/establish|connect|closed/i.test(text) && status !== 'connected') {
      return { ok: false, text: t.notRunning, offerDownload: true };
    }
    return { ok: false, text: format(t.failed, { message: text }) };
  }

  /** `retries` gives a NanoPrint that is just starting time to open its servers. */
  async function connect(retries = 0): Promise<void> {
    status = 'connecting';
    result = undefined;
    try {
      qz ??= await loadQz();
      const icon = await logo();
      const name = lang === 'es' ? 'Demo de NanoPrint' : 'NanoPrint demo';
      qz.security.setCertificatePromise((resolve) => {
        resolve(JSON.stringify({ nanoprint: { name, icon } }));
      });
      // Only this computer: qz-tray.js would otherwise also try a host name on the internet.
      if (!qz.websocket.isActive()) {
        await qz.websocket.connect({ host: 'localhost', retries, delay: 1 });
      }
      printers = await qz.printers.find();
      const preferred = await qz.printers.getDefault();
      printer = preferred && printers.includes(preferred) ? preferred : (printers[0] ?? '');
      status = 'connected';
      if (printers.length === 0) result = { ok: false, text: t.noPrinters };
    } catch (error) {
      result = describe(error);
      status = 'idle';
    }
  }

  /** Starts an installed NanoPrint that the user quit (the browser asks first), then connects. */
  async function wake(): Promise<void> {
    window.location.href = 'nanoprint://start';
    await connect(15);
    // Not installed (the browser then does nothing) or the user declined the browser's question.
    if (status !== 'connected' && result?.offerDownload) {
      result = { ok: false, text: t.wakeFailed, offerDownload: true };
    }
  }

  async function disconnect(): Promise<void> {
    result = undefined;
    if (qz?.websocket.isActive()) await qz.websocket.disconnect();
    status = 'idle';
  }

  function money(amount: number): string {
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency: lang === 'es' ? 'ARS' : 'USD',
    })
      .format(amount)
      .replaceAll(' ', ' ');
  }

  /** `left` and `right` on one line of `width` characters. */
  function row(left: string, right: string, width: number): string {
    return left + ' '.repeat(Math.max(1, width - left.length - right.length)) + right;
  }

  async function receipt(): Promise<Uint8Array> {
    const { default: ReceiptPrinterEncoder } =
      await import('@point-of-sale/receipt-printer-encoder');
    const encoder = new ReceiptPrinterEncoder({ columns })
      .initialize()
      .align('center')
      .bold(true)
      .size(2, 2)
      .line('NanoPrint')
      .size(1, 1)
      .bold(false)
      .line(t.ticket.title)
      .line(new Date().toLocaleString(locale))
      .align('left')
      .line('-'.repeat(columns));
    let total = 0;
    for (const [name, quantity, price] of t.ticket.items) {
      total += quantity * price;
      encoder.line(row(`${String(quantity)} x ${name}`, money(quantity * price), columns));
    }
    encoder
      .line('-'.repeat(columns))
      .bold(true)
      .line(row(t.ticket.total, money(total), columns))
      .bold(false)
      .newline()
      .align('center');
    if (withQr) encoder.qrcode(SITE_URL, { model: 2, size: 6, errorlevel: 'm' });
    return encoder.line(t.ticket.thanks).newline(2).cut().encode();
  }

  function toBase64(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return btoa(binary);
  }

  async function print(): Promise<void> {
    if (!qz || !printer) return;
    printing = true;
    result = undefined;
    try {
      const config = qz.configs.create(printer, { jobName: t.ticket.jobName });
      const data = toBase64(await receipt());
      await qz.print(config, [{ type: 'raw', format: 'command', flavor: 'base64', data }]);
      result = { ok: true, text: format(t.printed, { printer }) };
    } catch (error) {
      result = describe(error);
    } finally {
      printing = false;
    }
  }

  onDestroy(() => {
    if (qz?.websocket.isActive()) void qz.websocket.disconnect();
  });
</script>

<div class="rounded-2xl border border-line bg-surface p-6 sm:p-8">
  <ul class="mb-6 flex flex-col gap-1.5 text-sm text-muted">
    {#each t.steps as step (step)}
      <li class="flex gap-2"><span class="text-accent" aria-hidden="true">•</span>{step}</li>
    {/each}
  </ul>

  {#if status !== 'connected'}
    <button
      type="button"
      class="inline-flex items-center gap-2 rounded-lg bg-brand px-5 py-2.5 font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
      disabled={status === 'connecting'}
      onclick={() => void connect()}
    >
      <Plug class="size-4" aria-hidden="true" />
      {status === 'connecting' ? t.connecting : t.connect}
    </button>
  {:else}
    <div class="flex flex-col gap-5">
      <p class="flex items-center gap-2 text-sm text-ok">
        <span class="size-2 rounded-full bg-ok" aria-hidden="true"></span>
        {t.connected}
      </p>
      {#if printers.length > 0}
        <div class="grid gap-4 sm:grid-cols-[1fr_auto]">
          <label class="flex flex-col gap-1.5 text-sm">
            <span class="text-muted">{t.printer}</span>
            <select
              bind:value={printer}
              class="rounded-lg border border-line bg-surface-2 px-3 py-2 text-fg"
            >
              {#each printers as name (name)}
                <option value={name}>{name}</option>
              {/each}
            </select>
          </label>
          <label class="flex flex-col gap-1.5 text-sm">
            <span class="text-muted">{t.paper}</span>
            <select
              bind:value={columns}
              class="rounded-lg border border-line bg-surface-2 px-3 py-2 text-fg"
            >
              <option value={32}>58 mm</option>
              <option value={48}>80 mm</option>
            </select>
          </label>
        </div>
        <label class="flex w-fit items-center gap-2 text-sm">
          <input type="checkbox" bind:checked={withQr} class="size-4 accent-brand" />
          {t.qr}
        </label>
      {/if}
      <div class="flex flex-wrap items-center gap-3">
        {#if printers.length > 0}
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-lg bg-brand px-5 py-2.5 font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
            disabled={printing}
            onclick={() => void print()}
          >
            <Printer class="size-4" aria-hidden="true" />
            {printing ? t.printing : t.print}
          </button>
        {/if}
        <button
          type="button"
          class="rounded-lg px-3 py-2.5 text-sm text-muted transition-colors hover:text-fg"
          onclick={() => void disconnect()}>{t.disconnect}</button
        >
      </div>
    </div>
  {/if}

  {#if result}
    <div
      role={result.ok ? 'status' : 'alert'}
      class={[
        'mt-5 rounded-lg border px-4 py-3 text-sm',
        result.ok ? 'border-ok/40 bg-ok/10 text-fg' : 'border-danger/40 bg-danger/10 text-fg',
      ]}
    >
      <p>{result.text}</p>
      {#if result.offerDownload}
        <div class="mt-3 flex flex-wrap items-center gap-4">
          <button
            type="button"
            class="rounded-lg border border-line bg-surface-2 px-4 py-2 font-medium text-fg transition-colors hover:border-muted"
            onclick={() => void wake()}>{t.wake}</button
          >
          <a href={`${pages.home[lang]}#download`} class="link">{t.download}</a>
        </div>
      {/if}
    </div>
  {/if}
</div>
