<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import ExifDetails from './ExifDetails.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { mdiInformationOutline } from '@mdi/js';
  import { hint } from '$lib/keybinds';
  import { IconButton } from '@immich/ui';

  const hasExif = $derived((editor.asset?.exifInfo ?? null) != null);
</script>

{#if hasExif}
  <Popover
    open={ui.metaPopover === 'exif'}
    anchor="bottom"
    align="end"
    onOpenChange={(v) => (v ? ui.openPopover('exif') : ui.closePopover())}
    contentClass="p-3"
  >
    {#snippet trigger(props)}
      <IconButton
        size="small"
        variant="ghost"
        color={ui.metaPopover === 'exif' ? 'primary' : 'secondary'}
        icon={mdiInformationOutline}
        title={hint('Info', 'toggleInfo')}
        aria-label={hint('Info', 'toggleInfo')}
        {...props}
      />
    {/snippet}
    <ExifDetails />
  </Popover>
{/if}
