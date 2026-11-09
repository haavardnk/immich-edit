<script lang="ts">
  import SearchField from '#lib/components/SearchField.svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';

  let local = $state(browseControls.filters.filename);
  let timer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (browseControls.filters.filename !== local && timer === null) {
      local = browseControls.filters.filename;
    }
  });

  function oninput(value: string): void {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      browseControls.filters.filename = value;
      timer = null;
    }, 350);
  }
</script>

<SearchField bind:value={local} label="Name contains" {oninput} />
