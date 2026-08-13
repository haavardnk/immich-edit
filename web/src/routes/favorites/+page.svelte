<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { BrowseFeed } from '$lib/stores/browseFeed.svelte';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';

  const feed = new BrowseFeed({ baseBody: () => ({ isFavorite: true }) });

  onMount(() => {
    editor.unload();
    browseControls.enter('favorites');
    feed.fetchPage(true);
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowseShell
  title="Favorites"
  assets={feed.assets}
  loading={feed.loading && !feed.loadedOnce}
  loadingLabel="Loading favorites…"
  loadingMore={feed.loadingMore}
  onLoadMore={feed.nextPage ? () => feed.loadMore() : undefined}
  totalCount={feed.totalCount}
  favoriteLocked
  emptyTitle="No favorites yet"
  emptyMessage="Mark photos as favorites in Immich to see them here."
/>
