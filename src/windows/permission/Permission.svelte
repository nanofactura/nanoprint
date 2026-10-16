<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, tick } from 'svelte';
  import isologo from '../../../assets/brand/isologo-nanoprint.svg';
  import {
    answerPermissionRequest,
    getPermissionRequest,
    PERMISSION_REQUEST_CHANGED,
    type PermissionRequest,
  } from '$lib/agent';
  import SiteIcon from '$lib/components/SiteIcon.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Label } from '$lib/components/ui/label';
  import { setLocale, t } from '$lib/i18n/index.svelte';
  import { getSettings } from '$lib/settings';

  /** "Allow" stays disabled this long after a request appears, so a click meant for something
   * else (or a page tricking the user into double-clicking) cannot grant access. */
  const ALLOW_DELAY_MS = 1000;

  let request = $state<PermissionRequest | null>(null);
  let remember = $state(true);
  let allowEnabled = $state(false);
  let answering = $state(false);
  let blockButton = $state<HTMLButtonElement | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function load(): Promise<void> {
    const next = await getPermissionRequest();
    // Still the same site: keep what the user already chose on screen.
    if (next && next.origin === request?.origin) {
      request = next;
      return;
    }
    request = next;
    remember = request?.rememberable ?? false;
    answering = false;
    allowEnabled = false;
    clearTimeout(timer);
    timer = setTimeout(() => (allowEnabled = true), ALLOW_DELAY_MS);
    // The safe default: wait until the buttons of this request are on screen, then focus Block.
    await tick();
    blockButton?.focus();
  }

  async function answer(allow: boolean): Promise<void> {
    if (!request || answering) return;
    answering = true;
    await answerPermissionRequest(request.origin, allow, remember && request.rememberable);
  }

  onMount(() => {
    void getSettings().then((settings) => {
      setLocale(settings.resolvedLocale);
    });
    void load();
    const unlisten = listen(PERMISSION_REQUEST_CHANGED, () => void load());
    return () => {
      clearTimeout(timer);
      void unlisten.then((stop) => {
        stop();
      });
    };
  });
</script>

<main class="flex h-screen flex-col gap-4 p-5">
  <header class="flex items-center gap-4">
    <img src={isologo} alt="" class="h-9 w-auto" />
    <h1 class="text-base font-semibold">{t('permission.title')}</h1>
  </header>

  {#if request}
    <!-- The origin is what the user decides on; the name and logo are the site's own claims. -->
    <div class="flex items-center gap-3">
      <SiteIcon icon={request.icon} class="size-11" />
      <div class="min-w-0">
        <p class="font-mono text-sm font-semibold break-all select-text">{request.origin}</p>
        {#if request.name}
          <p class="text-xs text-muted-foreground">
            {t('permission.presentsAs', { name: request.name })}
          </p>
        {/if}
      </div>
    </div>
    <p class="text-sm">{t('permission.message')}</p>

    {#if request.rememberable}
      <div class="flex items-center gap-2">
        <Checkbox id="remember" bind:checked={remember} />
        <Label for="remember">{t('permission.remember')}</Label>
      </div>
    {:else}
      <p class="text-xs text-muted-foreground">{t('permission.httpsOnly')}</p>
    {/if}

    <div class="mt-auto flex justify-end gap-2">
      <Button
        bind:ref={blockButton}
        variant="outline"
        disabled={answering}
        onclick={() => void answer(false)}>{t('permission.block')}</Button
      >
      <Button disabled={!allowEnabled || answering} onclick={() => void answer(true)}
        >{t('permission.allow')}</Button
      >
    </div>
  {/if}
</main>
