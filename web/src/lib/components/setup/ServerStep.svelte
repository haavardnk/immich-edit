<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { Button, Field } from '@immich/ui';

  let {
    url = $bindable(),
    probing,
    error,
    onContinue
  }: {
    url: string;
    probing: boolean;
    error: string | null;
    onContinue: () => void;
  } = $props();

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    onContinue();
  }
</script>

<form onsubmit={submit} class="flex flex-1 flex-col gap-4">
  <div class="flex flex-col gap-1.5">
    <Field label="Immich URL" size="tiny" disabled={probing}>
      <TextInput
        type="url"
        placeholder="https://immich.example.com"
        autocomplete="url"
        bind:value={url}
      />
    </Field>
    <p class="text-xs text-dark/80">
      A Docker address such as <span class="font-mono">http://immich-server:2283</span> works too.
    </p>
  </div>
  {#if error}
    <Notice message={error} />
  {/if}
  <Button
    type="submit"
    size="small"
    color="primary"
    fullWidth
    class="touch:mt-auto"
    loading={probing}
    disabled={probing || url.trim() === ''}
  >
    Continue
  </Button>
</form>
