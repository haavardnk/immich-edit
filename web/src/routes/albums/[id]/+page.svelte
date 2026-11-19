<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { album } from '#lib/stores/album.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { resumeId } from '#lib/browse/gridReturn.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import BrowsePage from '#lib/components/browse/BrowsePage.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseFilter: () => ({ scope: { album: id } }) });

  $effect(() => {
    const current = id;
    untrack(() => {
      selection.clear();
      void album.load(current);
      browseControls.enter('collection', readFilterParams(page.url));
      feed.reset();
      void feed.resume(resumeId(page.url));
    });
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowsePage
  {feed}
  title={album.current?.albumName ?? 'Album'}
  loading={album.loading && !album.current}
  loadingLabel="Loading album…"
  totalCount={feed.totalCount ?? album.current?.assetCount}
  error={album.error}
  onRetry={() => {
    void album.load(id);
    feed.reset();
    void feed.fetchPage(true);
  }}
  emptyTitle="This album is empty"
/>
