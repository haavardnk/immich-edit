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

<div class="flex flex-col gap-1.5 touch:gap-2">
  <p class="text-xs leading-snug text-dark/70 touch:text-[13px]">
    Replace original keeps the original name.
  </p>
  {#if preview}
    <FilenamePreview name={preview} />
  {/if}
</div>
