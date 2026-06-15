<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import type { AssetSummary } from '$lib/types/album';
  import AssetTile from './AssetTile.svelte';
  import BulkActionBar from './BulkActionBar.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { rateAsset, toggleFavorite } from '$lib/cull';
  import { nextRatingFromKey } from '$lib/ratingShortcuts';

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

  const GAP = 4;
  const PAD = 8;
  const OVERSCAN = 2;

  let root: HTMLDivElement | undefined = $state();
  let scrollParent: HTMLElement | null = null;
  let gridWidth = $state(0);
  let parentHeight = $state(0);
  let viewTop = $state(0);

  const effectiveActive = $derived(activeId ?? browseView.activeId);

  const layout = $derived.by(() => {
    const inner = Math.max(0, gridWidth - PAD * 2);
    const cols = Math.max(1, Math.floor((inner + GAP) / (browseView.minTile + GAP)));
    const colWidth = (inner - (cols - 1) * GAP) / cols;
    const rowStride = colWidth + GAP;
    const rowCount = Math.ceil(assets.length / cols);
    const totalHeight = PAD * 2 + rowCount * colWidth + Math.max(0, rowCount - 1) * GAP;
    return { cols, colWidth, rowStride, rowCount, totalHeight };
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

  function ensureVisible(id: string): void {
    const idx = assets.findIndex((a) => a.id === id);
    if (idx < 0 || !scrollParent) return;
    const { cols, rowStride, colWidth } = layout;
    const rowTop = PAD + Math.floor(idx / cols) * rowStride;
    const rowBottom = rowTop + colWidth;
    if (rowTop < viewTop) {
      scrollParent.scrollTop -= viewTop - rowTop + GAP;
    } else if (rowBottom > viewTop + parentHeight) {
      scrollParent.scrollTop += rowBottom - (viewTop + parentHeight) + GAP;
    }
  }

  function isTyping(): boolean {
    const el = document.activeElement;
    if (!el) return false;
    const tag = el.tagName;
    return (
      tag === 'INPUT' ||
      tag === 'TEXTAREA' ||
      tag === 'SELECT' ||
      (el as HTMLElement).isContentEditable
    );
  }

  function targets(): string[] {
    if (selection.active) return [...selection.selected];
    return browseView.activeId ? [browseView.activeId] : [];
  }

  function applyRating(rating: number | null): void {
    const ids = targets();
    if (ids.length === 0) return;
    ids.forEach((id) => void rateAsset(id, rating));
  }

  function applyFavorite(): void {
    targets().forEach((id) => void toggleFavorite(id));
  }

  function onKeydown(e: KeyboardEvent): void {
    if (browseView.loupeId || isTyping()) return;

    if (e.key === '?' || (e.key === '/' && e.shiftKey)) {
      e.preventDefault();
      ui.toggleKeybindsHelp();
      return;
    }

    if (e.key === 'Escape' && selection.active) {
      e.preventDefault();
      selection.clear();
      return;
    }

    if (e.metaKey || e.ctrlKey || e.altKey) return;

    const { cols } = layout;
    const moveAndShow = (delta: number): void => {
      e.preventDefault();
      const id = browseView.moveActive(delta);
      if (id) ensureVisible(id);
    };

    switch (e.key) {
      case 'ArrowRight':
        return moveAndShow(1);
      case 'ArrowLeft':
        return moveAndShow(-1);
      case 'ArrowDown':
        return moveAndShow(cols);
      case 'ArrowUp':
        return moveAndShow(-cols);
      case 'Home':
        return moveAndShow(-assets.length);
      case 'End':
        return moveAndShow(assets.length);
      case 'PageDown':
        return moveAndShow(cols * Math.max(1, Math.floor(parentHeight / layout.rowStride)));
      case 'PageUp':
        return moveAndShow(-cols * Math.max(1, Math.floor(parentHeight / layout.rowStride)));
      case '-':
      case '_':
        e.preventDefault();
        return browseView.stepGridSize(-1);
      case '+':
      case '=':
        e.preventDefault();
        return browseView.stepGridSize(1);
      case 'f':
      case 'F':
        e.preventDefault();
        return applyFavorite();
      case 'Enter':
        if (browseView.activeId) {
          e.preventDefault();
          void goto(`/assets/${browseView.activeId}`);
        }
        return;
      case ' ':
        if (browseView.activeId) {
          e.preventDefault();
          browseView.openLoupe(browseView.activeId);
        }
        return;
    }

    if (e.key === '0' || (e.key >= '1' && e.key <= '5')) {
      const ids = targets();
      if (ids.length === 0) return;
      e.preventDefault();
      const idSet = new Set(ids);
      const ratings = assets.filter((a) => idSet.has(a.id)).map((a) => a.exifInfo?.rating ?? null);
      const current = ratings.every((r) => r === ratings[0]) ? ratings[0] : undefined;
      const next = nextRatingFromKey(e.key, current);
      if (next !== undefined) applyRating(next);
    }
  }

  onMount(() => {
    selection.clear();
    if (!root) return;
    scrollParent = findScrollParent(root);
    measure();
    const ro = new ResizeObserver(() => measure());
    ro.observe(root);
    window.addEventListener('keydown', onKeydown);
    if (scrollParent) {
      ro.observe(scrollParent);
      scrollParent.addEventListener('scroll', onScroll, { passive: true });
    }
    return () => {
      ro.disconnect();
      window.removeEventListener('keydown', onKeydown);
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
        active={asset.id === effectiveActive}
        selected={selection.has(asset.id)}
        selectionActive={selection.active}
        onToggle={() => selection.toggle(asset.id)}
        onRange={() => selection.range(assets.map((a) => a.id), asset.id)}
        onActivate={() => browseView.setActive(asset.id)}
        onLoupe={() => browseView.openLoupe(asset.id)}
      />
    {/each}
  </div>
</div>
{#if loadingMore}
  <div class="py-4 text-center text-xs text-immich-dark-fg/30">loading…</div>
{/if}

<BulkActionBar {assets} />
