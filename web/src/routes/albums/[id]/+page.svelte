<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { album } from '$lib/stores/album.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({
    baseBody: () => ({ albumIds: [id] }),
    includeStats: false,
    onFetchError: (initial) => {
      if (initial && album.current) {
        feed.assets = album.current.assets;
        browsing.set(feed.assets);
      }
    },
  });

  $effect(() => {
    const current = id;
    untrack(() => {
      album.load(current);
      browseControls.reset();
      feed.reset();
      feed.fetchPage(true);
    });
  });

  $effect(() => feed.watchFilterChange());

  onMount(() => {
    editor.unload();
  });
</script>

{#if (album.loading && !album.current) || (feed.loading && !feed.loadedOnce)}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading album…</div>
{:else if album.error}
  <div class="flex-1 flex items-center justify-center text-sm text-red-400">{album.error}</div>
{:else if album.current}
  <BrowseHeader
    title={album.current.albumName}
    loaded={feed.assets.length}
    totalCount={album.current.assetCount}
  />
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid assets={feed.assets} loadingMore={feed.loadingMore} onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined} />
  </div>
{/if}
