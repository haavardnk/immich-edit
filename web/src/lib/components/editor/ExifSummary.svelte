<script lang="ts">
  import ToolbarButton from '$lib/components/ToolbarButton.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import ExifDetails from './ExifDetails.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { mdiInformationOutline } from '@mdi/js';
  import { hint } from '$lib/keybinds';

  const hasExif = $derived((editor.asset?.exifInfo ?? null) != null);
</script>

{#if hasExif}
  <Popover
    open={ui.exifPopoverOpen}
    anchor="bottom"
    align="end"
    onClose={ui.closeExifPopover}
    contentClass="p-3"
  >
    {#snippet trigger()}
      <ToolbarButton
        path={mdiInformationOutline}
        size={18}
        title={hint('Info', 'toggleInfo')}
        active={ui.exifPopoverOpen}
        onclick={ui.toggleExifPopover}
      />
    {/snippet}
    {#snippet children()}
      <ExifDetails />
    {/snippet}
  </Popover>
{/if}
