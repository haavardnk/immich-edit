<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseBody: () => ({ personIds: [id] }) });
  const title = $derived(library.people.find((p) => p.id === id)?.name || 'Person');

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
  <div class="flex-1 flex items-center justify-center"><Spinner label="Loading photos…" /></div>
{:else}
  <BrowseHeader title={title} loaded={feed.assets.length} totalCount={feed.totalCount} />
  {#if feed.assets.length === 0}
    <EmptyState title="No photos for this person" />
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
      <AssetGrid assets={feed.assets} loadingMore={feed.loadingMore} onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined} />
    </div>
  {/if}
{/if}
