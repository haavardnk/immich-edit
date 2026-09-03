<script lang="ts">
  import { onMount } from 'svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { loadEditedAssets } from '$lib/browseSources';
  import { sortAssets } from '$lib/sortAssets';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let loading = $state(true);
  let hydrated = $state<AssetSummary[]>([]);

  const assets = $derived<AssetSummary[]>(
    sortAssets(hydrated, (a) => a.updatedAt, browseControls.sortDir)
  );

  onMount(async () => {
    browseControls.enter('edited', 'edited');
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
  totalCount={assets.length}
  sortBasis="edit"
  emptyTitle="No edited photos yet"
  emptyMessage="Open a photo and make edits to see it here."
/>
