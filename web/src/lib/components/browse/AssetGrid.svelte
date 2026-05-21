<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import AssetTile from './AssetTile.svelte';

  let {
    assets,
    activeId = null,
    loadingMore = false,
    onLoadMore
  }: {
    assets: AssetSummary[];
    activeId?: string | null;
    loadingMore?: boolean;
    onLoadMore?: () => void;
  } = $props();

  let sentinel: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!sentinel || !onLoadMore) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && !loadingMore) {
          onLoadMore();
        }
      },
      { rootMargin: '400px' }
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  });
</script>

<div
  class="grid gap-1 p-2"
  style="grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));"
>
  {#each assets as asset (asset.id)}
    <AssetTile {asset} active={asset.id === activeId} />
  {/each}
</div>
{#if onLoadMore}
  <div bind:this={sentinel} class="h-1"></div>
{/if}
{#if loadingMore}
  <div class="py-4 text-center text-xs text-immich-dark-fg/30">loading…</div>
{/if}
