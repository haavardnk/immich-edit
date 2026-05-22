<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseBody: () => ({ tagIds: [id] }) });
  const title = $derived(library.tags.find((t) => t.id === id)?.value || 'Tag');

  $effect(() => {
    const _ = id;
    untrack(() => {
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

{#if feed.loading && !feed.loadedOnce}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading…</div>
{:else}
  <BrowseHeader title={title} loaded={feed.assets.length} totalCount={feed.totalCount} />
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid assets={feed.assets} loadingMore={feed.loadingMore} onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined} />
  </div>
{/if}
