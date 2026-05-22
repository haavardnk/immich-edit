<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { searchMetadata } from '$lib/api/search';
  import { album } from '$lib/stores/album.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import type { AssetSummary } from '$lib/types/album';

  const id = $derived(page.params.id as string);
  let assets = $state<AssetSummary[]>([]);
  let loadingAssets = $state(false);
  let loadedOnce = $state(false);
  let loadingMore = $state(false);
  let nextPage = $state<string | null>(null);
  let prevKey = $state('');

  function fetchPage(albumId: string, initial: boolean): void {
    if (initial) {
      if (!loadedOnce) loadingAssets = true;
      nextPage = null;
    }
    const body = browseControls.searchBody({ albumIds: [albumId] });
    if (!initial && nextPage) body.page = nextPage;
    searchMetadata(body)
      .then((result) => {
        assets = initial ? result.items : [...assets, ...result.items];
        browsing.set(assets);
        nextPage = result.nextPage;
      })
      .catch(() => {
        if (initial && album.current) {
          assets = album.current.assets;
          browsing.set(assets);
        }
      })
      .finally(() => {
        loadingAssets = false;
        loadedOnce = true;
        loadingMore = false;
      });
  }

  $effect(() => {
    const current = id;
    untrack(() => {
      album.load(current);
      browseControls.reset();
      prevKey = '';
      assets = [];
      loadedOnce = false;
      fetchPage(current, true);
    });
  });

  $effect(() => {
    const key = browseControls.serverFilterKey;
    if (prevKey && key !== prevKey) {
      fetchPage(id, true);
    }
    prevKey = key;
  });

  function loadMore(): void {
    if (loadingMore || !nextPage) return;
    loadingMore = true;
    fetchPage(id, false);
  }

  onMount(() => {
    editor.unload();
  });
</script>

{#if (album.loading && !album.current) || (loadingAssets && !loadedOnce)}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading album…</div>
{:else if album.error}
  <div class="flex-1 flex items-center justify-center text-sm text-red-400">{album.error}</div>
{:else if album.current}
  <BrowseHeader
    title={album.current.albumName}
    loaded={assets.length}
    totalCount={album.current.assetCount}
  />
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid {assets} {loadingMore} onLoadMore={nextPage ? loadMore : undefined} />
  </div>
{/if}
