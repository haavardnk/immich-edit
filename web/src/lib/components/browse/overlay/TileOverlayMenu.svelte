<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import TileOverlayControls from '#lib/components/browse/overlay/TileOverlayControls.svelte';
  import TileOverlayReset from '#lib/components/browse/overlay/TileOverlayReset.svelte';
  import { tileInfoLabel } from '#lib/browse/viewOptions.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiTextBoxOutline } from '@mdi/js';

  let open = $state(false);

  const label = $derived(tileInfoLabel(browseView.tileInfo));
</script>

<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  appearance="panel"
  contentClass="flex w-72 flex-col"
  title="Thumbnail info"
  tooltip={hint(label, 'gridTileInfo')}
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="small"
      variant="ghost"
      color={open || browseView.tileInfo === 'full' ? 'primary' : 'secondary'}
      icon={mdiTextBoxOutline}
      title=""
      aria-label={label}
      {...popoverProps}
    />
  {/snippet}
  <div class="flex h-11 shrink-0 items-center border-b border-hairline pr-1.5 pl-3">
    <h2 class="text-label font-semibold text-light-950/90">Thumbnail info</h2>
    <TileOverlayReset />
  </div>
  <div class="p-3">
    <TileOverlayControls />
  </div>
</Popover>
