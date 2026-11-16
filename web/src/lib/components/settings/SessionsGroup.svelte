<script lang="ts">
  import SettingsGroup from '#lib/components/settings/SettingsGroup.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import { signedInDevices } from '#lib/stores/signedInDevices.svelte.ts';
  import { formatWhen } from '#lib/utils/datetime.ts';
  import { Badge, Button, IconButton } from '@immich/ui';
  import { mdiLaptop, mdiLogoutVariant } from '@mdi/js';
  import { onMount } from 'svelte';

  const sessions = $derived(signedInDevices.sessions);

  onMount(() => {
    void signedInDevices.load();
  });
</script>

{#snippet revokeOthers()}
  {#if sessions.length > 1}
    <Button
      size="tiny"
      variant="ghost"
      color="primary"
      onclick={() => void signedInDevices.revokeOthers()}
    >
      Revoke all other sessions
    </Button>
  {/if}
{/snippet}

<SettingsGroup title="Signed-in devices" anchor="sessions" action={revokeOthers}>
  {#if !signedInDevices.loaded}
    <SettingsRow label="Loading…" muted />
  {:else}
    {#each sessions as session (session.id)}
      {@const name = session.user_agent || 'Unknown device'}
      <SettingsRow label={name} icon={mdiLaptop}>
        {#snippet badge()}
          {#if session.current}<Badge size="tiny" color="primary">this session</Badge>{/if}
        {/snippet}
        {#snippet detail()}
          <span class="font-mono">{session.ip ?? '—'}</span> · last seen {formatWhen(
            session.last_seen_at
          )}
        {/snippet}
        {#if !session.current}
          <IconButton
            size="small"
            variant="ghost"
            color="secondary"
            class="shrink-0 touch:size-11"
            icon={mdiLogoutVariant}
            title="Revoke session"
            aria-label="Revoke session for {name}"
            onclick={() => void signedInDevices.revoke(session.id)}
          />
        {/if}
      </SettingsRow>
    {/each}
  {/if}
</SettingsGroup>
