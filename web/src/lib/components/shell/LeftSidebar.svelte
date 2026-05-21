<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { ui } from '$lib/stores/ui.svelte';
  import { library, type LibraryView } from '$lib/stores/library.svelte';
  import { searchMetadata } from '$lib/api/search';
  import AlbumList from '$lib/components/library/AlbumList.svelte';
  import PeopleList from '$lib/components/library/PeopleList.svelte';
  import TagList from '$lib/components/library/TagList.svelte';
  import FolderTree from '$lib/components/library/FolderTree.svelte';
  import FavoriteLink from '$lib/components/library/FavoriteLink.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import type { AssetSummary } from '$lib/types/album';
  import {
    mdiImageAlbum,
    mdiFolderOutline,
    mdiAccountOutline,
    mdiHeartOutline,
    mdiTagMultipleOutline,
    mdiMagnify,
    mdiClose,
  } from '@mdi/js';

  const views: { id: LibraryView; label: string; icon: string }[] = [
    { id: 'albums', label: 'Albums', icon: mdiImageAlbum },
    { id: 'folders', label: 'Folders', icon: mdiFolderOutline },
    { id: 'people', label: 'People', icon: mdiAccountOutline },
    { id: 'favorites', label: 'Favorites', icon: mdiHeartOutline },
    { id: 'tags', label: 'Tags', icon: mdiTagMultipleOutline },
  ];

  let searchResults = $state<AssetSummary[]>([]);
  let searching = $state(false);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  function switchView(v: LibraryView): void {
    void library.loadView(v);
  }

  async function doSearch(query: string): Promise<void> {
    if (!query.trim()) {
      searchResults = [];
      return;
    }
    searching = true;
    const result = await searchMetadata({ originalFileName: query, size: 50 });
    searchResults = result.items;
    searching = false;
  }

  function onSearchInput(): void {
    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => doSearch(ui.searchQuery), 300);
  }

  function clearSearch(): void {
    ui.searchQuery = '';
    searchResults = [];
  }

  function goToAsset(id: string): void {
    clearSearch();
    void goto(`/assets/${id}`);
  }

  onMount(() => {
    void library.load();
  });
</script>

<aside
  class="bg-immich-dark-gray border-r border-white/5 flex flex-col min-h-0 transition-[width] duration-200 ease-out overflow-hidden"
  class:w-64={!ui.leftCollapsed}
  class:w-0={ui.leftCollapsed}
>
  {#if !ui.leftCollapsed}
    <div class="px-3 pt-3 pb-2">
      <div class="relative">
        <Icon path={mdiMagnify} size={16} class="absolute left-2.5 top-1/2 -translate-y-1/2 opacity-40" />
        <input
          type="text"
          placeholder="Search…"
          bind:value={ui.searchQuery}
          oninput={onSearchInput}
          class="w-full bg-white/5 border border-white/10 rounded-lg pl-8 pr-8 py-1.5 text-xs text-immich-dark-fg placeholder:text-immich-dark-fg/30 outline-none focus:border-immich-dark-primary/50 transition-colors"
        />
        {#if ui.searchQuery}
          <button
            class="absolute right-2 top-1/2 -translate-y-1/2 opacity-40 hover:opacity-80"
            onclick={clearSearch}
          >
            <Icon path={mdiClose} size={14} />
          </button>
        {/if}
      </div>
    </div>

    {#if ui.searchQuery}
      <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden px-1">
        {#if searching}
          <div class="p-3 text-xs opacity-40">searching…</div>
        {:else if searchResults.length === 0}
          <div class="p-3 text-xs opacity-40">no results</div>
        {:else}
          <div class="px-2 py-1 text-[10px] opacity-40 uppercase tracking-wider">{searchResults.length} results</div>
          {#each searchResults as asset (asset.id)}
            <button
              class="w-full text-left flex items-center gap-2 py-1.5 px-2.5 rounded-lg hover:bg-white/5 transition-colors"
              onclick={() => goToAsset(asset.id)}
            >
              <img
                src={`/api/assets/${asset.id}/thumbnail`}
                alt=""
                loading="lazy"
                class="w-8 h-8 rounded object-cover flex-none"
              />
              <span class="truncate text-xs">{asset.originalFileName}</span>
            </button>
          {/each}
        {/if}
      </div>
    {:else}
      <nav class="flex gap-0.5 px-2 pb-2">
        {#each views as v (v.id)}
          {@const active = library.view === v.id}
          <button
            class="flex flex-col items-center gap-0.5 flex-1 py-1.5 rounded-lg transition-colors text-[10px] {active ? 'bg-immich-dark-primary/15 text-immich-dark-primary' : 'text-immich-dark-fg/50 hover:bg-white/5'}"
            onclick={() => switchView(v.id)}
            title={v.label}
          >
            <Icon path={v.icon} size={16} />
            <span>{v.label}</span>
          </button>
        {/each}
      </nav>

      <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden px-1">
        {#if library.loading}
          <div class="p-3 text-xs opacity-40">loading…</div>
        {:else if library.error}
          <div class="p-3 text-xs text-red-400">{library.error}</div>
        {:else if library.view === 'albums'}
          <AlbumList />
        {:else if library.view === 'folders'}
          <FolderTree nodes={library.folderTree} />
        {:else if library.view === 'people'}
          <PeopleList />
        {:else if library.view === 'favorites'}
          <FavoriteLink />
        {:else if library.view === 'tags'}
          <TagList />
        {/if}
      </div>
    {/if}
  {/if}
</aside>
