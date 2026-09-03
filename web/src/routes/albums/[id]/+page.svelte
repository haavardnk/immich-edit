<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { album } from '$lib/stores/album.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({
    baseBody: () => ({ albumIds: [id] }),
    onFetchError: (initial, error) => {
      if (initial && album.current) {
        feed.assets = album.current.assets;
        browsing.set(feed.assets);
      } else if (!initial) {
        toasts.fail('load', error);
      }
    }
  });

  $effect(() => {
    const current = id;
    untrack(() => {
      selection.clear();
      void album.load(current);
      browseControls.enter('album:' + current, 'collection');
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
  onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined}
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
