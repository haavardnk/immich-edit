<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { Field, Select } from '@immich/ui';
  import { SHARPEN_PPI, isPrintMedia, sharpenError } from './sharpen';
  import { SHARPEN_AMOUNTS, SHARPEN_MEDIA, formSharpen, type ExportForm } from './settings';

  let { form = $bindable<ExportForm>() }: { form: ExportForm } = $props();

  const error = $derived(sharpenError(formSharpen(form)));
</script>

<Field label="Sharpen" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={SHARPEN_MEDIA}
    value={form.sharpenMedia}
    onChange={(media) => (form.sharpenMedia = media)}
  />
</Field>

{#if form.sharpenMedia !== 'none'}
  <Field label="Amount" size="tiny">
    <Select
      size="tiny"
      class="editor-compact-select editor-compact-field"
      options={SHARPEN_AMOUNTS}
      value={form.sharpenAmount}
      onChange={(amount) => (form.sharpenAmount = amount)}
    />
  </Field>
  {#if isPrintMedia(form.sharpenMedia)}
    <TextInput
      label="Print PPI"
      type="number"
      compact
      color="neutral"
      class="ring-0 focus-within:ring-1 focus-within:ring-primary"
      min={SHARPEN_PPI.min}
      max={SHARPEN_PPI.max}
      step={1}
      value={String(form.sharpenPpi)}
      aria-invalid={error ? 'true' : undefined}
      oninput={(event: Event) => {
        form.sharpenPpi = (event.currentTarget as HTMLInputElement).valueAsNumber;
      }}
    />
    {#if error}
      <p class="px-1 text-[10px] leading-snug text-danger-300">{error}</p>
    {/if}
  {/if}
{/if}
