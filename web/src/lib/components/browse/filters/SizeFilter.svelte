<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { megabytes } from '$lib/browse/fileSize';
  import { browseControls } from '$lib/stores/browseControls.svelte';

  let { key, label }: { key: 'minSize' | 'maxSize'; label: string } = $props();

  let local = $state('');
  let timer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    const value = browseControls[key];
    if (timer === null && megabytes(local) !== value) local = value === null ? '' : String(value);
  });

  function onInput(e: Event): void {
    local = (e.target as HTMLInputElement).value;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      browseControls[key] = megabytes(local);
      timer = null;
    }, 350);
  }
</script>

<TextInput
  compact
  color="neutral"
  type="number"
  min="0"
  step="any"
  inputmode="decimal"
  {label}
  placeholder="MB"
  value={local}
  oninput={onInput}
/>
