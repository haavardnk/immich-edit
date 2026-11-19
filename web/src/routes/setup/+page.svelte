<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import {
    completeOAuthSetup,
    completeSetup,
    setupProviders,
    startOAuthSetup,
    type AuthProviders
  } from '#lib/api/auth.ts';
  import { callbackError, isOAuthCallback, redirectUri } from '#lib/auth/oauth.ts';
  import Wordmark from '#lib/components/Wordmark.svelte';
  import AdminStep from '#lib/components/setup/AdminStep.svelte';
  import ServerStep from '#lib/components/setup/ServerStep.svelte';
  import SetupSteps from '#lib/components/setup/SetupSteps.svelte';
  import { claimError, IMMICH_SILENT, probeError } from '#lib/setup/setupErrors.ts';
  import type { AdminCredentials } from '#lib/setup/setupSteps.ts';

  let immichUrl = $state('');
  let providers = $state<AuthProviders | null>(null);
  let probing = $state(false);
  let submitting = $state(false);
  let oauthBusy = $state(false);
  let error = $state<string | null>(null);

  const trimmedUrl = $derived(immichUrl.trim());
  const step = $derived(providers ? 'admin' : 'server');

  async function probe(): Promise<void> {
    if (probing || trimmedUrl === '') return;
    probing = true;
    error = null;
    try {
      const found = await setupProviders(trimmedUrl);
      if (found.degraded) error = IMMICH_SILENT;
      else providers = found;
    } catch (err: unknown) {
      error = probeError(err);
    } finally {
      probing = false;
    }
  }

  function editServer(): void {
    providers = null;
    error = null;
  }

  async function launchOAuth(): Promise<void> {
    if (oauthBusy) return;
    oauthBusy = true;
    error = null;
    try {
      const url = await startOAuthSetup(trimmedUrl, redirectUri(page.url));
      window.location.assign(url);
    } catch (err: unknown) {
      error = claimError(err, 'Could not start single sign-on.');
      oauthBusy = false;
    }
  }

  async function completeCallback(): Promise<void> {
    const declined = callbackError(page.url);
    if (declined) {
      error = `Single sign-on was declined (${declined}).`;
      return;
    }
    oauthBusy = true;
    try {
      await completeOAuthSetup(page.url.href);
      window.location.replace('/');
    } catch (err: unknown) {
      error = claimError(err, 'Single sign-on failed.');
      oauthBusy = false;
    }
  }

  async function claim(credentials: AdminCredentials): Promise<void> {
    if (submitting) return;
    submitting = true;
    error = null;
    try {
      await completeSetup({ immich_url: trimmedUrl, ...credentials });
      window.location.replace('/');
    } catch (err: unknown) {
      error = claimError(err, 'Setup failed');
      submitting = false;
    }
  }

  onMount(() => {
    if (isOAuthCallback(page.url)) void completeCallback();
  });
</script>

<div
  class="auth-stage flex h-full w-full flex-col overflow-y-auto px-6 py-10 touch:px-5 touch:pt-4 touch:pb-6"
>
  <main class="mx-auto my-auto flex w-full max-w-sm flex-col gap-5 touch:my-0 touch:flex-1">
    <Wordmark bright class="touch:hidden" />
    <h1 class="text-xl font-semibold text-light-950 touch:text-title">Set up immich-edit</h1>
    <SetupSteps current={step} />
    {#if providers}
      <AdminStep
        url={trimmedUrl}
        {providers}
        {submitting}
        {oauthBusy}
        {error}
        onEdit={editServer}
        onOAuth={() => void launchOAuth()}
        onClaim={(credentials) => void claim(credentials)}
      />
    {:else}
      <ServerStep bind:url={immichUrl} {probing} {error} onContinue={() => void probe()} />
    {/if}
  </main>
</div>
