<script lang="ts">
  import { onMount } from 'svelte';
  import { signedInDevices } from '#lib/stores/signedInDevices.svelte.ts';
  import { formatWhen } from '#lib/utils/datetime.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { mdiLaptop, mdiLogoutVariant } from '@mdi/js';
  import { Badge, Icon, IconButton, LoadingSpinner, Text } from '@immich/ui';

  const sessions = $derived(signedInDevices.sessions);
  const loading = $derived(signedInDevices.loading);

  onMount(() => {
    void signedInDevices.load();
  });
</script>

<section class="space-y-3 py-2">
  <div class="flex min-h-8 items-center justify-between gap-3">
    <Text size="tiny" color="muted">
      {countOf(sessions.length, 'active session')}
    </Text>
    {#if sessions.length > 1}
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={mdiLogoutVariant}
        title="Revoke all other sessions"
        aria-label="Revoke all other sessions"
        onclick={() => void signedInDevices.revokeOthers()}
      />
    {/if}
  </div>
  {#if loading}
    <div class="inline-flex items-center gap-2 text-dark/65" aria-live="polite">
      <LoadingSpinner size="small" />
      <span class="text-xs">Loading…</span>
    </div>
  {:else if sessions.length === 0}
    <Text size="tiny" color="muted">No active sessions.</Text>
  {:else}
    <ul class="divide-y divide-hairline">
      {#each sessions as s (s.id)}
        <li class="flex min-h-16 items-center gap-3 py-2.5 text-xs">
          <Icon icon={mdiLaptop} size="20px" class="shrink-0 text-dark/45" aria-hidden="true" />
          <div class="min-w-0 flex-1">
            <div class="flex min-w-0 items-center gap-1.5">
              <span class="truncate">{s.user_agent || 'Unknown device'}</span>
              {#if s.current}<Badge size="tiny" color="primary">this session</Badge>{/if}
            </div>
            <div class="text-dark/65 font-mono truncate">
              {s.ip ?? '—'} · last seen {formatWhen(s.last_seen_at)}
            </div>
          </div>
          {#if !s.current}
            <IconButton
              size="small"
              variant="ghost"
              color="secondary"
              class="shrink-0"
              icon={mdiLogoutVariant}
              title="Revoke session"
              aria-label={`Revoke session for ${s.user_agent || 'unknown device'}`}
              onclick={() => void signedInDevices.revoke(s.id)}
            />
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>
