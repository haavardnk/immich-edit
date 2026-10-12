<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';

  let filenameLocal = $state(browseControls.filters.filename);
  let filenameTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (browseControls.filters.filename !== filenameLocal && filenameTimer === null) {
      filenameLocal = browseControls.filters.filename;
    }
  });

  function onFilenameInput(e: Event): void {
    filenameLocal = (e.target as HTMLInputElement).value;
    if (filenameTimer) clearTimeout(filenameTimer);
    filenameTimer = setTimeout(() => {
      browseControls.filters.filename = filenameLocal;
      filenameTimer = null;
    }, 350);
  }
</script>

<TextInput
  compact
  color="neutral"
  label="Filename"
  placeholder="Search…"
  value={filenameLocal}
  oninput={onFilenameInput}
/>
