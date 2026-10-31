<script lang="ts">
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import BrowseShell from '#lib/components/browse/BrowseShell.svelte';

  const feed = new BrowseFeed({ baseFilter: () => ({}) });

  onMount(() => {
    browseControls.enter('timeline', readFilterParams(page.url));
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
