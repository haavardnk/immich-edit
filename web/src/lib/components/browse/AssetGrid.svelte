<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import AssetTile from './AssetTile.svelte';

  let {
    assets,
    activeId = null
  }: { assets: AssetSummary[]; activeId?: string | null } = $props();
</script>

<div
  class="grid gap-1 p-2"
  style="grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));"
>
  {#each assets as asset (asset.id)}
    <AssetTile {asset} active={asset.id === activeId} />
  {/each}
</div>
