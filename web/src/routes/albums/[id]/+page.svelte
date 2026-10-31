<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { album } from '#lib/stores/album.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import BrowseShell from '#lib/components/browse/BrowseShell.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseFilter: () => ({ scope: { album: id } }) });

  $effect(() => {
    const current = id;
    untrack(() => {
      selection.clear();
      void album.load(current);
      browseControls.enter('collection', readFilterParams(page.url));
      feed.reset();
      void feed.fetchPage(true);
    });
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowseShell
  title={album.current?.albumName ?? 'Album'}
  assets={feed.assets}
  loading={(album.loading && !album.current) || (feed.loading && !feed.loadedOnce)}
  loadingLabel="Loading album…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.hasMore ? () => feed.loadMore() : undefined}
  onLoadAll={() => feed.loadAll()}
  totalCount={feed.totalCount ?? album.current?.assetCount}
  error={album.error}
  onRetry={() => {
    void album.load(id);
    feed.reset();
    void feed.fetchPage(true);
  }}
  emptyTitle="This album is empty"
/>
