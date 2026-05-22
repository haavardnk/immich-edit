<script lang="ts">
  import { onMount } from 'svelte';
  import { listEditedAssetIds } from '$lib/api/edits';
  import { editor } from '$lib/stores/editor.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let assets = $state<AssetSummary[]>([]);
  let loading = $state(true);

  onMount(async () => {
    editor.unload();
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
  <div class="px-4 py-2.5 text-xs text-immich-dark-fg/40 border-b border-white/5 flex items-center gap-2">
    <span class="font-semibold text-immich-dark-fg/70 text-sm">Edited</span>
    <span class="text-immich-dark-fg/20">·</span>
    <span>{assets.length} assets</span>
  </div>
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid {assets} />
  </div>
{/if}
