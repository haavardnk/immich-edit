<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import ExifRows from '#lib/components/ExifRows.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';

  let { anchor }: { anchor: HTMLElement | null } = $props();
</script>

{#if editor.asset?.exifInfo}
  <Popover
    open={ui.metaPopover === 'exif'}
    anchor="bottom"
    align="end"
    customAnchor={anchor}
    onOpenChange={(v) => (v ? ui.openPopover('exif') : ui.closePopover())}
    contentClass="p-3"
  >
    <ExifRows asset={editor.asset} />
  </Popover>
{/if}
