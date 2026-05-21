<script lang="ts">
  import { onMount } from 'svelte';
  import { searchMetadata } from '$lib/api/search';
  import { editor } from '$lib/stores/editor.svelte';
  import AssetGrid from '$lib/components/browse/AssetGrid.svelte';
  import type { AssetSummary } from '$lib/types/album';

  let assets = $state<AssetSummary[]>([]);
  let loading = $state(true);

  onMount(async () => {
    editor.unload();
    const result = await searchMetadata({ isFavorite: true, size: 500 });
    assets = result.items;
    loading = false;
  });
</script>

{#if loading}
  <div class="flex-1 flex items-center justify-center text-sm text-immich-dark-fg/40">loading…</div>
{:else}
  <div class="px-4 py-2.5 text-xs text-immich-dark-fg/40 border-b border-white/5 flex items-center gap-2">
    <span class="font-semibold text-immich-dark-fg/70 text-sm">Favorites</span>
    <span class="text-immich-dark-fg/20">·</span>
    <span>{assets.length} assets</span>
  </div>
  <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
    <AssetGrid {assets} />
  </div>
{/if}
