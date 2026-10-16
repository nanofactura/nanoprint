<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';
  import { onMount } from 'svelte';
  import isologo from '../../../assets/brand/isologo-nanoprint.svg';
  import {
    getServerStatus,
    installCertificate,
    listSites,
    revokeSite,
    type ServerStatus,
    type Site,
  } from '$lib/agent';
  import { Button } from '$lib/components/ui/button';
  import * as Tabs from '$lib/components/ui/tabs';
  import { describeError } from '$lib/errors';
  import { setLocale, t } from '$lib/i18n/index.svelte';
  import { getSettings, type SettingsView } from '$lib/settings';
  import GeneralTab from './GeneralTab.svelte';
  import HistoryTab from './HistoryTab.svelte';
  import PrintersTab from './PrintersTab.svelte';
  import SitesTab from './SitesTab.svelte';

  let settings = $state<SettingsView>();
  let status = $state<ServerStatus>();
  let sites = $state<Site[]>([]);
  let version = $state('');
  let error = $state('');
  let activating = $state(false);
  /** Bumped when the panel regains focus: sites and history change while it is open. */
  let refreshKey = $state(0);

  const statusText = $derived.by(() => {
    if (!status) return t('settings.server.starting');
    const { insecure, secure } = status;
    if (insecure.state === 'failed') {
      return t('settings.server.failed', { message: insecure.message });
    }
    if (insecure.state === 'starting') return t('settings.server.starting');
    const plain = `ws://127.0.0.1:${String(insecure.port)}`;
    return secure.state === 'listening'
      ? t('settings.server.listeningBoth', {
          insecure: plain,
          secure: `wss://localhost:${String(secure.port)}`,
        })
      : t('settings.server.listening', { address: plain });
  });

  function applySettings(view: SettingsView): void {
    settings = view;
    setLocale(view.resolvedLocale);
  }

  function showError(message: string): void {
    error = message;
  }

  async function refresh(): Promise<void> {
    try {
      [status, sites] = await Promise.all([getServerStatus(), listSites()]);
      refreshKey += 1;
    } catch (e) {
      error = t('error.load', { message: describeError(e) });
    }
  }

  /** The system asks the user to confirm (a Windows dialog, a macOS password prompt). */
  async function activate(): Promise<void> {
    activating = true;
    try {
      await installCertificate();
    } catch (e) {
      error = t('error.save', { message: describeError(e) });
    } finally {
      activating = false;
      await refresh();
    }
  }

  async function revoke(origin: string): Promise<void> {
    try {
      await revokeSite(origin);
      await refresh();
    } catch (e) {
      error = t('error.save', { message: describeError(e) });
    }
  }

  onMount(() => {
    void (async () => {
      try {
        const [view, appVersion] = await Promise.all([getSettings(), getVersion()]);
        applySettings(view);
        version = appVersion;
        await refresh();
      } catch (e) {
        error = t('error.load', { message: describeError(e) });
      }
    })();
    const onFocus = () => void refresh();
    window.addEventListener('focus', onFocus);
    return () => {
      window.removeEventListener('focus', onFocus);
    };
  });
</script>

<main class="mx-auto flex h-screen max-w-xl flex-col gap-4 p-6">
  <header class="flex shrink-0 items-center gap-4">
    <img src={isologo} alt="" class="h-10 w-auto" />
    <div class="min-w-0">
      <h1 class="text-lg font-semibold">{t('app.name')}</h1>
      <p class="text-sm text-muted-foreground">{t('app.tagline')}</p>
      <!-- Text a user may need to copy is selectable; the rest of the UI is not. -->
      <p
        class={[
          'text-xs select-text',
          status?.insecure.state === 'failed' ? 'text-destructive' : 'text-muted-foreground',
        ]}
      >
        {statusText}
      </p>
    </div>
  </header>

  {#if error}
    <p role="alert" class="text-sm text-destructive select-text">{error}</p>
  {/if}

  {#if status?.secure.state === 'listening' && status.certificate?.trusted === false}
    <div class="flex shrink-0 items-center justify-between gap-3 rounded-lg border bg-muted/50 p-3">
      <p class="text-sm">{t('settings.secureBanner')}</p>
      <Button size="sm" disabled={activating} onclick={() => void activate()}>
        {activating ? t('settings.https.waiting') : t('settings.https.install')}
      </Button>
    </div>
  {/if}

  {#if settings}
    <Tabs.Root value="printers" class="min-h-0 flex-1">
      <Tabs.List class="w-full shrink-0">
        <Tabs.Trigger value="printers">{t('settings.tabs.printers')}</Tabs.Trigger>
        <Tabs.Trigger value="sites">{t('settings.tabs.sites')}</Tabs.Trigger>
        <Tabs.Trigger value="history">{t('settings.tabs.history')}</Tabs.Trigger>
        <Tabs.Trigger value="general">{t('settings.tabs.general')}</Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="printers" class="flex min-h-0 flex-col">
        <PrintersTab {refreshKey} onerror={showError} />
      </Tabs.Content>
      <Tabs.Content value="sites" class="flex min-h-0 flex-col">
        <SitesTab {sites} onrevoke={(origin: string) => void revoke(origin)} />
      </Tabs.Content>
      <Tabs.Content value="history" class="flex min-h-0 flex-col">
        <HistoryTab {refreshKey} />
      </Tabs.Content>
      <!-- The padding keeps the cards' outer ring inside the scrolling box, which clips it. -->
      <Tabs.Content value="general" class="-m-1 min-h-0 overflow-y-auto p-1">
        <GeneralTab
          {settings}
          {status}
          {version}
          onsettings={applySettings}
          onstatuschange={refresh}
          onerror={showError}
        />
      </Tabs.Content>
    </Tabs.Root>
  {/if}
</main>
