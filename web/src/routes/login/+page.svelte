<script lang="ts">
  import { page } from '$app/state';
  import { loginApiKey, loginPassword } from '$lib/api/auth';
  import { ApiError, isBackendDown } from '$lib/api/client';
  import Logo from '$lib/components/Logo.svelte';

  let method = $state<'password' | 'apikey'>('password');
  let email = $state('');
  let password = $state('');
  let apiKey = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  const canSubmit = $derived(
    !submitting && (method === 'password' ? email.length > 0 && password.length > 0 : apiKey.length > 0)
  );

  function messageFor(e: ApiError): string {
    switch (e.code) {
      case 'invalid_credentials':
      case 'unauthorized':
        return 'Invalid credentials.';
      case 'access_disabled':
        return 'Access for this account is disabled.';
      case 'missing_permissions':
        return 'The API key is missing required permissions.';
      case 'rate_limited':
        return 'Too many attempts. Wait a moment and try again.';
      case 'upstream_unavailable':
        return 'Could not reach the Immich server.';
      default:
        return e.message || 'Login failed.';
    }
  }

  function safeNext(): string {
    const next = page.url.searchParams.get('next') ?? '/';
    return next.startsWith('/') && !next.startsWith('//') ? next : '/';
  }

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    if (!canSubmit) return;
    submitting = true;
    error = null;
    try {
      if (method === 'password') {
        await loginPassword(email, password);
      } else {
        await loginApiKey(apiKey);
      }
      window.location.replace(safeNext());
    } catch (err: unknown) {
      error = isBackendDown(err)
        ? 'The immich-edit server is not responding. Check that it is running.'
        : err instanceof ApiError
          ? messageFor(err)
          : ((err as Error)?.message ?? 'Login failed');
      submitting = false;
    }
  }
</script>

<div class="h-full w-full flex items-center justify-center p-6">
  <form
    onsubmit={submit}
    class="w-full max-w-sm flex flex-col gap-4 p-6 rounded-lg bg-immich-dark-gray border border-immich-dark-gray"
  >
    <h1 class="flex items-center gap-2 text-xl font-semibold tracking-tight">
      <Logo size={26} />
      <span><span class="text-immich-dark-fg/90">immich</span><span style="color:#6366F1">-edit</span></span>
    </h1>

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
        <span class="text-sm opacity-70">Immich API key</span>
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
      {submitting ? 'Signing in…' : 'Sign in'}
    </button>

    <button
      type="button"
      onclick={() => {
        method = method === 'password' ? 'apikey' : 'password';
        error = null;
      }}
      class="text-xs opacity-70 hover:opacity-100 self-center"
    >
      {method === 'password' ? 'Use an Immich API key instead' : 'Use email and password instead'}
    </button>
  </form>
</div>
