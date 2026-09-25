<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { browseControls, type Visibility } from '$lib/stores/browseControls.svelte';
  import { browseView, type GridSize, type TileInfo } from '$lib/stores/browseView.svelte';
  import { mergeProps } from '$lib/utils/mergeProps';
  import { hint } from '$lib/keybinds';
  import {
    activeFilterChips,
    photoCountLabel,
    withoutFilter,
    type FilterKey
  } from '$lib/browseFilterChips';
  import { Button, Field, IconButton, Select } from '@immich/ui';
  import {
    mdiSortAscending,
    mdiSortDescending,
    mdiFilterOutline,
    mdiClose,
    mdiTextBoxOutline
  } from '@mdi/js';

  let {
    title,
    totalCount,
    hiddenCount = 0,
    favoriteLocked = false,
    hideSort = false,
    hideFilenameFilter = false,
    sortBasis = 'capture'
  }: {
    title: string;
    totalCount?: number;
    hiddenCount?: number;
    favoriteLocked?: boolean;
    hideSort?: boolean;
    hideFilenameFilter?: boolean;
    sortBasis?: 'capture' | 'edit';
  } = $props();

  const chips = $derived(activeFilterChips(browseControls.filters));

  function removeFilter(key: FilterKey): void {
    browseControls.applyFilters(withoutFilter(browseControls.filters, key));
  }

  const sortLabel = $derived(
    browseControls.sortDir === 'asc'
      ? sortBasis === 'edit'
        ? 'Oldest edit first'
        : 'Oldest first'
      : sortBasis === 'edit'
        ? 'Newest edit first'
        : 'Newest first'
  );

  let filterOpen = $state(false);
  let filenameLocal = $state(browseControls.filename);
  let filenameTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (browseControls.filename !== filenameLocal && filenameTimer === null) {
      filenameLocal = browseControls.filename;
    }
  });

  function onFilenameInput(e: Event): void {
    filenameLocal = (e.target as HTMLInputElement).value;
    if (filenameTimer) clearTimeout(filenameTimer);
    filenameTimer = setTimeout(() => {
      browseControls.filename = filenameLocal;
      filenameTimer = null;
    }, 350);
  }

  const hasFilter = $derived(browseControls.isFiltered);

  const ratingOptions: { value: string; label: string }[] = [
    { value: 'any', label: 'Any' },
    { value: 'unrated', label: 'Unrated' },
    { value: '1', label: '1 ★' },
    { value: '2', label: '2 ★' },
    { value: '3', label: '3 ★' },
    { value: '4', label: '4 ★' },
    { value: '5', label: '5 ★' }
  ];

  const visibilityOptions: { value: Visibility; label: string }[] = [
    { value: 'timeline', label: 'Timeline' },
    { value: 'archive', label: 'Archived' },
    { value: 'hidden', label: 'Hidden' }
  ];

  function setRating(value: string): void {
    browseControls.rating =
      value === 'any' || value === 'unrated' ? value : (Number(value) as 1 | 2 | 3 | 4 | 5);
  }

  function toggleDir(): void {
    browseControls.setSortDir(browseControls.sortDir === 'asc' ? 'desc' : 'asc');
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

    {#if !hideSort}
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={browseControls.sortDir === 'asc' ? mdiSortAscending : mdiSortDescending}
        title={sortLabel}
        aria-label={sortLabel}
        onclick={toggleDir}
      />
    {/if}

    <Popover
      open={filterOpen}
      align="end"
      onOpenChange={(v) => (filterOpen = v)}
      contentClass="flex min-w-55 flex-col gap-2.5 p-3"
    >
      {#snippet trigger(popoverProps)}
        <IconButton
          size="small"
          variant="ghost"
          color={hasFilter ? 'primary' : 'secondary'}
          icon={mdiFilterOutline}
          title="Filters"
          aria-label={hasFilter ? 'Filters (active)' : 'Filters'}
          {...mergeProps(popoverProps)}
        />
      {/snippet}
      <div class="flex items-center justify-between">
        <span class="text-[11px] text-dark/65 font-medium">Filters</span>
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          icon={mdiClose}
          aria-label="Close filters"
          onclick={() => (filterOpen = false)}
        />
      </div>

      <Field label="Visibility" size="small">
        <Select
          size="small"
          options={visibilityOptions}
          value={browseControls.visibility}
          onChange={(v) => (browseControls.visibility = v)}
        />
      </Field>

      <Field label="Rating" size="small">
        <Select
          size="small"
          options={ratingOptions}
          value={String(browseControls.rating)}
          onChange={setRating}
        />
      </Field>

      {#if !favoriteLocked}
        <CheckboxRow
          label="Favorites only"
          checked={browseControls.favoriteOnly}
          onChange={(checked) => (browseControls.favoriteOnly = checked)}
        />
      {/if}

      <CheckboxRow
        label="Exclude rejected"
        checked={browseControls.excludeRejected}
        onChange={(checked) => (browseControls.excludeRejected = checked)}
      />

      {#if !hideFilenameFilter}
        <Field label="Filename" size="small">
          <TextInput
            size="small"
            type="text"
            placeholder="Search…"
            value={filenameLocal}
            oninput={onFilenameInput}
          />
        </Field>
      {/if}

      <div class="grid grid-cols-2 gap-2">
        <Field label="Taken after" size="small">
          <TextInput
            size="small"
            type="date"
            value={browseControls.takenAfter}
            oninput={(e) => (browseControls.takenAfter = e.currentTarget.value)}
          />
        </Field>
        <Field label="Taken before" size="small">
          <TextInput
            size="small"
            type="date"
            value={browseControls.takenBefore}
            oninput={(e) => (browseControls.takenBefore = e.currentTarget.value)}
          />
        </Field>
      </div>

      {#if !browseControls.isDefault}
        <Button
          size="small"
          variant="ghost"
          color="primary"
          class="self-start"
          onclick={() => browseControls.reset()}
        >
          Reset all
        </Button>
      {/if}
    </Popover>
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
