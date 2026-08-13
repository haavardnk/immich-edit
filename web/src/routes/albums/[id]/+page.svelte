<script lang="ts">
  import { page } from '$app/state';
  import { onMount, untrack } from 'svelte';
  import { album } from '$lib/stores/album.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({
    baseBody: () => ({ albumIds: [id] }),
    onFetchError: (initial) => {
      if (initial && album.current) {
        feed.assets = album.current.assets;
        browsing.set(feed.assets);
      }
    }
  });

  $effect(() => {
    const current = id;
    untrack(() => {
      selection.clear();
      album.load(current);
      browseControls.enter('album:' + current);
      feed.reset();
      feed.fetchPage(true);
    });
  });

  $effect(() => feed.watchFilterChange());

  onMount(() => {
    editor.unload();
  });
</script>

<BrowseShell
  title={album.current?.albumName ?? 'Album'}
  assets={feed.assets}
  loading={(album.loading && !album.current) || (feed.loading && !feed.loadedOnce)}
  loadingLabel="Loading album…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined}
  totalCount={feed.totalCount ?? album.current?.assetCount}
  error={album.error}
  emptyTitle="This album is empty"
/>
