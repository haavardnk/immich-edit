<script lang="ts">
  import {
    ORIGINAL_FILENAME_TEMPLATE,
    renderTemplate,
    type NameContext
  } from '#lib/editor/filenameTemplate.ts';
  import FilenamePreview from './FilenamePreview.svelte';

  let { example = null, extension }: { example?: NameContext | null; extension: string } = $props();

  const preview = $derived(
    example ? `${renderTemplate(ORIGINAL_FILENAME_TEMPLATE, example)}.${extension}` : null
  );
</script>

{#if preview}
  <FilenamePreview name={preview} />
{/if}
