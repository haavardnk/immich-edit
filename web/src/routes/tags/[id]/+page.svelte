<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { library } from '$lib/stores/library.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseBody: () => ({ tagIds: [id] }) });
  const title = $derived(library.tags.find((t) => t.id === id)?.value || 'Tag');

  $effect(() => {
    const _ = id;
    untrack(() => {
      selection.clear();
      browseControls.enter('tag:' + id, 'collection');
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
  loadingLabel="Loading tag…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined}
  totalCount={feed.totalCount}
  emptyTitle="No photos with this tag"
/>
