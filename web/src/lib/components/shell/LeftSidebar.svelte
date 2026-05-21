<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { ui } from '$lib/stores/ui.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { searchMetadata } from '$lib/api/search';
  import AlbumList from '$lib/components/library/AlbumList.svelte';
  import PeopleList from '$lib/components/library/PeopleList.svelte';
  import TagList from '$lib/components/library/TagList.svelte';
  import FolderTree from '$lib/components/library/FolderTree.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import type { AssetSummary } from '$lib/types/album';
  import {
    mdiImageMultipleOutline,
    mdiImageAlbum,
    mdiFolderOutline,
    mdiAccountOutline,
    mdiHeartOutline,
    mdiTagMultipleOutline,
    mdiPencilOutline,
    mdiMagnify,
    mdiClose,
    mdiChevronDown,
    mdiChevronRight,
    mdiChevronLeft,
  } from '@mdi/js';

  type ExpandableSection = 'people' | 'albums' | 'tags' | 'folders';

  let expanded = $state(new Set<ExpandableSection>());
  let searchResults = $state<AssetSummary[]>([]);
  let searching = $state(false);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  function toggleSection(id: ExpandableSection): void {
    if (expanded.has(id)) {
      expanded.delete(id);
    } else {
      expanded.add(id);
      void library.loadView(id);
    }
    expanded = new Set(expanded);
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

  const currentPath = $derived(page.url.pathname);

  onMount(() => {
    void library.load();
    void library.loadCounts();
  });
</script>

<aside
  class="bg-immich-dark-gray border-r border-white/5 flex flex-col min-h-0 transition-[width] duration-200 ease-out overflow-hidden"
  class:w-64={!ui.leftCollapsed}
  class:w-7={ui.leftCollapsed}
>
  {#if ui.leftCollapsed}
    <button
      class="flex-1 flex items-center justify-center hover:bg-white/5 transition-colors"
      onclick={ui.toggleLeft}
      aria-label="expand library panel"
      title="Library"
    >
      <Icon path={mdiChevronRight} size={16} class="opacity-40" />
    </button>
  {:else}
    <div class="flex items-center px-3 pt-2 pb-1">
      <span class="flex-1 text-[10px] uppercase tracking-widest text-immich-dark-fg/40 font-semibold pl-1">Library</span>
      <button
        class="p-0.5 rounded hover:bg-white/10 transition-colors"
        onclick={ui.toggleLeft}
        aria-label="collapse library panel"
        title="Collapse"
      >
        <Icon path={mdiChevronLeft} size={14} class="opacity-40" />
      </button>
    </div>
    <div class="px-3 pb-2">
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
      <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
        <!-- Photos -->
        <a
          href="/photos"
          class="flex items-center gap-2.5 py-2 px-4 transition-colors {currentPath === '/photos' ? 'bg-immich-dark-primary/15 text-immich-dark-primary' : 'text-immich-dark-fg/70 hover:bg-white/5'}"
        >
          <Icon path={mdiImageMultipleOutline} size={18} class="flex-none" />
          <span class="text-[13px] font-medium flex-1">Photos</span>
          {#if library.photosCount != null}
            <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{library.photosCount}</span>
          {/if}
        </a>

        <!-- People -->
        <div class="border-t border-white/5">
          <button
            class="w-full flex items-center gap-2.5 py-2 px-4 transition-colors text-immich-dark-fg/70 hover:bg-white/5"
            onclick={() => toggleSection('people')}
          >
            <Icon path={mdiAccountOutline} size={18} class="flex-none" />
            <span class="text-[13px] font-medium flex-1 text-left">People</span>
            {#if library.people.length > 0}
              <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{library.people.length}</span>
            {/if}
            <Icon path={expanded.has('people') ? mdiChevronDown : mdiChevronRight} size={16} class="opacity-40" />
          </button>
          {#if expanded.has('people')}
            <div class="pb-1">
              <PeopleList />
            </div>
          {/if}
        </div>

        <!-- Favorites -->
        <div class="border-t border-white/5">
          <a
            href="/favorites"
            class="flex items-center gap-2.5 py-2 px-4 transition-colors {currentPath === '/favorites' ? 'bg-immich-dark-primary/15 text-immich-dark-primary' : 'text-immich-dark-fg/70 hover:bg-white/5'}"
          >
            <Icon path={mdiHeartOutline} size={18} class="flex-none" />
            <span class="text-[13px] font-medium flex-1">Favorites</span>
            {#if library.favoritesCount != null}
              <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{library.favoritesCount}</span>
            {/if}
          </a>
        </div>

        <!-- Albums -->
        <div class="border-t border-white/5">
          <button
            class="w-full flex items-center gap-2.5 py-2 px-4 transition-colors text-immich-dark-fg/70 hover:bg-white/5"
            onclick={() => toggleSection('albums')}
          >
            <Icon path={mdiImageAlbum} size={18} class="flex-none" />
            <span class="text-[13px] font-medium flex-1 text-left">Albums</span>
            {#if library.albums.length > 0}
              <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{library.albums.length}</span>
            {/if}
            <Icon path={expanded.has('albums') ? mdiChevronDown : mdiChevronRight} size={16} class="opacity-40" />
          </button>
          {#if expanded.has('albums')}
            <div class="pb-1">
              <AlbumList />
            </div>
          {/if}
        </div>

        <!-- Tags -->
        <div class="border-t border-white/5">
          <button
            class="w-full flex items-center gap-2.5 py-2 px-4 transition-colors text-immich-dark-fg/70 hover:bg-white/5"
            onclick={() => toggleSection('tags')}
          >
            <Icon path={mdiTagMultipleOutline} size={18} class="flex-none" />
            <span class="text-[13px] font-medium flex-1 text-left">Tags</span>
            {#if library.tags.length > 0}
              <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{library.tags.length}</span>
            {/if}
            <Icon path={expanded.has('tags') ? mdiChevronDown : mdiChevronRight} size={16} class="opacity-40" />
          </button>
          {#if expanded.has('tags')}
            <div class="pb-1">
              <TagList />
            </div>
          {/if}
        </div>

        <!-- Folders -->
        <div class="border-t border-white/5">
          <button
            class="w-full flex items-center gap-2.5 py-2 px-4 transition-colors text-immich-dark-fg/70 hover:bg-white/5"
            onclick={() => toggleSection('folders')}
          >
            <Icon path={mdiFolderOutline} size={18} class="flex-none" />
            <span class="text-[13px] font-medium flex-1 text-left">Folders</span>
            {#if library.foldersCount != null && library.foldersCount > 0}
              <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{library.foldersCount}</span>
            {/if}
            <Icon path={expanded.has('folders') ? mdiChevronDown : mdiChevronRight} size={16} class="opacity-40" />
          </button>
          {#if expanded.has('folders')}
            <div class="pb-1">
              <FolderTree nodes={library.folderTree} />
            </div>
          {/if}
        </div>

        <!-- Edited -->
        <div class="border-t border-white/5">
          <a
            href="/edited"
            class="flex items-center gap-2.5 py-2 px-4 transition-colors {currentPath === '/edited' ? 'bg-immich-dark-primary/15 text-immich-dark-primary' : 'text-immich-dark-fg/70 hover:bg-white/5'}"
          >
            <Icon path={mdiPencilOutline} size={18} class="flex-none" />
            <span class="text-[13px] font-medium flex-1">Edited</span>
            {#if library.editedCount != null && library.editedCount > 0}
              <span class="text-[11px] text-immich-dark-fg/30 tabular-nums">{library.editedCount}</span>
            {/if}
          </a>
        </div>
      </div>
    {/if}
  {/if}
</aside>
