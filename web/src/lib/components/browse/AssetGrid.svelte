<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { afterNavigate } from '$app/navigation';
  import { observeSize } from '#lib/actions/observeSize.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import AssetTile from './AssetTile.svelte';
  import BulkActionBar from './bulk/BulkActionBar.svelte';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { hides } from '#lib/browse/loadedFilter.ts';
  import { trash } from '#lib/stores/trash.svelte.ts';
  import { isCopy } from '#lib/browse/assetKey.ts';
  import { browseTarget, setRating } from '#lib/browse/assetMeta.ts';
  import { gridCurrent, openMulti } from '#lib/browse/gridActions.ts';
  import { gridHost } from '#lib/commands/grid/gridHost.ts';
  import {
    createAssetGridLayout,
    verticalAssetIndex,
    visibleAssetRange,
    type AssetGridBox
  } from '#lib/browse/assetGridLayout.ts';

  let {
    assets,
    loadingMore = false,
    onLoadMore,
    onLoadAll
  }: {
    assets: AssetSummary[];
    loadingMore?: boolean;
    onLoadMore?: () => void;
    onLoadAll?: () => Promise<boolean>;
  } = $props();

  const GAP = 4;
  const PAD = 8;
  const OVERSCAN = 2;

  let root: HTMLDivElement | undefined = $state();
  let scrollParent: HTMLElement | null = null;
  let gridWidth = $state(0);
  let parentHeight = $state(0);
  let viewTop = $state(0);
  let scrollKey = '';
  let pendingRestore: number | null = null;
  let revealOnReturn = false;
  let loupeWasOpen = false;
  let shiftPressed = $state(false);
  let hoveredId = $state<string | null>(null);
  let selectingAll = $state(false);
  let bulkBar: BulkActionBar | undefined = $state();

  const items = $derived(assets.filter((a) => !hides(browseControls.filters, a)));

  const layout = $derived.by(() => {
    const inner = Math.max(0, gridWidth - PAD * 2);
    const geometry = createAssetGridLayout(items, inner, browseView.minTile, GAP);
    return { ...geometry, totalHeight: PAD * 2 + geometry.height };
  });

  const view = $derived.by(() => {
    return visibleAssetRange(layout, viewTop - PAD, viewTop + parentHeight - PAD, OVERSCAN);
  });

  const visibleAssets = $derived(
    items
      .slice(view.startIndex, view.endIndex)
      .map((asset, offset) => ({ asset, box: layout.boxes[view.startIndex + offset] }))
      .filter((item): item is { asset: AssetSummary; box: AssetGridBox } => item.box !== undefined)
  );

  const rangePreview = $derived.by(() => {
    if (selectingAll || !shiftPressed || !hoveredId) return null;
    return selection.rangeTarget(
      items.map((asset) => asset.id),
      hoveredId
    );
  });

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

  function onResize(): void {
    measure();
    restoreScroll();
  }

  function saveScroll(): void {
    if (!scrollParent || !scrollKey || pendingRestore !== null) return;
    browseView.setGridScroll(scrollKey, scrollParent.scrollTop);
  }

  function onScroll(): void {
    measure();
    saveScroll();
    if (onLoadMore && !loadingMore && layout.totalHeight - (viewTop + parentHeight) < 400) {
      onLoadMore();
    }
  }

  function restoreScroll(): void {
    if (!scrollParent || pendingRestore === null) return;
    const maxTop = Math.max(0, scrollParent.scrollHeight - scrollParent.clientHeight);
    if (pendingRestore > maxTop + 1) {
      if (onLoadMore && !loadingMore) onLoadMore();
      return;
    }
    scrollParent.scrollTop = Math.min(pendingRestore, maxTop);
    pendingRestore = null;
    measure();
  }

  function ensureVisible(id: string): void {
    const idx = items.findIndex((a) => a.id === id);
    if (idx < 0 || !scrollParent) return;
    const box = layout.boxes[idx];
    if (!box) return;
    const rowTop = PAD + box.top;
    const rowBottom = rowTop + box.height;
    if (rowTop < viewTop) {
      scrollParent.scrollTop -= viewTop - rowTop + GAP;
    } else if (rowBottom > viewTop + parentHeight) {
      scrollParent.scrollTop += rowBottom - (viewTop + parentHeight) + GAP;
    }
  }

  function centerIfHidden(box: AssetGridBox): void {
    const rowTop = PAD + box.top;
    if (rowTop + box.height > viewTop && rowTop < viewTop + parentHeight) return;
    const center = rowTop + box.height / 2 - parentHeight / 2;
    pendingRestore = Math.min(Math.max(0, center), Math.max(0, layout.totalHeight - parentHeight));
    restoreScroll();
  }

  function revealActive(): boolean {
    const id = browseView.activeId;
    if (!id) return true;
    const box = layout.boxes[items.findIndex((a) => a.id === id)];
    if (box) {
      centerIfHidden(box);
      return true;
    }
    if (!onLoadMore) return true;
    if (!loadingMore) onLoadMore();
    return false;
  }

  function moveIndex(key: string): number {
    if (key === 'Home') return 0;
    if (key === 'End') return items.length - 1;
    const cursor = selection.active ? gridCurrent(items) : browseView.activeId;
    const index = cursor ? items.findIndex((asset) => asset.id === cursor) : -1;
    if (index < 0 || !selection.active) return Math.max(0, index);
    if (key === 'ArrowRight') return index + 1;
    if (key === 'ArrowLeft') return index - 1;
    const rows = key.startsWith('Page')
      ? Math.max(1, Math.floor(parentHeight / browseView.minTile))
      : 1;
    return verticalAssetIndex(layout, index, key === 'ArrowUp' || key === 'PageUp' ? -rows : rows);
  }

  function move(key: string, extend: boolean): void {
    const target = items[Math.min(items.length - 1, Math.max(0, moveIndex(key)))];
    if (!target) return;
    if (extend && selection.active)
      selection.range(
        items.map((asset) => asset.id),
        target.id
      );
    else selection.selectLoaded([target.id]);
    browseView.setActive(target.id);
    ensureVisible(target.id);
  }

  async function selectAll(): Promise<boolean> {
    if (loadingMore || selectingAll) return false;
    selectingAll = true;
    try {
      if (onLoadAll && !(await onLoadAll())) return false;
      selection.selectLoaded(items.map((asset) => asset.id));
      return true;
    } finally {
      selectingAll = false;
    }
  }

  function trackShift(e: KeyboardEvent): void {
    shiftPressed = e.shiftKey;
  }

  function clearShiftPreview(): void {
    shiftPressed = false;
    hoveredId = null;
  }

  afterNavigate(() => {
    const path = `${window.location.pathname}${window.location.search}`;
    if (scrollKey && scrollKey !== path) {
      scrollKey = path;
      browseView.setLastGridPath(path);
    }
    restoreScroll();
  });

  onMount(() =>
    gridHost.attach({
      items: () => items,
      selectingAll: () => selectingAll,
      selectAll,
      move,
      reveal: ensureVisible,
      focusBulkBar: () => bulkBar?.focusFirst()
    })
  );

  onMount(() => {
    const path = `${window.location.pathname}${window.location.search}`;
    if (browseView.lastGridPath !== path) selection.clear();
    else revealOnReturn = true;
    if (!root) return;
    scrollKey = path;
    browseView.setLastGridPath(scrollKey);
    const savedTop = browseView.getGridScroll(scrollKey);
    pendingRestore = savedTop > 0 ? savedTop : null;
    scrollParent = findScrollParent(root);
    const parentResize = scrollParent ? observeSize(scrollParent, onResize) : undefined;
    if (scrollParent) {
      scrollParent.addEventListener('scroll', onScroll, { passive: true });
    }
    return () => {
      parentResize?.destroy?.();
      scrollParent?.removeEventListener('scroll', onScroll);
    };
  });

  $effect(() => {
    const _tracked = items.length;
    measure();
    restoreScroll();
    if (revealOnReturn && pendingRestore === null && items.length > 0)
      revealOnReturn = !untrack(revealActive);
  });

  $effect(() => {
    const open = browseView.loupeId !== null;
    if (loupeWasOpen && !open) untrack(revealActive);
    loupeWasOpen = open;
  });
</script>

<svelte:window onkeydown={trackShift} onkeyup={trackShift} onblur={clearShiftPreview} />

<div
  bind:this={root}
  use:observeSize={onResize}
  class="relative"
  style:height="{layout.totalHeight}px"
>
  {#each visibleAssets as item (item.asset.id)}
    <div
      class="absolute"
      style:top="{PAD + item.box.top}px"
      style:left="{PAD + item.box.left}px"
      style:width="{item.box.width}px"
      style:height="{item.box.height}px"
    >
      <AssetTile
        asset={item.asset}
        info={browseView.tileInfo}
        selected={selectingAll || selection.has(item.asset.id)}
        rangePreview={rangePreview?.has(item.asset.id) && !selection.has(item.asset.id)}
        selectionActive={selection.active || selectingAll}
        onToggle={() => {
          if (selectingAll) return;
          selection.toggle(item.asset.id);
          if (selection.has(item.asset.id)) browseView.setActive(item.asset.id);
        }}
        onPreview={() => (hoveredId = item.asset.id)}
        onPreviewEnd={() => {
          if (hoveredId === item.asset.id) hoveredId = null;
        }}
        onRange={() => {
          if (selectingAll) return;
          selection.range(
            items.map((a) => a.id),
            item.asset.id
          );
        }}
        onActivate={() => browseView.setActive(item.asset.id)}
        onLoupe={() => browseView.openLoupe(item.asset.id)}
        onRate={(rating) => void setRating(browseTarget(item.asset.id), rating)}
        onDeleteCopy={isCopy(item.asset.id)
          ? () => void trash.removeCopy(item.asset.id)
          : undefined}
      />
    </div>
  {/each}
</div>
{#if loadingMore}
  <div class="py-4 text-center text-xs text-muted" role="status">Loading more photos…</div>
{/if}

<BulkActionBar
  bind:this={bulkBar}
  assets={items}
  selectedIds={[...selection.selected]}
  onClear={selection.clear}
  onMulti={(mode) => openMulti(mode, items)}
  onSelectAll={selectAll}
  hasMore={onLoadMore !== undefined}
  {loadingMore}
  {selectingAll}
/>
