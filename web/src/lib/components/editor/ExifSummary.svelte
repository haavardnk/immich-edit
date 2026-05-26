<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import ExifDetails from './ExifDetails.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { mdiInformationOutline } from '@mdi/js';

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
      <button
        type="button"
        class="btn btn-ghost btn-sm btn-square {ui.exifPopoverOpen ? 'text-immich-dark-primary' : ''}"
        title="Info (I)"
        aria-label="Info"
        onclick={ui.toggleExifPopover}
      >
        <Icon path={mdiInformationOutline} size={20} />
      </button>
    {/snippet}
    {#snippet children()}
      <ExifDetails />
    {/snippet}
  </Popover>
{/if}
