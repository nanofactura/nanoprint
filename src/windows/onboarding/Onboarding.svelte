<script lang="ts">
  import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import isologo from '../../../assets/brand/isologo-nanoprint.svg';
  import {
    getOnboardingReason,
    getServerStatus,
    installCertificate,
    type OnboardingReason,
    type ServerStatus,
  } from '$lib/agent';
  import { Button } from '$lib/components/ui/button';
  import { setLocale, t } from '$lib/i18n/index.svelte';
  import { getSettings } from '$lib/settings';

  /** On first run the certificate is still being created: check again this often. */
  const STATUS_RETRY_MS = 500;

  let reason = $state<OnboardingReason>('welcome');
  let status = $state<ServerStatus>();
  let waiting = $state(false);
  let activated = $state(false);
  let failed = $state(false);

  const platform = navigator.userAgent.includes('Windows')
    ? 'windows'
    : navigator.userAgent.includes('Mac')
      ? 'mac'
      : 'linux';
  /** Shown before the system's own confirmation, so it does not come as a surprise. */
  const confirmHint = $derived(
    platform === 'windows'
      ? t('onboarding.secure.confirmWindows')
      : platform === 'mac'
        ? t('onboarding.secure.confirmMac')
        : '',
  );

  const needsCertificate = $derived(
    status?.secure.state === 'listening' && status.certificate?.trusted === false,
  );

  async function loadStatus(): Promise<void> {
    status = await getServerStatus();
    if (
      status.secure.state === 'starting' ||
      (status.secure.state === 'listening' && !status.certificate)
    ) {
      setTimeout(() => void loadStatus(), STATUS_RETRY_MS);
    }
  }

  async function activate(): Promise<void> {
    waiting = true;
    failed = false;
    try {
      await installCertificate();
      status = await getServerStatus();
      activated = status.certificate?.trusted === true;
      failed = !activated;
    } catch {
      // Most often the user declined the system's confirmation; they can try again.
      failed = true;
    } finally {
      waiting = false;
    }
  }

  function close(): void {
    void getCurrentWindow().close();
  }

  onMount(() => {
    void (async () => {
      const [settings, current] = await Promise.all([getSettings(), getOnboardingReason()]);
      setLocale(settings.resolvedLocale);
      reason = current;
      await loadStatus();
    })();
  });
</script>

<!-- min-h-screen: if a translation runs long, the page scrolls instead of squeezing the buttons. -->
<main class="flex min-h-screen flex-col gap-4 p-5">
  <header class="flex items-center gap-4">
    <img src={isologo} alt="" class="h-9 w-auto" />
    <h1 class="text-base font-semibold">
      {reason === 'welcome' ? t('onboarding.welcome.title') : t('onboarding.refused.title')}
    </h1>
  </header>

  {#if reason === 'welcome'}
    <p class="text-sm">{t('onboarding.welcome.running')}</p>
    <!-- Starting with the system is a change to the user's computer: say so, and how to undo it. -->
    <p class="text-sm text-muted-foreground">{t('onboarding.welcome.autostart')}</p>
    {#if platform === 'windows'}
      <!-- Windows hides new tray icons, and apps cannot pin themselves: say where it is. -->
      <p class="text-xs text-muted-foreground">{t('onboarding.welcome.trayWindows')}</p>
    {/if}
    <p class="text-sm text-muted-foreground">{t('onboarding.welcome.permission')}</p>
  {:else}
    <p class="text-sm">{t('onboarding.refused.message')}</p>
  {/if}

  {#if needsCertificate || activated}
    <section class="flex flex-col gap-2 rounded-lg border p-4">
      <h2 class="flex items-center gap-2 text-sm font-medium">
        <ShieldCheckIcon class="size-4" />
        {t('onboarding.secure.title')}
      </h2>
      {#if activated}
        <p class="text-sm">
          {reason === 'welcome' ? t('onboarding.secure.done') : t('onboarding.secure.doneReload')}
        </p>
      {:else}
        <p class="text-sm text-muted-foreground">{t('onboarding.secure.description')}</p>
        {#if confirmHint}
          <p class="text-xs text-muted-foreground">{confirmHint}</p>
        {/if}
        {#if failed}
          <p role="alert" class="text-xs text-destructive">{t('onboarding.secure.failed')}</p>
        {/if}
      {/if}
    </section>
  {/if}

  <footer class={['mt-auto flex justify-end gap-2', !status && 'invisible']}>
    {#if needsCertificate && !activated}
      <Button variant="outline" disabled={waiting} onclick={close}>{t('onboarding.later')}</Button>
      <Button disabled={waiting} onclick={() => void activate()}>
        {waiting ? t('onboarding.secure.waiting') : t('onboarding.secure.activate')}
      </Button>
    {:else}
      <Button onclick={close}>{t('onboarding.close')}</Button>
    {/if}
  </footer>
</main>
