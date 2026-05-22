<script lang="ts">
  import { onMount } from 'svelte';
  import { listEditedAssetIds } from '$lib/api/edits';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import BrowseHeader from '$lib/components/browse/BrowseHeader.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let assets = $state<AssetSummary[]>([]);
  let loading = $state(true);

  onMount(async () => {
    editor.unload();
    browseControls.reset();
    const ids = await listEditedAssetIds();
    assets = ids.map((id) => ({
      id,
      originalFileName: '',
      type: 'IMAGE',
      fileCreatedAt: null,
      updatedAt: null,
      checksum: null,
      isFavorite: false,
      exifInfo: null,
    }));
    browsing.set(assets);
    loading = false;
  });
</script>

{#if loading}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading…</div>
{:else if assets.length === 0}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/30">No edited photos yet</div>
{:else}
  <BrowseHeader title="Edited" loaded={assets.length} />
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid {assets} />
  </div>
{/if}
