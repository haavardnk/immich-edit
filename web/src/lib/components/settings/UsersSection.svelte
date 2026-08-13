<script lang="ts">
  import { onMount } from 'svelte';
  import { session } from '$lib/stores/session.svelte';
  import { listUsers, setUserAccess, purgeUserData, type AdminUser } from '$lib/api/admin';
  import Spinner from '$lib/components/Spinner.svelte';

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

<section class="space-y-2">
  <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Users</h2>
  {#if error}
    <p class="text-xs text-red-400">{error}</p>
  {/if}
  {#if loading}
    <Spinner label="Loading…" />
  {:else if users.length === 0}
    <p class="text-xs text-immich-dark-fg/50">No users yet.</p>
  {:else}
    <ul class="space-y-1">
      {#each users as u (u.id)}
        <li class="flex items-center justify-between rounded bg-white/5 px-3 py-2 text-xs">
          <div class="min-w-0">
            <div class="truncate">
              {u.name || u.email}
              {#if u.is_admin}<span class="text-immich-primary"> · admin</span>{/if}
              {#if !u.access_enabled}<span class="text-amber-300"> · disabled</span>{/if}
            </div>
            <div class="text-immich-dark-fg/40 font-mono truncate">{u.email}</div>
          </div>
          <div class="flex items-center gap-2 shrink-0 ml-2">
            {#if u.id !== session.user?.id}
              <button
                class="px-2 py-1 rounded bg-white/5 hover:bg-white/10 disabled:opacity-50"
                onclick={() => void toggleAccess(u)}
                disabled={busyUser === u.id}
              >
                {u.access_enabled ? 'Disable' : 'Enable'}
              </button>
            {/if}
            <button
              class="px-2 py-1 rounded bg-red-500/10 text-red-300 hover:bg-red-500/20 disabled:opacity-50"
              onclick={() => {
                purgeTarget = u;
                purgeConfirm = '';
              }}
              disabled={busyUser === u.id}
            >
              Purge data
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  {#if purgeTarget}
    <div class="rounded border border-red-500/30 bg-red-500/5 p-3 space-y-2 text-xs">
      <p>
        Delete all edits, presets and export jobs for
        <span class="font-mono">{purgeTarget.email}</span>? This cannot be undone.
      </p>
      <p class="text-immich-dark-fg/50">
        Type <span class="font-mono">{purgeTarget.email}</span> to confirm.
      </p>
      <input
        class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
        bind:value={purgeConfirm}
        placeholder={purgeTarget.email}
      />
      <div class="flex items-center gap-2">
        <button
          class="px-3 py-1.5 rounded bg-red-500/20 text-red-200 hover:bg-red-500/30 disabled:opacity-50"
          onclick={() => void confirmPurge()}
          disabled={purgeConfirm !== purgeTarget.email || busyUser === purgeTarget.id}
        >
          {busyUser === purgeTarget.id ? 'Purging…' : 'Purge data'}
        </button>
        <button
          class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10"
          onclick={() => {
            purgeTarget = null;
            purgeConfirm = '';
          }}
        >
          Cancel
        </button>
      </div>
    </div>
  {/if}
</section>
