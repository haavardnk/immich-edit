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
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';

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
  const overlaySummary = $derived(
    `${countOf(preferences.overlay.icons.length, 'icon')} · ${countOf(preferences.overlay.details.length, 'detail')}`
  );
</script>

<section aria-labelledby="view-block-heading" class="flex flex-col gap-2 px-4 pb-4">
  <h3 id="view-block-heading" class="section-label flex h-10 items-center text-dark/65">View</h3>
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
  <button
    type="button"
    aria-label="Icons and details, {overlaySummary}"
    class="flex h-12 items-center gap-2 rounded-xl bg-ghost pr-3 pl-3.5 text-left transition-colors outline-none hover:bg-light-200 focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-inset"
    onclick={() => (browseView.filterPage = 'tileOverlay')}
  >
    <span class="flex-1 text-sm font-medium text-light-950">Icons and details</span>
    <span class="min-w-0 truncate text-caption text-dark/65">{overlaySummary}</span>
    <Icon icon={mdiChevronRight} size="20px" class="shrink-0 text-dark/65" aria-hidden="true" />
  </button>
</section>
