<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    authProviders,
    completeOAuthLogin,
    loginApiKey,
    loginPassword,
    startOAuthLogin,
    type AuthProviders
  } from '$lib/api/auth';
  import { ApiError, isBackendDown } from '$lib/api/client';
  import {
    callbackError,
    isOAuthCallback,
    redirectUri,
    safeNext,
    shouldAutoLaunch
  } from '$lib/auth/oauth';
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
  let providers = $state<AuthProviders | null>(null);
  let oauthBusy = $state(false);
  let passwordFallback = $state(false);

  const showOAuth = $derived(providers?.oauth === true);
  const showPasswordForm = $derived(
    providers === null || providers.password_login !== false || passwordFallback
  );
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
      case 'bad_request':
        return 'The sign-in link expired. Start again.';
      default:
        return e.message || 'Login failed.';
    }
  }

  function describe(err: unknown, fallback: string): string {
    if (isBackendDown(err)) {
      return 'The immich-edit server is not responding. Check that it is running.';
    }
    if (err instanceof ApiError) return messageFor(err);
    return (err as Error)?.message ?? fallback;
  }

  function nextTarget(): string {
    return safeNext(page.url.searchParams.get('next'));
  }

  async function launchOAuth(): Promise<void> {
    if (oauthBusy) return;
    oauthBusy = true;
    error = null;
    try {
      const url = await startOAuthLogin(redirectUri(page.url), nextTarget());
      window.location.assign(url);
    } catch (err: unknown) {
      error = describe(err, 'Could not start single sign-on.');
      oauthBusy = false;
    }
  }

  async function completeCallback(): Promise<void> {
    const declined = callbackError(page.url);
    if (declined) {
      error = `Single sign-on was declined (${declined}).`;
      passwordFallback = true;
      return;
    }
    oauthBusy = true;
    try {
      const session = await completeOAuthLogin(page.url.href);
      window.location.replace(safeNext(session.next ?? null));
    } catch (err: unknown) {
      error = describe(err, 'Single sign-on failed.');
      passwordFallback = true;
      oauthBusy = false;
    }
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
      window.location.replace(nextTarget());
    } catch (err: unknown) {
      error = describe(err, 'Login failed');
      submitting = false;
    }
  }

  onMount(() => {
    void (async () => {
      const callback = isOAuthCallback(page.url);
      if (callback) await completeCallback();
      try {
        providers = await authProviders();
      } catch {
        providers = null;
        return;
      }
      if (!callback && shouldAutoLaunch(page.url, providers)) await launchOAuth();
    })();
  });
</script>

<div class="auth-stage flex h-full w-full items-center justify-center px-5 py-8">
  <main class="w-full max-w-xs">
    <Wordmark bright class="mb-6" />
    <div class="mb-5">
      <h1 class="text-xl font-semibold text-white">Sign in</h1>
    </div>
    <div class="flex flex-col gap-3.5">
      {#if showOAuth && providers}
        <Button
          type="button"
          size="small"
          color="primary"
          disabled={oauthBusy}
          loading={oauthBusy}
          fullWidth
          onclick={launchOAuth}
        >
          {providers.button_text}
        </Button>
        {#if showPasswordForm}
          <div class="flex items-center gap-3 text-xs text-white/35">
            <span class="h-px flex-1 bg-white/15"></span>
            or
            <span class="h-px flex-1 bg-white/15"></span>
          </div>
        {/if}
      {/if}

      {#if showPasswordForm}
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

          <Button
            type="submit"
            size="small"
            variant={showOAuth ? 'outline' : 'filled'}
            disabled={!canSubmit}
            loading={submitting}
            fullWidth
          >
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
      {:else}
        {#if error}
          <Notice message={error} />
        {/if}
        <Button
          type="button"
          variant="ghost"
          color="secondary"
          size="tiny"
          onclick={() => {
            method = 'apikey';
            passwordFallback = true;
          }}
        >
          Use an Immich API key instead
        </Button>
      {/if}
    </div>
  </main>
</div>
