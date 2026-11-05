<script lang="ts">
  import SortControls from '#lib/components/browse/SortControls.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { GRID_SIZE_OPTIONS, TILE_INFO_OPTIONS } from '#lib/browse/viewOptions.ts';
  import { resizeGrid } from '#lib/browse/gridResize.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { Button, Label } from '@immich/ui';
  import { mdiRestore } from '@mdi/js';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();
</script>

{#if sortBasis}
  <SortControls basis={sortBasis} />
{/if}
<div class="flex flex-col gap-1">
  <Label label="Thumbnail size" size="small" />
  <SegmentedChoice
    options={GRID_SIZE_OPTIONS}
    value={browseView.gridSize}
    label="Thumbnail size"
    onChange={(size) => resizeGrid(size)}
  />
</div>
<div class="flex flex-col gap-1">
  <Label label="Thumbnail info" size="small" />
  <SegmentedChoice
    options={TILE_INFO_OPTIONS}
    value={browseView.tileInfo}
    label="Thumbnail info"
    onChange={(info) => browseView.setTileInfo(info)}
  />
</div>
{#if !browseControls.isDefault}
  <Button
    size="small"
    variant="ghost"
    color="secondary"
    class="min-h-11 self-start"
    leadingIcon={mdiRestore}
    onclick={() => browseControls.reset()}
  >
    Reset filters and sort
  </Button>
{/if}
