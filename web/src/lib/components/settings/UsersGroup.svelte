<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { onMount } from 'svelte';
  import { session } from '#lib/stores/session.svelte.ts';
  import { listUsers, setUserAccess, purgeUserData, type AdminUser } from '#lib/api/admin.ts';
  import { errorMessage } from '#lib/utils/errors.ts';
  import Dialog from '#lib/components/Dialog.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import SettingsGroup from '#lib/components/settings/SettingsGroup.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import { mdiDeleteOutline } from '@mdi/js';
  import { Badge, Button, IconButton, Switch } from '@immich/ui';

  let users = $state<AdminUser[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let busyUser = $state<string | null>(null);
  let purgeTarget = $state<AdminUser | null>(null);
  let purgeConfirm = $state('');
  let purgeError = $state<string | null>(null);

  async function load(): Promise<void> {
    loading = true;
    error = null;
    try {
      users = await listUsers();
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  async function toggleAccess(u: AdminUser): Promise<void> {
    if (busyUser) return;
    busyUser = u.id;
    error = null;
    try {
      await setUserAccess(u.id, !u.access_enabled);
      await load();
    } catch (e) {
      error = errorMessage(e);
    } finally {
      busyUser = null;
    }
  }

  function closePurge(): void {
    purgeTarget = null;
    purgeConfirm = '';
    purgeError = null;
  }

  async function confirmPurge(): Promise<void> {
    if (!purgeTarget || busyUser) return;
    busyUser = purgeTarget.id;
    purgeError = null;
    try {
      await purgeUserData(purgeTarget.id);
      closePurge();
      await load();
    } catch (e) {
      purgeError = errorMessage(e);
    } finally {
      busyUser = null;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<SettingsGroup title="Users" anchor="users">
  {#if error}
    <div class="px-4 py-3"><Notice message={error} /></div>
  {/if}
  {#if loading && users.length === 0}
    <SettingsRow label="Loading…" muted />
  {:else}
    {#each users as u (u.id)}
      <SettingsRow label={u.name || u.email} muted={!u.access_enabled}>
        {#snippet badge()}
          {#if u.is_admin}<Badge size="tiny" color="primary">admin</Badge>{/if}
        {/snippet}
        {#snippet detail()}<span class="font-mono">{u.email}</span>{/snippet}
        {#if u.id !== session.user?.id}
          <Switch
            checked={u.access_enabled}
            aria-label={`${u.access_enabled ? 'Disable' : 'Enable'} access for ${u.email}`}
            onCheckedChange={() => void toggleAccess(u)}
            disabled={busyUser === u.id}
          />
        {/if}
        <IconButton
          size="small"
          variant="ghost"
          color="danger"
          class="shrink-0 touch:size-11"
          icon={mdiDeleteOutline}
          title="Purge local data"
          aria-label={`Purge local data for ${u.email}`}
          onclick={() => {
            purgeTarget = u;
            purgeConfirm = '';
          }}
          disabled={busyUser === u.id}
        />
      </SettingsRow>
    {:else}
      <SettingsRow label="No users yet" muted />
    {/each}
  {/if}
</SettingsGroup>

{#if purgeTarget}
  {@const target = purgeTarget}
  <Dialog title="Purge local data?" size="small" onClose={closePurge}>
    <div class="flex flex-col gap-3 text-sm">
      <p>
        Delete all edits, presets and export jobs for
        <span class="font-mono">{target.email}</span>? This cannot be undone.
      </p>
      <TextInput
        bind:value={purgeConfirm}
        class="font-mono"
        placeholder={target.email}
        aria-label="Confirm email"
      />
      {#if purgeError}
        <Notice message={purgeError} />
      {/if}
      <div class="flex justify-end gap-2">
        <Button size="small" variant="ghost" color="secondary" onclick={closePurge}>Cancel</Button>
        <Button
          size="small"
          color="danger"
          loading={busyUser === target.id}
          onclick={() => void confirmPurge()}
          disabled={purgeConfirm !== target.email || busyUser === target.id}
        >
          Purge data
        </Button>
      </div>
    </div>
  </Dialog>
{/if}
