<script lang="ts">
  import { onMount } from 'svelte';
  import { session } from '$lib/stores/session.svelte';
  import { getInstance, rebindInstance, type InstanceInfo } from '$lib/api/admin';
  import { formatWhen } from '$lib/utils/datetime';

  let instance = $state<InstanceInfo | null>(null);
  let showRebind = $state(false);
  let url = $state('');
  let method = $state<'password' | 'apikey'>('password');
  let email = $state('');
  let password = $state('');
  let apiKey = $state('');
  let confirm = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);

  const host = $derived(hostnameOf(url));

  function hostnameOf(value: string): string {
    try {
      return new URL(value.trim()).hostname;
    } catch {
      return '';
    }
  }

  async function load(): Promise<void> {
    try {
      instance = await getInstance();
    } catch (e) {
      error = (e as Error).message;
    }
  }

  async function submit(): Promise<void> {
    if (busy) return;
    error = null;
    if (!host) {
      error = 'Enter a valid Immich URL.';
      return;
    }
    if (confirm.trim().toLowerCase() !== host.toLowerCase()) {
      error = 'Type the new hostname to confirm.';
      return;
    }
    busy = true;
    try {
      await rebindInstance({
        immich_url: url.trim(),
        confirm_hostname: confirm.trim(),
        email: method === 'password' ? email.trim() : undefined,
        password: method === 'password' ? password : undefined,
        api_key: method === 'apikey' ? apiKey.trim() : undefined
      });
      session.clear();
      window.location.replace('/login');
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<section class="space-y-2">
  <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Immich instance</h2>
  {#if error}
    <p class="text-xs text-red-400">{error}</p>
  {/if}
  {#if instance}
    <dl class="grid grid-cols-[160px_1fr] gap-y-1 text-xs">
      <dt class="text-immich-dark-fg/50">Server URL</dt>
      <dd class="font-mono break-all">{instance.immich_url}</dd>
      <dt class="text-immich-dark-fg/50">Config epoch</dt>
      <dd class="font-mono">{instance.server_epoch}</dd>
      <dt class="text-immich-dark-fg/50">Configured</dt>
      <dd class="font-mono">{formatWhen(instance.configured_at)}</dd>
    </dl>
  {/if}
  {#if !showRebind}
    <button
      class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs"
      onclick={() => (showRebind = true)}
    >
      Rebind to a different Immich server…
    </button>
  {:else}
    <div class="rounded border border-amber-500/30 bg-amber-500/5 p-3 space-y-2 text-xs">
      <p class="text-amber-200">
        Rebinding points immich-edit at a new Immich server and wipes all local users, edits and
        export jobs. Shared LUTs and camera profiles are kept. This cannot be undone.
      </p>
      <input
        class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
        bind:value={url}
        placeholder="https://immich.example.com"
      />
      <div class="flex items-center gap-2">
        <button
          class="px-2 py-1 rounded {method === 'password'
            ? 'bg-white/15'
            : 'bg-white/5 hover:bg-white/10'}"
          onclick={() => (method = 'password')}
        >
          Password
        </button>
        <button
          class="px-2 py-1 rounded {method === 'apikey'
            ? 'bg-white/15'
            : 'bg-white/5 hover:bg-white/10'}"
          onclick={() => (method = 'apikey')}
        >
          API key
        </button>
      </div>
      {#if method === 'password'}
        <input
          class="w-full rounded bg-black/30 border border-white/10 px-2 py-1"
          bind:value={email}
          placeholder="admin@example.com"
          autocomplete="off"
        />
        <input
          class="w-full rounded bg-black/30 border border-white/10 px-2 py-1"
          type="password"
          bind:value={password}
          placeholder="Password"
          autocomplete="off"
        />
      {:else}
        <input
          class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
          bind:value={apiKey}
          placeholder="Immich API key"
          autocomplete="off"
        />
      {/if}
      <p class="text-immich-dark-fg/50">
        Type the new hostname
        {#if host}<span class="font-mono">{host}</span>{/if}
        to confirm.
      </p>
      <input
        class="w-full rounded bg-black/30 border border-white/10 px-2 py-1 font-mono"
        bind:value={confirm}
        placeholder={host || 'hostname'}
      />
      <div class="flex items-center gap-2">
        <button
          class="px-3 py-1.5 rounded bg-amber-500/20 text-amber-100 hover:bg-amber-500/30 disabled:opacity-50"
          onclick={() => void submit()}
          disabled={busy}
        >
          {busy ? 'Rebinding…' : 'Rebind and reset'}
        </button>
        <button
          class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10"
          onclick={() => {
            showRebind = false;
            error = null;
          }}
        >
          Cancel
        </button>
      </div>
    </div>
  {/if}
</section>
