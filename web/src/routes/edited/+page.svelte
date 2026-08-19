<script lang="ts">
  import { onMount } from 'svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { sortAssets } from '$lib/sortAssets';
  import BrowseShell from '$lib/components/browse/BrowseShell.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let loading = $state(true);

  const assets = $derived<AssetSummary[]>(
    sortAssets(
      editedThumbs.entries.map((e) => ({
        id: e.id,
        originalFileName: '',
        type: 'IMAGE',
        fileCreatedAt: null,
        updatedAt: e.updated_at,
        checksum: null,
        isFavorite: false,
        exifInfo: null,
        tags: []
      })),
      (a) => a.updatedAt,
      browseControls.sortDir
    )
  );

  onMount(async () => {
    editor.unload();
    browseControls.enter('edited', 'edited');
    await editedThumbs.loadOnce();
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
