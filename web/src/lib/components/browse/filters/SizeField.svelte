<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { megabytes } from '#lib/browse/fileSize.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';

  let { key, label }: { key: 'minSize' | 'maxSize'; label: string } = $props();

  let local = $state('');
  let timer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    const value = browseControls.filters[key];
    if (timer === null && megabytes(local) !== value) local = value === null ? '' : String(value);
  });

  function onInput(e: Event): void {
    local = (e.target as HTMLInputElement).value;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      browseControls.filters[key] = megabytes(local);
      timer = null;
    }, 350);
  }
</script>

<TextInput
  size="tiny"
  color="neutral"
  type="number"
  min="0"
  step="any"
  inputmode="decimal"
  {label}
  class="ring-0 focus-within:ring-1 focus-within:ring-primary [&_input]:h-7 touch:[&_input]:h-11"
  placeholder="MB"
  value={local}
  oninput={onInput}
/>
