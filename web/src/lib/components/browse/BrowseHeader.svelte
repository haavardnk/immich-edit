<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import {
    browseControls,
    type RatingFilter,
    type Visibility
  } from '$lib/stores/browseControls.svelte';
  import { browseView, type GridSize } from '$lib/stores/browseView.svelte';
  import { mdiSortAscending, mdiSortDescending, mdiFilterOutline, mdiClose } from '@mdi/js';

  let {
    title,
    loaded,
    totalCount,
    favoriteLocked = false,
    hideSort = false,
    hideFilenameFilter = false,
    sortBasis = 'capture'
  }: {
    title: string;
    loaded: number;
    totalCount?: number;
    favoriteLocked?: boolean;
    hideSort?: boolean;
    hideFilenameFilter?: boolean;
    sortBasis?: 'capture' | 'edit';
  } = $props();

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

  const ratingOptions: { value: RatingFilter; label: string }[] = [
    { value: 'any', label: 'Any' },
    { value: 'unrated', label: 'Unrated' },
    { value: 1, label: '1 ★' },
    { value: 2, label: '2 ★' },
    { value: 3, label: '3 ★' },
    { value: 4, label: '4 ★' },
    { value: 5, label: '5 ★' }
  ];

  const visibilityOptions: { value: Visibility; label: string }[] = [
    { value: 'timeline', label: 'Timeline' },
    { value: 'archive', label: 'Archived' },
    { value: 'hidden', label: 'Hidden' }
  ];

  function toggleDir(): void {
    browseControls.setSortDir(browseControls.sortDir === 'asc' ? 'desc' : 'asc');
  }

  const sizeOptions: { value: GridSize; label: string }[] = [
    { value: 'sm', label: 'S' },
    { value: 'md', label: 'M' },
    { value: 'lg', label: 'L' },
    { value: 'xl', label: 'XL' }
  ];
</script>

<div
  class="px-4 py-2 text-xs text-immich-dark-fg/40 border-b border-white/5 flex items-center gap-2 flex-none"
>
  <span class="font-semibold text-immich-dark-fg/70 text-sm truncate">{title}</span>
  <span class="text-immich-dark-fg/20">·</span>
  {#if totalCount === undefined}
    <span>{loaded} loaded</span>
  {:else if loaded < totalCount}
    <span>{loaded} of {totalCount}</span>
  {:else}
    <span>{totalCount} assets</span>
  {/if}

  <div class="flex-1"></div>

  <div class="flex items-center rounded bg-white/5 p-0.5 gap-0.5">
    {#each sizeOptions as opt (opt.value)}
      <button
        class="px-1.5 py-0.5 rounded text-[10px] font-medium transition-colors {browseView.gridSize ===
        opt.value
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/40 hover:text-immich-dark-fg/70'}"
        title="Thumbnail size {opt.label}"
        onclick={() => browseView.setGridSize(opt.value)}
      >
        {opt.label}
      </button>
    {/each}
  </div>

  {#if !browseControls.isDefault}
    <button
      class="p-0.5 rounded hover:bg-white/10 text-immich-dark-fg/40 hover:text-immich-dark-fg/70"
      title="Reset filters & sort"
      onclick={() => browseControls.reset()}
    >
      <Icon path={mdiClose} size={14} />
    </button>
  {/if}

  {#if !hideSort}
    <button class="p-0.5 rounded hover:bg-white/10" title={sortLabel} onclick={toggleDir}>
      <Icon
        path={browseControls.sortDir === 'asc' ? mdiSortAscending : mdiSortDescending}
        size={14}
      />
    </button>
  {/if}

  <div class="relative">
    <button
      class="p-0.5 rounded hover:bg-white/10"
      class:text-immich-dark-primary={hasFilter}
      title="Filters"
      onclick={() => (filterOpen = !filterOpen)}
    >
      <Icon path={mdiFilterOutline} size={14} />
    </button>

    {#if filterOpen}
      <div
        class="absolute right-0 top-full mt-1 z-30 bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl p-3 flex flex-col gap-2.5 min-w-55"
      >
        <div class="flex items-center justify-between">
          <span class="text-[11px] text-immich-dark-fg/60 font-medium">Filters</span>
          <button class="p-0.5 rounded hover:bg-white/10" onclick={() => (filterOpen = false)}>
            <Icon path={mdiClose} size={12} />
          </button>
        </div>

        <label class="flex flex-col gap-1">
          <span class="text-[10px] text-immich-dark-fg/40">Visibility</span>
          <select
            class="bg-white/5 text-[11px] rounded px-1.5 py-1 outline-none cursor-pointer hover:bg-white/10 w-full"
            value={browseControls.visibility}
            onchange={(e) =>
              (browseControls.visibility = (e.target as HTMLSelectElement).value as Visibility)}
          >
            {#each visibilityOptions as opt (opt.value)}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-[10px] text-immich-dark-fg/40">Rating</span>
          <select
            class="bg-white/5 text-[11px] rounded px-1.5 py-1 outline-none cursor-pointer hover:bg-white/10 w-full"
            value={browseControls.rating}
            onchange={(e) => {
              const v = (e.target as HTMLSelectElement).value;
              browseControls.rating =
                v === 'any' || v === 'unrated' ? v : (Number(v) as 1 | 2 | 3 | 4 | 5);
            }}
          >
            {#each ratingOptions as opt (opt.value)}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        {#if !favoriteLocked}
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              class="checkbox checkbox-xs"
              checked={browseControls.favoriteOnly}
              onchange={(e) =>
                (browseControls.favoriteOnly = (e.target as HTMLInputElement).checked)}
            />
            <span class="text-[11px]">Favorites only</span>
          </label>
        {/if}

        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            class="checkbox checkbox-xs"
            checked={browseControls.excludeRejected}
            onchange={(e) =>
              (browseControls.excludeRejected = (e.target as HTMLInputElement).checked)}
          />
          <span class="text-[11px]">Exclude rejected</span>
        </label>

        {#if !hideFilenameFilter}
          <label class="flex flex-col gap-1">
            <span class="text-[10px] text-immich-dark-fg/40">Filename</span>
            <input
              type="text"
              class="bg-white/5 text-[11px] rounded px-1.5 py-1 outline-none w-full"
              placeholder="Search…"
              value={filenameLocal}
              oninput={onFilenameInput}
            />
          </label>
        {/if}

        <div class="grid grid-cols-2 gap-2">
          <label class="flex flex-col gap-1">
            <span class="text-[10px] text-immich-dark-fg/40">Taken after</span>
            <input
              type="date"
              class="bg-white/5 text-[11px] rounded px-1.5 py-1 outline-none w-full"
              value={browseControls.takenAfter}
              oninput={(e) => (browseControls.takenAfter = (e.target as HTMLInputElement).value)}
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-[10px] text-immich-dark-fg/40">Taken before</span>
            <input
              type="date"
              class="bg-white/5 text-[11px] rounded px-1.5 py-1 outline-none w-full"
              value={browseControls.takenBefore}
              oninput={(e) => (browseControls.takenBefore = (e.target as HTMLInputElement).value)}
            />
          </label>
        </div>

        {#if !browseControls.isDefault}
          <button
            class="text-[11px] text-immich-dark-primary hover:underline self-start"
            onclick={() => browseControls.reset()}
          >
            Reset all
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>
