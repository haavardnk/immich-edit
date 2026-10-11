<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { library } from '$lib/stores/library.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { readFilterParams } from '$lib/browse/filterParams';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseFilter: () => ({ scope: { person: id } }) });
  const title = $derived(library.people.find((p) => p.id === id)?.name || 'Person');

  $effect(() => {
    const _ = id;
    untrack(() => {
      browseControls.enter('collection', readFilterParams(page.url.searchParams));
      feed.reset();
      void feed.fetchPage(true);
    });
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowseShell
  {title}
  assets={feed.assets}
  loading={feed.loading && !feed.loadedOnce}
  loadingLabel="Loading photos…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.hasMore ? () => feed.loadMore() : undefined}
  onLoadAll={() => feed.loadAll()}
  totalCount={feed.totalCount}
  emptyTitle="No photos for this person"
/>
