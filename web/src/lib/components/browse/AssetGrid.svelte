<script lang="ts">
  import { onMount } from 'svelte';
  import type { AssetSummary } from '$lib/types/album';
  import AssetTile from './AssetTile.svelte';
  import BulkActionBar from './BulkActionBar.svelte';
  import { selection } from '$lib/stores/selection.svelte';

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

  const MIN = 140;
  const GAP = 4;
  const PAD = 8;
  const OVERSCAN = 2;

  let root: HTMLDivElement | undefined = $state();
  let scrollParent: HTMLElement | null = null;
  let gridWidth = $state(0);
  let parentHeight = $state(0);
  let viewTop = $state(0);

  const layout = $derived.by(() => {
    const inner = Math.max(0, gridWidth - PAD * 2);
    const cols = Math.max(1, Math.floor((inner + GAP) / (MIN + GAP)));
    const colWidth = (inner - (cols - 1) * GAP) / cols;
    const rowStride = colWidth + GAP;
    const rowCount = Math.ceil(assets.length / cols);
    const totalHeight = PAD * 2 + rowCount * colWidth + Math.max(0, rowCount - 1) * GAP;
    return { cols, rowStride, rowCount, totalHeight };
  });

  const view = $derived.by(() => {
    const { cols, rowStride, rowCount } = layout;
    const startRow = Math.max(0, Math.floor((viewTop - PAD) / rowStride) - OVERSCAN);
    const rowsInView = Math.ceil(parentHeight / rowStride) + OVERSCAN * 2;
    const endRow = Math.min(rowCount, startRow + rowsInView);
    return {
      startIdx: startRow * cols,
      endIdx: Math.min(assets.length, endRow * cols),
      offsetY: PAD + startRow * rowStride
    };
  });

  const visibleAssets = $derived(assets.slice(view.startIdx, view.endIdx));

  function findScrollParent(el: HTMLElement): HTMLElement | null {
    let p = el.parentElement;
    while (p) {
      const oy = getComputedStyle(p).overflowY;
      if (oy === 'auto' || oy === 'scroll') return p;
      p = p.parentElement;
    }
    return null;
  }

  function measure(): void {
    if (!root || !scrollParent) return;
    gridWidth = root.clientWidth;
    parentHeight = scrollParent.clientHeight;
    viewTop = scrollParent.getBoundingClientRect().top - root.getBoundingClientRect().top;
  }

  function onScroll(): void {
    measure();
    if (onLoadMore && !loadingMore && layout.totalHeight - (viewTop + parentHeight) < 400) {
      onLoadMore();
    }
  }

  onMount(() => {
    selection.clear();
    if (!root) return;
    scrollParent = findScrollParent(root);
    measure();
    const ro = new ResizeObserver(() => measure());
    ro.observe(root);
    if (scrollParent) {
      ro.observe(scrollParent);
      scrollParent.addEventListener('scroll', onScroll, { passive: true });
    }
    return () => {
      ro.disconnect();
      scrollParent?.removeEventListener('scroll', onScroll);
    };
  });

  $effect(() => {
    assets.length;
    measure();
  });
</script>

<div bind:this={root} class="relative" style:height="{layout.totalHeight}px">
  <div
    class="grid gap-1 absolute"
    style:top="{view.offsetY}px"
    style:left="{PAD}px"
    style:right="{PAD}px"
    style="grid-template-columns: repeat({layout.cols}, minmax(0, 1fr));"
  >
    {#each visibleAssets as asset (asset.id)}
      <AssetTile
        {asset}
        active={asset.id === activeId}
        selected={selection.has(asset.id)}
        selectionActive={selection.active}
        onToggle={() => selection.toggle(asset.id)}
        onRange={() => selection.range(assets.map((a) => a.id), asset.id)}
      />
    {/each}
  </div>
</div>
{#if loadingMore}
  <div class="py-4 text-center text-xs text-immich-dark-fg/30">loading…</div>
{/if}

<BulkActionBar {assets} />
