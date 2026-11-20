<script lang="ts">
  import type { AuthProviders } from '#lib/api/auth.ts';
  import Notice from '#lib/components/Notice.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import type { AdminCredentials } from '#lib/setup/setupSteps.ts';
  import { Button, Field } from '@immich/ui';

  type Method = 'password' | 'apikey';

  const METHODS: ReadonlyArray<{ value: Method; label: string }> = [
    { value: 'password', label: 'Email & password' },
    { value: 'apikey', label: 'API key' }
  ];

  let {
    url,
    providers,
    submitting,
    oauthBusy,
    error,
    onEdit,
    onOAuth,
    onClaim
  }: {
    url: string;
    providers: AuthProviders;
    submitting: boolean;
    oauthBusy: boolean;
    error: string | null;
    onEdit: () => void;
    onOAuth: () => void;
    onClaim: (credentials: AdminCredentials) => void;
  } = $props();

  let method = $state<Method>('password');
  let email = $state('');
  let password = $state('');
  let apiKey = $state('');

  const canSubmit = $derived(
    !submitting && (method === 'password' ? email !== '' && password !== '' : apiKey !== '')
  );

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    if (!canSubmit) return;
    onClaim(method === 'password' ? { email, password } : { api_key: apiKey });
  }
</script>

<form onsubmit={submit} class="flex flex-1 flex-col gap-4">
  <div class="flex items-center gap-2.5 rounded-lg bg-light-100 py-1.5 ps-3 pe-1.5 text-sm">
    <span aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-success"></span>
    <span class="min-w-0 flex-1 truncate font-mono text-xs text-light-950" data-truncate>
      {url}
    </span>
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="primary"
      aria-label="Edit Immich server"
      onclick={onEdit}
    >
      Edit
    </Button>
  </div>

  {#if providers.oauth}
    <Button
      type="button"
      size="small"
      color="primary"
      fullWidth
      disabled={oauthBusy || submitting}
      loading={oauthBusy}
      onclick={onOAuth}
    >
      {providers.button_text}
    </Button>
    <div class="flex items-center gap-3 text-xs text-light-950/45">
      <span class="h-px flex-1 bg-light-950/15"></span>
      or
      <span class="h-px flex-1 bg-light-950/15"></span>
    </div>
  {/if}

  <SegmentedChoice
    options={METHODS}
    value={method}
    label="Sign-in method"
    appearance="view"
    fill
    onChange={(next) => (method = next)}
  />

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

  <Notice
    color="warning"
    message="Complete setup before exposing this instance publicly. The first Immich admin to finish claims the instance."
  />

  {#if error}
    <Notice message={error} />
  {/if}

  <Button
    type="submit"
    size="small"
    color="primary"
    fullWidth
    class="touch:mt-auto"
    disabled={!canSubmit}
    loading={submitting}
  >
    {submitting ? 'Connecting…' : 'Connect and claim instance'}
  </Button>
</form>
