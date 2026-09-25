<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import {
    DEFAULT_FILENAME_TEMPLATE,
    FILENAME_TOKENS,
    renderTemplate,
    templateError,
    type NameContext
  } from '$lib/filenameTemplate';

  let {
    value = $bindable(),
    example = null,
    extension
  }: {
    value: string;
    example?: NameContext | null;
    extension: string;
  } = $props();

  const id = $props.id();
  const error = $derived(templateError(value));
  const preview = $derived(
    !error && example ? `${renderTemplate(value, example)}.${extension}` : null
  );
</script>

<TextInput
  label="Filename"
  compact
  color="neutral"
  class="ring-0 focus-within:ring-1 focus-within:ring-primary"
  bind:value
  placeholder={DEFAULT_FILENAME_TEMPLATE}
  aria-invalid={error ? 'true' : undefined}
  aria-describedby="{id}-hint"
/>
<p id="{id}-hint" class="px-1 text-[10px] leading-snug text-dark/65">
  {#if error}
    <span class="text-danger-300">{error}</span>
  {:else if preview}
    <span class="font-mono">{preview}</span>
  {/if}
  <span class="block">Tokens: {FILENAME_TOKENS.join(', ')}</span>
</p>
