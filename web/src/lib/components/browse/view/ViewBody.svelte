<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import TileOverlayControls from '#lib/components/browse/overlay/TileOverlayControls.svelte';
  import TileOverlayReset from '#lib/components/browse/overlay/TileOverlayReset.svelte';
  import { resizeGrid } from '#lib/browse/gridResize.ts';
  import {
    EDIT_SORT_SHORT,
    SORT_DIRECTION_SHORT,
    SORT_FIELD_SHORT,
    sortFields
  } from '#lib/browse/sortOptions.ts';
  import { GRID_SIZE_OPTIONS } from '#lib/browse/viewOptions.ts';
  import { browseControls, type SortDir } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();

  const DIRECTIONS: SortDir[] = ['desc', 'asc'];

  const id = $props.id();
  const fieldOptions = $derived(
    sortFields(immichCapabilities.structuredSearch).map((value) => ({
      value,
      label: SORT_FIELD_SHORT[value]
    }))
  );
  const directionLabels = $derived(
    sortBasis === 'edit' ? EDIT_SORT_SHORT : SORT_DIRECTION_SHORT[browseControls.sortField]
  );
  const directionOptions = $derived(
    DIRECTIONS.map((value) => ({ value, label: directionLabels[value] }))
  );
</script>

<section aria-labelledby="{id}-sort" class="flex flex-col gap-2 px-4 pb-4">
  <h3 id="{id}-sort" class="section-label flex h-10 items-center text-dark/65">
    {sortBasis ? 'Sort and size' : 'Size'}
  </h3>
  {#if sortBasis === 'capture'}
    <SegmentedChoice
      appearance="view"
      fill
      options={fieldOptions}
      value={browseControls.sortField}
      label="Sort by"
      onChange={(field) => browseControls.setSortField(field)}
    />
  {/if}
  <div class="flex gap-2">
    {#if sortBasis}
      <div class="min-w-0 flex-1">
        <SegmentedChoice
          appearance="view"
          fill
          options={directionOptions}
          value={browseControls.sortDir}
          label="Order"
          onChange={(dir) => browseControls.setSortDir(dir)}
        />
      </div>
    {/if}
    <div class="min-w-0 flex-1">
      <SegmentedChoice
        appearance="view"
        fill
        options={GRID_SIZE_OPTIONS}
        value={browseView.gridSize}
        label="Thumbnail size"
        onChange={(size) => resizeGrid(size)}
      />
    </div>
  </div>
</section>
<section aria-labelledby="{id}-info" class="flex flex-col border-t border-hairline px-4 pb-4">
  <div class="flex h-11 items-center gap-2">
    <h3 id="{id}-info" class="section-label text-dark/65">Thumbnail info</h3>
    <TileOverlayReset />
  </div>
  <TileOverlayControls />
</section>
