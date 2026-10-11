<script lang="ts">
  import SortMenu from '$lib/components/browse/SortMenu.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { LABEL_TEXT } from '$lib/stores/labels';
  import { browseView, type GridSize, type TileInfo } from '$lib/stores/browseView.svelte';
  import { hint } from '$lib/shortcuts/labels';
  import {
    activeFilterChips,
    photoCountLabel,
    withoutFilter,
    type FilterKey
  } from '$lib/browse/filterChips';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { tick } from 'svelte';
  import {
    mdiSortAscending,
    mdiSortDescending,
    mdiFilterOutline,
    mdiCircle,
    mdiClose,
    mdiTextBoxOutline
  } from '@mdi/js';

  let {
    title,
    totalCount,
    hiddenCount = 0,
    hideSort = false,
    sortBasis = 'capture'
  }: {
    title: string;
    totalCount?: number;
    hiddenCount?: number;
    hideSort?: boolean;
    sortBasis?: 'capture' | 'edit';
  } = $props();

  const chips = $derived(activeFilterChips(browseControls.filters));

  function removeFilter(key: FilterKey): void {
    browseControls.applyFilters(withoutFilter(browseControls.filters, key));
  }

  const editSortLabel = $derived(
    browseControls.sortDir === 'asc' ? 'Oldest edit first' : 'Newest edit first'
  );

  const hasFilter = $derived(browseControls.isFiltered);

  function toggleDir(): void {
    browseControls.setSortDir(browseControls.sortDir === 'asc' ? 'desc' : 'asc');
  }

  async function openFilters(): Promise<void> {
    browseView.setFiltersOpen(true);
    await tick();
    document.getElementById('browse-filters-close')?.focus();
  }

  const sizeOptions: { value: GridSize; label: string }[] = [
    { value: 'sm', label: 'S' },
    { value: 'md', label: 'M' },
    { value: 'lg', label: 'L' },
    { value: 'xl', label: 'XL' }
  ];

  const TILE_INFO_LABELS: Record<TileInfo, string> = {
    none: 'hover only',
    badges: 'badges',
    full: 'name and date'
  };
</script>

<header
  class="flex h-12 shrink-0 items-center gap-3 border-b border-hairline bg-light px-3 sm:px-5"
>
  <div class="flex min-w-0 flex-1 items-baseline gap-2">
    <h1 class="truncate text-sm font-semibold text-white">{title}</h1>
    {#if totalCount !== undefined}
      <span class="shrink-0 text-white/20">·</span>
      <p class="shrink-0 text-[10px] text-dark/65">
        {photoCountLabel(totalCount, hiddenCount)}
      </p>
    {/if}
  </div>

  <div class="flex shrink-0 items-center gap-1.5">
    <div class="flex items-center gap-0.5">
      {#each sizeOptions as opt (opt.value)}
        <Button
          size="tiny"
          variant={browseView.gridSize === opt.value ? 'filled' : 'ghost'}
          color={browseView.gridSize === opt.value ? 'primary' : 'secondary'}
          class="min-w-7 px-1.5 sm:min-w-8 sm:px-2"
          title={hint(`Thumbnail size ${opt.label}`, 'gridSize')}
          aria-label="Thumbnail size {opt.label}"
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
        title={hint(`Thumbnail info: ${TILE_INFO_LABELS[browseView.tileInfo]}`, 'gridTileInfo')}
        aria-label={`Thumbnail info: ${TILE_INFO_LABELS[browseView.tileInfo]}`}
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

    {#if !hideSort && sortBasis === 'capture'}
      <SortMenu />
    {:else if !hideSort}
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={browseControls.sortDir === 'asc' ? mdiSortAscending : mdiSortDescending}
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
        onclick={openFilters}
      />
    {/if}
  </div>
</header>
{#if chips.length > 0}
  <div
    role="list"
    aria-label="Active filters"
    class="flex shrink-0 flex-wrap items-center gap-1.5 border-b border-hairline bg-light px-3 py-1.5 sm:px-5"
  >
    {#each chips as chip (chip.key)}
      <span
        role="listitem"
        class="inline-flex h-6 shrink-0 items-center gap-1 rounded-full bg-primary-200 pl-2.5 text-[11px] whitespace-nowrap"
      >
        {#if chip.color}
          <Icon icon={mdiCircle} size="10px" class={LABEL_TEXT[chip.color]} aria-hidden="true" />
        {/if}
        {chip.label}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          class="opacity-60 hover:opacity-100"
          icon={mdiClose}
          title={`Remove ${chip.label}`}
          aria-label={`Remove ${chip.label}`}
          onclick={() => removeFilter(chip.key)}
        />
      </span>
    {/each}
  </div>
{/if}
