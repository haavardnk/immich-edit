<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  const feed = new BrowseFeed({ baseBody: () => ({}) });

  onMount(() => {
    editor.unload();
    browseControls.reset();
    feed.fetchPage(true);
  });

  $effect(() => feed.watchFilterChange());
</script>

{#if feed.loading && !feed.loadedOnce}
  <div class="flex-1 flex items-center justify-center"><Spinner label="Loading photos…" /></div>
{:else}
  <BrowseHeader title="Photos" loaded={feed.assets.length} totalCount={feed.totalCount} />
  {#if feed.assets.length === 0}
    <EmptyState title="No photos" message="Connect an Immich library or upload assets to get started." />
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
      <AssetGrid assets={feed.assets} loadingMore={feed.loadingMore} onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined} />
    </div>
  {/if}
{/if}
