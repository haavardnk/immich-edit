<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { library } from '#lib/stores/library.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import BrowseShell from '#lib/components/browse/BrowseShell.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseFilter: () => ({ scope: { tag: id } }) });
  const title = $derived(library.tags.find((t) => t.id === id)?.value || 'Tag');

  $effect(() => {
    const _ = id;
    untrack(() => {
      selection.clear();
      browseControls.enter('collection', readFilterParams(page.url));
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
  loadingLabel="Loading tag…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.hasMore ? () => feed.loadMore() : undefined}
  onLoadAll={() => feed.loadAll()}
  totalCount={feed.totalCount}
  emptyTitle="No photos with this tag"
/>
