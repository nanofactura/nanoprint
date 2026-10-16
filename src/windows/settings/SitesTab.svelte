<script lang="ts">
  import type { Site } from '$lib/agent';
  import SiteIcon from '$lib/components/SiteIcon.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { formatDateTime } from '$lib/format';
  import { t } from '$lib/i18n/index.svelte';

  let { sites, onrevoke }: { sites: Site[]; onrevoke: (origin: string) => void } = $props();

  function summary(site: Site): string {
    const status =
      site.decision === 'allow'
        ? t('settings.sites.allowed')
        : site.decision === 'block'
          ? t('settings.sites.blocked')
          : t('settings.sites.untrusted');
    const parts = [status, t('settings.sites.since', { date: formatDateTime(site.grantedAt) })];
    if (site.sessionOnly) parts.push(t('settings.sites.sessionOnly'));
    if (site.lastUsedAt !== null) {
      parts.push(t('settings.sites.lastUsed', { date: formatDateTime(site.lastUsedAt) }));
    }
    if (site.jobCount > 0) parts.push(t('settings.sites.jobs', { count: site.jobCount }));
    return parts.join(' · ');
  }

  function actionLabel(site: Site): string {
    if (site.decision === 'allow') return t('settings.sites.revoke');
    if (site.decision === 'block') return t('settings.sites.unblock');
    return t('settings.sites.remove');
  }
</script>

<Card.Root class="min-h-0">
  <Card.Header>
    <Card.Title>{t('settings.sites.title')}</Card.Title>
    <Card.Description>{t('settings.sites.description')}</Card.Description>
  </Card.Header>
  <Card.Content class="flex min-h-0 flex-col">
    {#if sites.length === 0}
      <p class="text-sm text-muted-foreground">{t('settings.sites.empty')}</p>
    {:else}
      <ul class="-mr-2 flex min-h-0 flex-col divide-y overflow-y-auto pr-2">
        {#each sites as site (site.origin)}
          <li class="flex items-center gap-3 py-2">
            <SiteIcon icon={site.icon} class="size-9" />
            <div class="min-w-0 flex-1">
              <!-- The origin is what was allowed; the name is only the site's own claim. -->
              <p class="truncate font-mono text-sm font-medium select-text" title={site.origin}>
                {site.origin}
              </p>
              {#if site.name}
                <p class="truncate text-xs text-muted-foreground">
                  {t('permission.presentsAs', { name: site.name })}
                </p>
              {/if}
              <p
                class={[
                  'text-xs',
                  site.decision === null ? 'text-destructive' : 'text-muted-foreground',
                ]}
              >
                {summary(site)}
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              onclick={() => {
                onrevoke(site.origin);
              }}
            >
              {actionLabel(site)}
            </Button>
          </li>
        {/each}
      </ul>
    {/if}
  </Card.Content>
</Card.Root>
