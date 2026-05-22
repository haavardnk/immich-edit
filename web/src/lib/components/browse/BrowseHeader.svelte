<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import {
    browseControls,
    type MediaType,
    type RatingFilter,
    type Visibility
  } from '$lib/stores/browseControls.svelte';
  import {
    mdiSortAscending,
    mdiSortDescending,
    mdiFilterOutline,
    mdiClose
  } from '@mdi/js';

  let {
    title,
    loaded,
    totalCount,
    favoriteLocked = false
  }: {
    title: string;
    loaded: number;
    totalCount?: number;
    favoriteLocked?: boolean;
  } = $props();

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

  const typeOptions: { value: MediaType; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'IMAGE', label: 'Photos' },
    { value: 'VIDEO', label: 'Videos' }
  ];

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
    browseControls.sortDir = browseControls.sortDir === 'asc' ? 'desc' : 'asc';
  }
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

  {#if !browseControls.isDefault}
    <button
      class="p-0.5 rounded hover:bg-white/10 text-immich-dark-fg/40 hover:text-immich-dark-fg/70"
      title="Reset filters & sort"
      onclick={() => browseControls.reset()}
    >
      <Icon path={mdiClose} size={14} />
    </button>
  {/if}

  <button
    class="p-0.5 rounded hover:bg-white/10"
    title={browseControls.sortDir === 'asc' ? 'Oldest first' : 'Newest first'}
    onclick={toggleDir}
  >
    <Icon
      path={browseControls.sortDir === 'asc' ? mdiSortAscending : mdiSortDescending}
      size={14}
    />
  </button>

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
          <span class="text-[10px] text-immich-dark-fg/40">Type</span>
          <select
            class="bg-white/5 text-[11px] rounded px-1.5 py-1 outline-none cursor-pointer hover:bg-white/10 w-full"
            value={browseControls.mediaType}
            onchange={(e) =>
              (browseControls.mediaType = (e.target as HTMLSelectElement).value as MediaType)}
          >
            {#each typeOptions as opt (opt.value)}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

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
