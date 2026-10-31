<script lang="ts">
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { loadEditedAssets } from '#lib/browse/sources.ts';
  import { timeOf } from '#lib/browse/sortAssets.ts';
  import { arrangeLoaded } from '#lib/browse/loadedFilter.ts';
  import BrowseShell from '#lib/components/browse/BrowseShell.svelte';
  import type { AssetSummary } from '#lib/types/album.ts';

  let loading = $state(true);
  let hydrated = $state<AssetSummary[]>([]);

  const assets = $derived(arrangeLoaded(hydrated, (a) => timeOf(a.updatedAt)));

  onMount(async () => {
    browseControls.enter('edited', readFilterParams(page.url));
    const [, entries] = await Promise.all([editedThumbs.loadOnce(), loadEditedAssets()]);
    hydrated = entries;
    loading = false;
  });

  $effect(() => {
    browsing.set(assets);
  });
</script>

<BrowseShell
  title="Edited"
  {assets}
  {loading}
  loadingLabel="Loading edited photos…"
  countLoaded
  sortBasis="edit"
  emptyTitle="No edited photos yet"
  emptyMessage="Open a photo and make edits to see it here."
/>
