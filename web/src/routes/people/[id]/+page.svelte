<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { library } from '#lib/stores/library.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { resumeId } from '#lib/browse/gridReturn.ts';
  import BrowsePage from '#lib/components/browse/BrowsePage.svelte';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseFilter: () => ({ scope: { person: id } }) });
  const title = $derived(library.people.find((p) => p.id === id)?.name || 'Person');

  $effect(() => {
    const _ = id;
    untrack(() => {
      browseControls.enter('collection', readFilterParams(page.url));
      feed.reset();
      void feed.open(resumeId(page.url));
    });
    return () => feed.close();
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowsePage {feed} {title} loadingLabel="Loading photos…" emptyTitle="No photos for this person" />
