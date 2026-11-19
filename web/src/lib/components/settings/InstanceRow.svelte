<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { onMount } from 'svelte';
  import { session } from '#lib/stores/session.svelte.ts';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import { getInstance, rebindInstance, type InstanceInfo } from '#lib/api/admin.ts';
  import { errorMessage } from '#lib/utils/errors.ts';
  import { Button, Select } from '@immich/ui';
  import Dialog from '#lib/components/Dialog.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';

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
  let loadError = $state<string | null>(null);

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
      loadError = null;
    } catch (e) {
      loadError = errorMessage(e);
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
      appearance.reset();
      window.location.replace('/login');
    } catch (e) {
      error = errorMessage(e);
    } finally {
      busy = false;
    }
  }

  function close(): void {
    showRebind = false;
    error = null;
  }

  onMount(() => {
    void load();
  });
</script>

<SettingsRow label="Immich instance" anchor="instance">
  {#snippet detail()}
    {#if loadError}
      <span class="text-danger-800">{loadError}</span>
    {:else if instance}
      <span
        class="me-1.5 inline-block size-1.5 rounded-full bg-success align-middle"
        aria-hidden="true"
      ></span><span class="font-mono">{instance.immich_url}</span>
    {/if}
  {/snippet}
  <Button
    size="small"
    variant="ghost"
    color="secondary"
    class="shrink-0 bg-light-200/60 touch:min-h-11"
    onclick={() => (showRebind = true)}
  >
    Change server…
  </Button>
</SettingsRow>

{#if showRebind}
  <Dialog title="Change Immich server" size="medium" onClose={close}>
    <div class="flex flex-col gap-3 text-xs">
      {#if error}
        <Notice message={error} />
      {/if}
      <Notice
        color="warning"
        message="Rebinding points immich-edit at a new Immich server and wipes all local users, edits and export jobs. Shared LUTs and camera profiles are kept. This cannot be undone."
      />
      <TextInput
        class="font-mono"
        bind:value={url}
        placeholder="https://immich.example.com"
        aria-label="Immich server URL"
      />
      <Select
        size="small"
        class="w-40"
        options={methodOptions}
        value={method}
        placeholder="Authentication method"
        onChange={(value) => (method = value as 'password' | 'apikey')}
      />
      {#if method === 'password'}
        <TextInput
          type="email"
          bind:value={email}
          placeholder="admin@example.com"
          aria-label="Email"
          autocomplete="off"
        />
        <TextInput
          type="password"
          bind:value={password}
          placeholder="Password"
          aria-label="Password"
          autocomplete="off"
        />
      {:else}
        <TextInput
          class="font-mono"
          bind:value={apiKey}
          placeholder="Immich API key"
          aria-label="Immich API key"
          autocomplete="off"
        />
      {/if}
      <p class="text-dark/75">
        Type the new hostname
        {#if host}<span class="font-mono">{host}</span>{/if}
        to confirm.
      </p>
      <TextInput
        class="font-mono"
        bind:value={confirm}
        placeholder={host || 'hostname'}
        aria-label="Confirm hostname"
      />
      <div class="flex justify-end gap-2">
        <Button size="small" variant="ghost" color="secondary" onclick={close}>Cancel</Button>
        <Button
          size="small"
          color="warning"
          loading={busy}
          onclick={() => void submit()}
          disabled={busy}
        >
          Rebind and reset
        </Button>
      </div>
    </div>
  </Dialog>
{/if}
