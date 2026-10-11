<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';

  let filenameLocal = $state(browseControls.filename);
  let filenameTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (browseControls.filename !== filenameLocal && filenameTimer === null) {
      filenameLocal = browseControls.filename;
    }
  });

  function onFilenameInput(e: Event): void {
    filenameLocal = (e.target as HTMLInputElement).value;
    if (filenameTimer) clearTimeout(filenameTimer);
    filenameTimer = setTimeout(() => {
      browseControls.filename = filenameLocal;
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
