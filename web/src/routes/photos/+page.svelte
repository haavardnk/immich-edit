<script lang="ts">
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { readFilterParams } from '$lib/browse/filterParams';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const feed = new BrowseFeed({ baseFilter: () => ({}) });

  onMount(() => {
    browseControls.enter('timeline', readFilterParams(page.url.searchParams));
    void feed.fetchPage(true);
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowseShell
  title="Photos"
  assets={feed.assets}
  loading={feed.loading && !feed.loadedOnce}
  loadingLabel="Loading photos…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.hasMore ? () => feed.loadMore() : undefined}
  onLoadAll={() => feed.loadAll()}
  totalCount={feed.totalCount}
  emptyTitle="No photos"
  emptyMessage="Connect an Immich library or upload assets to get started."
/>
