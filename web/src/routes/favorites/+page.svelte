<script lang="ts">
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { resumeId } from '#lib/browse/gridReturn.ts';
  import BrowsePage from '#lib/components/browse/BrowsePage.svelte';

  const feed = new BrowseFeed({ baseFilter: () => ({ favorite: true }) });

  onMount(() => {
    browseControls.enter('timeline', readFilterParams(page.url));
    void feed.resume(resumeId(page.url));
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowsePage
  {feed}
  title="Favorites"
  loadingLabel="Loading favorites…"
  favoriteLocked
  emptyTitle="No favorites yet"
  emptyMessage="Mark photos as favorites in Immich to see them here."
/>
