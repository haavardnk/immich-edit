<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { onMount } from 'svelte';
  import { session } from '$lib/stores/session.svelte';
  import { listUsers, setUserAccess, purgeUserData, type AdminUser } from '$lib/api/admin';
  import Notice from '$lib/components/Notice.svelte';
  import { mdiAccountCircleOutline, mdiDeleteOutline } from '@mdi/js';
  import { Badge, Button, Icon, IconButton, LoadingSpinner, Switch, Text } from '@immich/ui';

  let users = $state<AdminUser[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let busyUser = $state<string | null>(null);
  let purgeTarget = $state<AdminUser | null>(null);
  let purgeConfirm = $state('');

  async function load(): Promise<void> {
    loading = true;
    error = null;
    try {
      users = await listUsers();
    } catch (e) {
      error = (e as Error).message;
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
      error = (e as Error).message;
    } finally {
      busyUser = null;
    }
  }

  async function confirmPurge(): Promise<void> {
    if (!purgeTarget || busyUser) return;
    busyUser = purgeTarget.id;
    error = null;
    try {
      await purgeUserData(purgeTarget.id);
      purgeTarget = null;
      purgeConfirm = '';
      await load();
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busyUser = null;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<section class="space-y-3 py-2">
  {#if error}
    <Notice message={error} />
  {/if}
  {#if loading}
    <div class="inline-flex items-center gap-2 text-dark/65" aria-live="polite">
      <LoadingSpinner size="small" />
      <span class="text-xs">Loading…</span>
    </div>
  {:else if users.length === 0}
    <Text size="tiny" color="muted">No users yet.</Text>
  {:else}
    <ul class="divide-y divide-hairline">
      {#each users as u (u.id)}
        <li class="flex min-h-16 items-center gap-3 py-2.5 text-xs">
          <Icon
            icon={mdiAccountCircleOutline}
            size="24px"
            class="shrink-0 text-dark/45"
            aria-hidden="true"
          />
          <div class="min-w-0 flex-1">
            <div class="flex min-w-0 items-center gap-1.5">
              <span class="truncate">{u.name || u.email}</span>
              {#if u.is_admin}<Badge size="tiny" color="primary">admin</Badge>{/if}
              {#if !u.access_enabled}<Badge size="tiny" color="warning">disabled</Badge>{/if}
            </div>
            <div class="text-dark/65 font-mono truncate">{u.email}</div>
          </div>
          <div class="flex shrink-0 items-center gap-1">
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
              icon={mdiDeleteOutline}
              title="Purge local data"
              aria-label={`Purge local data for ${u.email}`}
              onclick={() => {
                purgeTarget = u;
                purgeConfirm = '';
              }}
              disabled={busyUser === u.id}
            />
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  {#if purgeTarget}
    <div class="space-y-2 border-l-2 border-danger-500 bg-danger-500/5 p-3 text-xs">
      <p>
        Delete all edits, presets and export jobs for
        <span class="font-mono">{purgeTarget.email}</span>? This cannot be undone.
      </p>
      <p class="text-dark/65">
        Type <span class="font-mono">{purgeTarget.email}</span> to confirm.
      </p>
      <TextInput
        size="tiny"
        class="font-mono"
        bind:value={purgeConfirm}
        placeholder={purgeTarget.email}
        aria-label="Confirm email"
      />
      <div class="flex items-center gap-2">
        <Button
          size="tiny"
          color="danger"
          onclick={() => void confirmPurge()}
          disabled={purgeConfirm !== purgeTarget.email || busyUser === purgeTarget.id}
        >
          {busyUser === purgeTarget.id ? 'Purging…' : 'Purge data'}
        </Button>
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          onclick={() => {
            purgeTarget = null;
            purgeConfirm = '';
          }}
        >
          Cancel
        </Button>
      </div>
    </div>
  {/if}
</section>
