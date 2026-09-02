<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { onMount } from 'svelte';
  import { session } from '$lib/stores/session.svelte';
  import { getInstance, rebindInstance, type InstanceInfo } from '$lib/api/admin';
  import { formatWhen } from '$lib/utils/datetime';
  import { mdiServerNetworkOutline, mdiSwapHorizontal } from '@mdi/js';
  import { Button, Icon, IconButton, Select } from '@immich/ui';
  import Notice from '$lib/components/Notice.svelte';

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

  const methodOptions = [
    { value: 'password', label: 'Password' },
    { value: 'apikey', label: 'API key' }
  ];

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

<section class="space-y-5 py-2">
  {#if error}
    <Notice message={error} />
  {/if}
  {#if instance}
    <div class="flex min-h-16 items-center gap-3">
      <Icon
        icon={mdiServerNetworkOutline}
        size="28px"
        class="shrink-0 text-dark/45"
        aria-hidden="true"
      />
      <div class="min-w-0 flex-1">
        <div class="truncate font-mono text-sm">{instance.immich_url}</div>
        <div class="mt-0.5 text-xs text-dark/65">
          Configured {formatWhen(instance.configured_at)} · epoch {instance.server_epoch}
        </div>
      </div>
      {#if !showRebind}
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          icon={mdiSwapHorizontal}
          title="Connect to a different Immich server"
          aria-label="Connect to a different Immich server"
          onclick={() => (showRebind = true)}
        />
      {/if}
    </div>
  {/if}
  {#if showRebind}
    <div class="max-w-xl space-y-3 border-t border-hairline pt-5 text-xs">
      <Notice
        color="warning"
        message="Rebinding points immich-edit at a new Immich server and wipes all local users, edits and export jobs. Shared LUTs and camera profiles are kept. This cannot be undone."
      />
      <TextInput
        size="tiny"
        class="font-mono"
        bind:value={url}
        placeholder="https://immich.example.com"
        aria-label="Immich server URL"
      />
      <Select
        size="tiny"
        class="w-40"
        options={methodOptions}
        value={method}
        placeholder="Authentication method"
        onChange={(value) => (method = value as 'password' | 'apikey')}
      />
      {#if method === 'password'}
        <TextInput
          size="tiny"
          type="email"
          bind:value={email}
          placeholder="admin@example.com"
          aria-label="Email"
          autocomplete="off"
        />
        <TextInput
          type="password"
          size="tiny"
          bind:value={password}
          placeholder="Password"
          aria-label="Password"
          autocomplete="off"
        />
      {:else}
        <TextInput
          size="tiny"
          class="font-mono"
          bind:value={apiKey}
          placeholder="Immich API key"
          aria-label="Immich API key"
          autocomplete="off"
        />
      {/if}
      <p class="text-dark/65">
        Type the new hostname
        {#if host}<span class="font-mono">{host}</span>{/if}
        to confirm.
      </p>
      <TextInput
        size="tiny"
        class="font-mono"
        bind:value={confirm}
        placeholder={host || 'hostname'}
        aria-label="Confirm hostname"
      />
      <div class="flex items-center gap-2">
        <Button size="tiny" color="warning" onclick={() => void submit()} disabled={busy}>
          {busy ? 'Rebinding…' : 'Rebind and reset'}
        </Button>
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          onclick={() => {
            showRebind = false;
            error = null;
          }}
        >
          Cancel
        </Button>
      </div>
    </div>
  {/if}
</section>
