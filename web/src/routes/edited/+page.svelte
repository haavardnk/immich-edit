<script lang="ts">
  import { onMount } from 'svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { rejected } from '$lib/stores/rejected.svelte';
  import { sortAssets } from '$lib/sortAssets';
  import { listEditedAssets } from '$lib/api/edits';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let loading = $state(true);
  let hydrated = $state<AssetSummary[]>([]);

  const assets = $derived<AssetSummary[]>(
    sortAssets(hydrated, (a) => a.updatedAt, browseControls.sortDir)
  );

  async function hydrate(): Promise<void> {
    const entries = await listEditedAssets(true);
    hydrated = rejected.stamp(
      entries.map<AssetSummary>((entry) =>
        entry.asset
          ? { ...entry.asset, updatedAt: entry.updated_at }
          : {
              id: entry.id,
              originalFileName: `Asset ${entry.id}`,
              type: 'IMAGE',
              fileCreatedAt: null,
              updatedAt: entry.updated_at,
              checksum: null,
              isFavorite: false,
              exifInfo: null,
              tags: []
            }
      )
    );
  }

  onMount(async () => {
    browseControls.enter('edited', 'edited');
    await Promise.all([editedThumbs.loadOnce(), rejected.load().catch(() => undefined), hydrate()]);
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
