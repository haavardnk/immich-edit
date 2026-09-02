<script lang="ts">
  import { page } from '$app/state';
  import { loginApiKey, loginPassword } from '$lib/api/auth';
  import { ApiError, isBackendDown } from '$lib/api/client';
  import Wordmark from '$lib/components/Wordmark.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { Button } from '@immich/ui';

  let method = $state<'password' | 'apikey'>('password');
  let email = $state('');
  let password = $state('');
  let apiKey = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  const canSubmit = $derived(
    !submitting &&
      (method === 'password' ? email.length > 0 && password.length > 0 : apiKey.length > 0)
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

<div class="auth-stage flex h-full w-full items-center justify-center px-5 py-8">
  <main class="w-full max-w-xs">
    <Wordmark bright class="mb-6" />
    <div class="mb-5">
      <h1 class="text-xl font-semibold text-white">Sign in</h1>
    </div>
    <form onsubmit={submit} class="flex flex-col gap-3.5">
      {#if method === 'password'}
        <TextInput
          label="Immich email"
          type="email"
          autocomplete="username"
          disabled={submitting}
          bind:value={email}
        />
        <TextInput
          label="Immich password"
          type="password"
          autocomplete="current-password"
          disabled={submitting}
          bind:value={password}
        />
      {:else}
        <TextInput
          label="Immich API key"
          type="password"
          autocomplete="off"
          disabled={submitting}
          bind:value={apiKey}
        />
      {/if}

      {#if error}
        <Notice message={error} />
      {/if}

      <Button type="submit" size="small" disabled={!canSubmit} loading={submitting} fullWidth>
        {submitting ? 'Signing in…' : 'Sign in'}
      </Button>

      <Button
        type="button"
        variant="ghost"
        color="secondary"
        size="tiny"
        onclick={() => {
          method = method === 'password' ? 'apikey' : 'password';
          error = null;
        }}
      >
        {method === 'password'
          ? 'Use an Immich API key instead'
          : 'Use Immich email and password instead'}
      </Button>
    </form>
  </main>
</div>
