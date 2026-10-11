<script lang="ts">
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { readFilterParams } from '$lib/browse/filterParams';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const feed = new BrowseFeed({ baseFilter: () => ({ favorite: true }) });

  onMount(() => {
    browseControls.enter('timeline', readFilterParams(page.url.searchParams));
    void feed.fetchPage(true);
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowseShell
  title="Favorites"
  assets={feed.assets}
  loading={feed.loading && !feed.loadedOnce}
  loadingLabel="Loading favorites…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.hasMore ? () => feed.loadMore() : undefined}
  onLoadAll={() => feed.loadAll()}
  totalCount={feed.totalCount}
  favoriteLocked
  emptyTitle="No favorites yet"
  emptyMessage="Mark photos as favorites in Immich to see them here."
/>
