<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listSessions,
    revokeSession,
    revokeAllSessions,
    type SessionInfo
  } from '$lib/api/account';
  import { formatWhen } from '$lib/utils/datetime';
  import Spinner from '$lib/components/Spinner.svelte';

  let sessions = $state<SessionInfo[]>([]);
  let loading = $state(false);

  async function load(): Promise<void> {
    loading = true;
    try {
      sessions = await listSessions();
    } catch {
      sessions = [];
    } finally {
      loading = false;
    }
  }

  async function revokeOne(id: string): Promise<void> {
    await revokeSession(id);
    await load();
  }

  async function revokeOthers(): Promise<void> {
    await revokeAllSessions();
    await load();
  }

  onMount(() => {
    void load();
  });
</script>

<section class="space-y-2">
  <div class="flex items-center justify-between">
    <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Active sessions</h2>
    {#if sessions.length > 1}
      <button
        class="px-2 py-1 rounded bg-white/5 hover:bg-white/10 text-xs"
        onclick={() => void revokeOthers()}
      >
        Revoke all other sessions
      </button>
    {/if}
  </div>
  {#if loading}
    <Spinner label="Loading…" />
  {:else if sessions.length === 0}
    <p class="text-xs text-immich-dark-fg/50">No active sessions.</p>
  {:else}
    <ul class="space-y-1">
      {#each sessions as s (s.id)}
        <li class="flex items-center justify-between rounded bg-white/5 px-3 py-2 text-xs">
          <div class="min-w-0 flex-1">
            <div class="truncate">
              {s.user_agent || 'Unknown device'}
              {#if s.current}<span class="text-immich-primary"> · this session</span>{/if}
            </div>
            <div class="text-immich-dark-fg/40 font-mono truncate">
              {s.ip ?? '—'} · last seen {formatWhen(s.last_seen_at)}
            </div>
          </div>
          {#if !s.current}
            <button
              class="ml-2 px-2 py-1 rounded bg-white/5 hover:bg-white/10 shrink-0"
              onclick={() => void revokeOne(s.id)}
            >
              Revoke
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>
