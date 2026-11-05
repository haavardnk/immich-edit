<script lang="ts">
  import SortMenu from '#lib/components/browse/SortMenu.svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { openFilterPanel } from '#lib/browse/filterPanel.ts';
  import { EDIT_SORT_ICONS, EDIT_SORT_LABELS } from '#lib/browse/sortOptions.ts';
  import { GRID_SIZE_OPTIONS, tileInfoLabel } from '#lib/browse/viewOptions.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiFilterOutline, mdiTextBoxOutline } from '@mdi/js';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();

  const editSortLabel = $derived(EDIT_SORT_LABELS[browseControls.sortDir]);
  const hasFilter = $derived(browseControls.isFiltered);

  function toggleDir(): void {
    browseControls.setSortDir(browseControls.sortDir === 'asc' ? 'desc' : 'asc');
  }
</script>

<div class="flex items-center gap-0.5">
  {#each GRID_SIZE_OPTIONS as opt (opt.value)}
    <Button
      size="tiny"
      variant={browseView.gridSize === opt.value ? 'filled' : 'ghost'}
      color={browseView.gridSize === opt.value ? 'primary' : 'secondary'}
      class="min-w-7 px-1.5 sm:min-w-8 sm:px-2"
      title={hint(opt.title, 'gridSize')}
      aria-label={opt.title}
      aria-pressed={browseView.gridSize === opt.value}
      onclick={() => browseView.setGridSize(opt.value)}
    >
      {opt.label}
    </Button>
  {/each}
  <IconButton
    size="small"
    variant="ghost"
    color={browseView.tileInfo === 'full' ? 'primary' : 'secondary'}
    icon={mdiTextBoxOutline}
    title={hint(tileInfoLabel(browseView.tileInfo), 'gridTileInfo')}
    aria-label={tileInfoLabel(browseView.tileInfo)}
    onclick={() => browseView.cycleTileInfo()}
  />
</div>

<div class="mx-1 h-4 w-px bg-hairline"></div>

{#if !browseControls.isDefault}
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    icon={mdiClose}
    title="Reset filters & sort"
    aria-label="Reset filters and sort"
    onclick={() => browseControls.reset()}
  />
{/if}

{#if sortBasis === 'capture'}
  <SortMenu />
{:else if sortBasis === 'edit'}
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    icon={EDIT_SORT_ICONS[browseControls.sortDir]}
    title={editSortLabel}
    aria-label={editSortLabel}
    onclick={toggleDir}
  />
{/if}

{#if !browseView.filtersOpen}
  <IconButton
    id="browse-filters-toggle"
    size="small"
    variant="ghost"
    color={hasFilter ? 'primary' : 'secondary'}
    icon={mdiFilterOutline}
    title={hint('Filters', 'toggleFilters')}
    aria-label={hasFilter ? 'Filters (active)' : 'Filters'}
    onclick={openFilterPanel}
  />
{/if}
