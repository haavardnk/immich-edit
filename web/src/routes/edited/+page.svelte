<script lang="ts">
  import { onMount } from 'svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { sortAssets } from '$lib/sortAssets';
  import { getAsset } from '$lib/api/assets';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let loading = $state(true);
  let hydrated = $state<AssetSummary[]>([]);

  const assets = $derived<AssetSummary[]>(
    sortAssets(hydrated, (a) => a.updatedAt, browseControls.sortDir)
  );

  async function hydrate(): Promise<void> {
    const entries = editedThumbs.entries;
    const results = entries.map<AssetSummary>((entry) => ({
      id: entry.id,
      originalFileName: entry.id,
      type: 'IMAGE',
      fileCreatedAt: null,
      updatedAt: entry.updated_at,
      checksum: null,
      isFavorite: false,
      exifInfo: null,
      tags: []
    }));
    let next = 0;
    const workers = Array.from({ length: Math.min(6, entries.length) }, async () => {
      while (next < entries.length) {
        const index = next++;
        const entry = entries[index];
        try {
          const asset = await getAsset(entry.id);
          results[index] = { ...asset, updatedAt: entry.updated_at };
        } catch {
          results[index] = { ...results[index], originalFileName: `Asset ${entry.id}` };
        }
      }
    });
    await Promise.all(workers);
    hydrated = results;
  }

  onMount(async () => {
    browseControls.enter('edited', 'edited');
    await editedThumbs.loadOnce();
    await hydrate();
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
