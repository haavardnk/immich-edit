<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { library } from '$lib/stores/library.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseBody: () => ({ personIds: [id] }) });
  const title = $derived(library.people.find((p) => p.id === id)?.name || 'Person');

  $effect(() => {
    const _ = id;
    untrack(() => {
      browseControls.enter('person:' + id, 'collection');
      feed.reset();
      feed.fetchPage(true);
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
  onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined}
  totalCount={feed.totalCount}
  emptyTitle="No photos for this person"
/>
