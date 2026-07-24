<script lang="ts">
  import { completeSetup } from '$lib/api/auth';
  import { ApiError } from '$lib/api/client';
  import Logo from '$lib/components/Logo.svelte';

  let immichUrl = $state('');
  let method = $state<'password' | 'apikey'>('password');
  let email = $state('');
  let password = $state('');
  let apiKey = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  const canSubmit = $derived(
    !submitting &&
      immichUrl.trim().length > 0 &&
      (method === 'password' ? email.length > 0 && password.length > 0 : apiKey.length > 0)
  );

  function messageFor(e: ApiError): string {
    switch (e.code) {
      case 'admin_required':
        return 'That account is not an Immich administrator. Setup requires an admin.';
      case 'invalid_credentials':
        return 'Invalid credentials.';
      case 'missing_permissions':
        return 'The API key is missing required permissions.';
      case 'rate_limited':
        return 'Too many attempts. Wait a moment and try again.';
      case 'upstream_unavailable':
        return 'Could not reach the Immich server. Check the URL.';
      case 'conflict':
        return 'This instance is already configured.';
      default:
        return e.message || 'Setup failed.';
    }
  }

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    if (!canSubmit) return;
    submitting = true;
    error = null;
    try {
      const body =
        method === 'password'
          ? { immich_url: immichUrl.trim(), email, password }
          : { immich_url: immichUrl.trim(), api_key: apiKey };
      await completeSetup(body);
      window.location.replace('/');
    } catch (err: unknown) {
      error = err instanceof ApiError ? messageFor(err) : ((err as Error)?.message ?? 'Setup failed');
      submitting = false;
    }
  }
</script>

<div class="h-full w-full flex items-center justify-center p-6">
  <form
    onsubmit={submit}
    class="w-full max-w-md flex flex-col gap-4 p-6 rounded-lg bg-immich-dark-gray border border-immich-dark-gray"
  >
    <h1 class="flex items-center gap-2 text-xl font-semibold tracking-tight">
      <Logo size={26} />
      <span
        ><span class="text-immich-dark-fg/90">immich</span><span style="color:#6366F1">-edit</span
        ></span
      >
    </h1>
    <p class="text-sm opacity-70">
      Connect this instance to your Immich server. Sign in as an Immich administrator to claim it.
    </p>
    <div class="rounded border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-200">
      Complete setup before exposing this instance publicly. The first Immich admin to finish claims
      the instance.
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm opacity-70">Immich URL</span>
      <input
        type="url"
        placeholder="https://immich.example.com"
        autocomplete="url"
        bind:value={immichUrl}
        disabled={submitting}
        class="px-3 py-2 rounded bg-black/30 border border-white/10 focus:outline-none focus:ring-1 focus:ring-immich-primary"
      />
    </label>

    <div class="flex gap-1 rounded bg-black/30 border border-white/10 p-1 text-sm">
      <button
        type="button"
        onclick={() => (method = 'password')}
        class="flex-1 rounded px-2 py-1 {method === 'password' ? 'bg-immich-primary text-white' : 'opacity-70'}"
      >
        Email &amp; password
      </button>
      <button
        type="button"
        onclick={() => (method = 'apikey')}
        class="flex-1 rounded px-2 py-1 {method === 'apikey' ? 'bg-immich-primary text-white' : 'opacity-70'}"
      >
        API key
      </button>
    </div>

    {#if method === 'password'}
      <label class="flex flex-col gap-1">
        <span class="text-sm opacity-70">Email</span>
        <input
          type="email"
          autocomplete="username"
          bind:value={email}
          disabled={submitting}
          class="px-3 py-2 rounded bg-black/30 border border-white/10 focus:outline-none focus:ring-1 focus:ring-immich-primary"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm opacity-70">Password</span>
        <input
          type="password"
          autocomplete="current-password"
          bind:value={password}
          disabled={submitting}
          class="px-3 py-2 rounded bg-black/30 border border-white/10 focus:outline-none focus:ring-1 focus:ring-immich-primary"
        />
      </label>
    {:else}
      <label class="flex flex-col gap-1">
        <span class="text-sm opacity-70">API key</span>
        <input
          type="password"
          autocomplete="off"
          bind:value={apiKey}
          disabled={submitting}
          class="px-3 py-2 rounded bg-black/30 border border-white/10 focus:outline-none focus:ring-1 focus:ring-immich-primary"
        />
      </label>
    {/if}

    {#if error}
      <p class="text-sm text-red-400">{error}</p>
    {/if}

    <button
      type="submit"
      disabled={!canSubmit}
      class="px-3 py-2 rounded bg-immich-primary text-white disabled:opacity-50"
    >
      {submitting ? 'Connecting…' : 'Connect and claim instance'}
    </button>
  </form>
</div>
