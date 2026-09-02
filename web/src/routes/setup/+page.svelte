<script lang="ts">
  import { completeSetup } from '$lib/api/auth';
  import { ApiError, isBackendDown } from '$lib/api/client';
  import Wordmark from '$lib/components/Wordmark.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { Alert, Button, Field } from '@immich/ui';

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
      error = isBackendDown(err)
        ? 'The immich-edit server is not responding. Check that it is running.'
        : err instanceof ApiError
          ? messageFor(err)
          : ((err as Error)?.message ?? 'Setup failed');
      submitting = false;
    }
  }
</script>

<div class="auth-stage h-full w-full overflow-y-auto px-6 py-10">
  <main class="mx-auto flex min-h-full w-full max-w-md flex-col justify-center">
    <Wordmark size="large" bright class="mb-8" />
    <div class="mb-6">
      <h1 class="text-2xl font-semibold text-white">Connect Immich</h1>
      <p class="mt-1 text-sm text-white/45">
        Sign in as an Immich administrator to claim this editor.
      </p>
    </div>
    <div class="flex flex-col gap-4">
      <Alert color="warning" size="small" shape="rectangle" icon={false}>
        Complete setup before exposing this instance publicly. The first Immich admin to finish
        claims the instance.
      </Alert>
      <form onsubmit={submit} class="flex flex-col gap-4">
        <Field label="Immich URL" size="tiny" disabled={submitting}>
          <TextInput
            type="url"
            placeholder="https://immich.example.com"
            autocomplete="url"
            bind:value={immichUrl}
          />
        </Field>

        <div class="flex gap-1 rounded bg-light-100 p-1 text-sm">
          <Button
            type="button"
            size="tiny"
            variant={method === 'password' ? 'filled' : 'ghost'}
            color={method === 'password' ? 'primary' : 'secondary'}
            class="flex-1"
            aria-pressed={method === 'password'}
            onclick={() => (method = 'password')}
          >
            Email &amp; password
          </Button>
          <Button
            type="button"
            size="tiny"
            variant={method === 'apikey' ? 'filled' : 'ghost'}
            color={method === 'apikey' ? 'primary' : 'secondary'}
            class="flex-1"
            aria-pressed={method === 'apikey'}
            onclick={() => (method = 'apikey')}
          >
            API key
          </Button>
        </div>

        {#if method === 'password'}
          <Field label="Email" size="tiny" disabled={submitting}>
            <TextInput type="email" autocomplete="username" bind:value={email} />
          </Field>
          <Field label="Password" size="tiny" disabled={submitting}>
            <TextInput type="password" autocomplete="current-password" bind:value={password} />
          </Field>
        {:else}
          <Field label="API key" size="tiny" disabled={submitting}>
            <TextInput type="password" autocomplete="off" bind:value={apiKey} />
          </Field>
        {/if}

        {#if error}
          <Notice message={error} />
        {/if}

        <Button
          type="submit"
          size="small"
          color="primary"
          disabled={!canSubmit}
          loading={submitting}
          fullWidth
        >
          {submitting ? 'Connecting…' : 'Connect and claim instance'}
        </Button>
      </form>
    </div>
  </main>
</div>
