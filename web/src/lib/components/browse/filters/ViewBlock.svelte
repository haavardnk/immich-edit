<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { resizeGrid } from '#lib/browse/gridResize.ts';
  import {
    EDIT_SORT_SHORT,
    SORT_DIRECTION_SHORT,
    SORT_FIELD_SHORT,
    sortFields
  } from '#lib/browse/sortOptions.ts';
  import { GRID_SIZE_OPTIONS, TILE_INFO_OPTIONS } from '#lib/browse/viewOptions.ts';
  import { browseControls, type SortDir } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();

  const DIRECTIONS: SortDir[] = ['desc', 'asc'];

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

<section aria-labelledby="view-block-heading" class="flex flex-col gap-2 px-4 pb-4">
  <h3
    id="view-block-heading"
    class="flex h-10 items-center text-xs font-semibold tracking-[.06em] text-dark/65 uppercase"
  >
    View
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
  <SegmentedChoice
    appearance="view"
    fill
    options={TILE_INFO_OPTIONS}
    value={browseView.tileInfo}
    label="Thumbnail info"
    onChange={(info) => browseView.setTileInfo(info)}
  />
</section>
