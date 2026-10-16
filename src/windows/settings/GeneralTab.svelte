<script lang="ts">
  import * as autostart from '@tauri-apps/plugin-autostart';
  import { onMount } from 'svelte';
  import {
    installCertificate,
    openLogFolder,
    removeCertificate,
    showLicenses,
    type ServerStatus,
  } from '$lib/agent';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Select from '$lib/components/ui/select';
  import { Separator } from '$lib/components/ui/separator';
  import { Switch } from '$lib/components/ui/switch';
  import { formatDate } from '$lib/format';
  import { describeError } from '$lib/errors';
  import { locales, t } from '$lib/i18n/index.svelte';
  import {
    isLanguagePreference,
    restart,
    setLanguage,
    setPorts,
    type LanguagePreference,
    type SettingsView,
  } from '$lib/settings';

  let {
    settings,
    status,
    version,
    onsettings,
    onstatuschange,
    onerror,
  }: {
    settings: SettingsView;
    status: ServerStatus | undefined;
    version: string;
    onsettings: (view: SettingsView) => void;
    onstatuschange: () => Promise<void>;
    onerror: (message: string) => void;
  } = $props();

  /** Firefox trusts the system store on Windows and macOS, but not on Linux. */
  const onLinux = navigator.userAgent.includes('Linux');

  let autostartEnabled = $state(false);
  let waitingForSystem = $state(false);
  /** The port fields, as typed. Empty means automatic. */
  let insecurePort = $state('');
  let securePort = $state('');
  let portError = $state('');
  /** New ports were saved: they apply once NanoPrint restarts. */
  let portsSaved = $state(false);

  const portsChanged = $derived(
    insecurePort.trim() !== portText(settings.insecurePort) ||
      securePort.trim() !== portText(settings.securePort),
  );

  const languageOptions = $derived<{ value: LanguagePreference; label: string }[]>([
    {
      value: 'system',
      label: t('settings.language.system', { language: t(`language.${settings.systemLocale}`) }),
    },
    ...locales.map((locale) => ({ value: locale, label: t(`language.${locale}`) })),
  ]);
  const selectedLanguageLabel = $derived(
    languageOptions.find((option) => option.value === settings.language)?.label ?? '',
  );

  onMount(() => {
    insecurePort = portText(settings.insecurePort);
    securePort = portText(settings.securePort);
    autostart
      .isEnabled()
      .then((enabled) => (autostartEnabled = enabled))
      .catch((e: unknown) => {
        onerror(t('error.load', { message: describeError(e) }));
      });
  });

  async function onLanguageChange(value: string): Promise<void> {
    if (!isLanguagePreference(value)) return;
    try {
      onsettings(await setLanguage(value));
    } catch (e) {
      onerror(t('error.save', { message: describeError(e) }));
    }
  }

  async function onAutostartChange(enabled: boolean): Promise<void> {
    try {
      await (enabled ? autostart.enable() : autostart.disable());
    } catch (e) {
      autostartEnabled = !enabled;
      onerror(t('error.save', { message: describeError(e) }));
    }
  }

  function portText(port: number | null): string {
    return port === null ? '' : String(port);
  }

  /** `null` for an empty field (automatic), `undefined` for anything but a usable port. */
  function parsePort(text: string): number | null | undefined {
    const trimmed = text.trim();
    if (trimmed === '') return null;
    const port = Number(trimmed);
    return /^\d+$/.test(trimmed) && port >= 1024 && port <= 65535 ? port : undefined;
  }

  async function savePorts(): Promise<void> {
    const insecure = parsePort(insecurePort);
    const secure = parsePort(securePort);
    if (insecure === undefined || secure === undefined) {
      portError = t('error.code.port_invalid');
      return;
    }
    try {
      onsettings(await setPorts(insecure, secure));
      portError = '';
      portsSaved = true;
    } catch (e) {
      portError = describeError(e);
    }
  }

  /** Both actions open a system confirmation (Windows dialog, macOS password prompt). */
  async function changeCertificate(install: boolean): Promise<void> {
    waitingForSystem = true;
    try {
      await (install ? installCertificate() : removeCertificate());
    } catch (e) {
      onerror(t('error.save', { message: describeError(e) }));
    } finally {
      waitingForSystem = false;
      await onstatuschange();
    }
  }
</script>

<div class="flex flex-col gap-4">
  <Card.Root>
    <Card.Header>
      <Card.Title>{t('settings.general.title')}</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col gap-4">
      <div class="flex items-center justify-between gap-4">
        <div class="flex flex-col gap-1">
          <Label for="language">{t('settings.language.label')}</Label>
          <p class="text-sm text-muted-foreground">{t('settings.language.description')}</p>
        </div>
        <Select.Root type="single" value={settings.language} onValueChange={onLanguageChange}>
          <Select.Trigger id="language" class="w-44">{selectedLanguageLabel}</Select.Trigger>
          <Select.Content>
            {#each languageOptions as option (option.value)}
              <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <Separator />
      <div class="flex items-center justify-between gap-4">
        <div class="flex flex-col gap-1">
          <Label for="autostart">{t('settings.autostart.label')}</Label>
          <p class="text-sm text-muted-foreground">{t('settings.autostart.description')}</p>
        </div>
        <Switch
          id="autostart"
          bind:checked={autostartEnabled}
          onCheckedChange={onAutostartChange}
        />
      </div>
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>{t('settings.https.title')}</Card.Title>
      <Card.Description>{t('settings.https.description')}</Card.Description>
    </Card.Header>
    <Card.Content class="flex flex-col gap-3">
      {#if status?.secure.state === 'failed'}
        <p class="text-sm text-destructive">
          {t('settings.server.secureFailed', { message: status.secure.message })}
        </p>
      {:else if status?.certificate}
        <div class="flex items-center justify-between gap-4">
          <p
            class={[
              'text-sm',
              status.certificate.trusted ? 'text-muted-foreground' : 'text-destructive',
            ]}
          >
            {waitingForSystem
              ? t('settings.https.waiting')
              : status.certificate.trusted
                ? t('settings.https.trusted', { date: formatDate(status.certificate.expiresAt) })
                : t('settings.https.untrusted')}
          </p>
          {#if status.certificate.trusted}
            <Button
              variant="outline"
              size="sm"
              disabled={waitingForSystem}
              onclick={() => void changeCertificate(false)}>{t('settings.https.remove')}</Button
            >
          {:else}
            <Button
              size="sm"
              disabled={waitingForSystem}
              onclick={() => void changeCertificate(true)}>{t('settings.https.install')}</Button
            >
          {/if}
        </div>
      {/if}
      {#if onLinux}
        <p class="text-xs text-muted-foreground">{t('settings.https.firefox')}</p>
      {/if}
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>{t('settings.ports.title')}</Card.Title>
      <Card.Description>{t('settings.ports.description')}</Card.Description>
    </Card.Header>
    <Card.Content class="flex flex-col gap-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-2">
          <Label for="insecure-port">{t('settings.ports.insecure')}</Label>
          <Input
            id="insecure-port"
            inputmode="numeric"
            maxlength={5}
            placeholder={t('settings.ports.automatic')}
            bind:value={insecurePort}
          />
        </div>
        <div class="flex flex-col gap-2">
          <Label for="secure-port">{t('settings.ports.secure')}</Label>
          <Input
            id="secure-port"
            inputmode="numeric"
            maxlength={5}
            placeholder={t('settings.ports.automatic')}
            bind:value={securePort}
          />
        </div>
      </div>
      {#if portError}
        <p role="alert" class="text-xs text-destructive">{portError}</p>
      {/if}
      <div class="flex items-center justify-end gap-2">
        {#if portsSaved}
          <p class="mr-auto text-xs text-muted-foreground">{t('settings.ports.restartNeeded')}</p>
          <Button
            size="sm"
            onclick={() => {
              restart().catch((e: unknown) => {
                onerror(describeError(e));
              });
            }}>{t('settings.ports.restart')}</Button
          >
        {/if}
        <Button
          variant="outline"
          size="sm"
          disabled={!portsChanged}
          onclick={() => void savePorts()}>{t('settings.ports.save')}</Button
        >
      </div>
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>{t('settings.about.title')}</Card.Title>
      <Card.Description class="select-text"
        >{t('settings.about.version', { version })}</Card.Description
      >
    </Card.Header>
    <Card.Content class="flex flex-col gap-4">
      <p class="text-sm text-muted-foreground">
        {t('settings.about.license')}
        <button
          type="button"
          class="text-foreground underline underline-offset-4 hover:no-underline"
          onclick={() => {
            showLicenses().catch((e: unknown) => {
              onerror(t('error.load', { message: describeError(e) }));
            });
          }}>{t('settings.about.licenses')}</button
        >
      </p>
      <Separator />
      <div class="flex items-center justify-between gap-4">
        <div class="flex flex-col gap-1">
          <Label>{t('settings.about.logs')}</Label>
          <p class="text-sm text-muted-foreground">{t('settings.about.logsDescription')}</p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onclick={() => {
            openLogFolder().catch((e: unknown) => {
              onerror(t('error.load', { message: describeError(e) }));
            });
          }}>{t('settings.about.openLogs')}</Button
        >
      </div>
    </Card.Content>
  </Card.Root>
</div>
