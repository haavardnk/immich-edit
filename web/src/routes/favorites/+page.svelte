<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';

  const feed = new BrowseFeed({ baseBody: () => ({ isFavorite: true }) });

  onMount(() => {
    editor.unload();
    browseControls.reset();
    feed.fetchPage(true);
  });

  $effect(() => feed.watchFilterChange());
</script>

{#if feed.loading && !feed.loadedOnce}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading…</div>
{:else}
  <BrowseHeader title="Favorites" loaded={feed.assets.length} totalCount={feed.totalCount} favoriteLocked />
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid assets={feed.assets} loadingMore={feed.loadingMore} onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined} />
  </div>
{/if}
