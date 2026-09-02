<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import type { AssetSummary } from '$lib/types/album';
  import AssetTile from './AssetTile.svelte';
  import BulkActionBar from './BulkActionBar.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { browseControls } from '$lib/stores/browseControls.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { compare } from '$lib/stores/compare.svelte';
  import { multiMembers, type MultiMode } from '$lib/compareEntry';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { deleteCopy } from '$lib/api/copies';
  import { isCopy } from '$lib/assetKey';
  import { rateAsset, toggleFavorite, toggleReject, clearFlags } from '$lib/cull';
  import { isRejected } from '$lib/reject';
  import { nextRatingFromKey } from '$lib/ratingShortcuts';
  import { matchKeybind, type KeybindContext } from '$lib/keybinds';

  const GRID_CONTEXTS: KeybindContext[] = ['grid', 'global'];

  let {
    assets,
    loadingMore = false,
    onLoadMore
  }: {
    assets: AssetSummary[];
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
  let scrollKey = '';
  let pendingRestore: number | null = null;
  let shiftPressed = $state(false);
  let hoveredId = $state<string | null>(null);

  const items = $derived(
    browseControls.excludeRejected ? assets.filter((a) => !isRejected(a)) : assets
  );

  const layout = $derived.by(() => {
    const inner = Math.max(0, gridWidth - PAD * 2);
    const cols = Math.max(1, Math.floor((inner + GAP) / (browseView.minTile + GAP)));
    const colWidth = (inner - (cols - 1) * GAP) / cols;
    const rowStride = colWidth + GAP;
    const rowCount = Math.ceil(items.length / cols);
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
      endIdx: Math.min(items.length, endRow * cols),
      offsetY: PAD + startRow * rowStride
    };
  });

  const visibleAssets = $derived(items.slice(view.startIdx, view.endIdx));

  const rangePreview = $derived.by(() => {
    if (!shiftPressed || !hoveredId || selection.allFiltered) return null;
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
    if (pendingRestore > maxTop + 1 && onLoadMore && !loadingMore) {
      onLoadMore();
      return;
    }
    scrollParent.scrollTop = Math.min(pendingRestore, maxTop);
    pendingRestore = null;
    measure();
  }

  function ensureVisible(id: string): void {
    const idx = items.findIndex((a) => a.id === id);
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

  function applyReject(): void {
    targets().forEach((id) => void toggleReject(id));
  }

  function applyUnflag(): void {
    targets().forEach((id) => void clearFlags(id));
  }

  function moveAndShow(e: KeyboardEvent, delta: number): void {
    e.preventDefault();
    const id = browseView.moveActive(delta);
    if (id) ensureVisible(id);
  }

  function openMulti(mode: MultiMode): void {
    const members = multiMembers(
      mode,
      items.map((a) => a.id),
      selection.selected,
      browseView.activeId
    );
    if (members.length < 2) {
      toasts.push('info', `${mode} needs two photos`);
      return;
    }
    browseView.openLoupe(members[0]);
    compare.enter(mode, members);
  }

  async function removeCopy(id: string): Promise<void> {
    try {
      await deleteCopy(id);
    } catch (e) {
      toasts.push('error', `delete copy: ${(e as Error).message}`);
      return;
    }
    if (selection.selected.has(id)) selection.toggle(id);
    if (browseView.activeId === id) browseView.setActive(null);
    browsing.remove(id);
    toasts.push('success', 'Virtual copy deleted');
  }

  function onKeydown(e: KeyboardEvent): void {
    shiftPressed = e.shiftKey;
    if (browseView.loupeId || isTyping()) return;

    const bind = matchKeybind(e, GRID_CONTEXTS);
    if (!bind) return;

    switch (bind) {
      case 'help':
        e.preventDefault();
        return ui.toggleKeybindsHelp();
      case 'gridClearSelection':
        if (!selection.active) return;
        e.preventDefault();
        return selection.clear();
      case 'gridSelectAll':
        e.preventDefault();
        return selection.selectLoaded(items.map((a) => a.id));
      case 'gridMove': {
        const { cols } = layout;
        if (e.key === 'ArrowRight') return moveAndShow(e, 1);
        if (e.key === 'ArrowLeft') return moveAndShow(e, -1);
        if (e.key === 'ArrowDown') return moveAndShow(e, cols);
        return moveAndShow(e, -cols);
      }
      case 'gridEdge':
        return moveAndShow(e, e.key === 'Home' ? -assets.length : assets.length);
      case 'gridPage': {
        const page = layout.cols * Math.max(1, Math.floor(parentHeight / layout.rowStride));
        return moveAndShow(e, e.key === 'PageUp' ? -page : page);
      }
      case 'gridSize':
        e.preventDefault();
        return browseView.stepGridSize(e.key === '-' || e.key === '_' ? -1 : 1);
      case 'favorite':
        e.preventDefault();
        return applyFavorite();
      case 'reject':
        e.preventDefault();
        return applyReject();
      case 'unflag':
        e.preventDefault();
        return applyUnflag();
      case 'enterCompare':
        e.preventDefault();
        return openMulti('compare');
      case 'enterSurvey':
        e.preventDefault();
        return openMulti('survey');
      case 'openEditor':
        if (!browseView.activeId) return;
        e.preventDefault();
        void goto(`/assets/${browseView.activeId}`);
        return;
      case 'openLoupe':
        if (!browseView.activeId) return;
        e.preventDefault();
        browseView.openLoupe(browseView.activeId);
        return;
      case 'rate': {
        const ids = targets();
        if (ids.length === 0) return;
        e.preventDefault();
        const idSet = new Set(ids);
        const ratings = items.filter((a) => idSet.has(a.id)).map((a) => a.exifInfo?.rating ?? null);
        const current = ratings.every((r) => r === ratings[0]) ? ratings[0] : undefined;
        const next = nextRatingFromKey(e.key, current);
        if (next !== undefined) applyRating(next);
        return;
      }
    }
  }

  function onKeyup(e: KeyboardEvent): void {
    shiftPressed = e.shiftKey;
  }

  function clearShiftPreview(): void {
    shiftPressed = false;
    hoveredId = null;
  }
  onMount(() => {
    selection.clear();
    if (!root) return;
    scrollKey = `${window.location.pathname}${window.location.search}`;
    browseView.setLastGridPath(scrollKey);
    const savedTop = browseView.getGridScroll(scrollKey);
    pendingRestore = savedTop > 0 ? savedTop : null;
    scrollParent = findScrollParent(root);
    measure();
    restoreScroll();
    const ro = new ResizeObserver(() => measure());
    ro.observe(root);
    window.addEventListener('keydown', onKeydown);
    window.addEventListener('keyup', onKeyup);
    window.addEventListener('blur', clearShiftPreview);
    if (scrollParent) {
      ro.observe(scrollParent);
      scrollParent.addEventListener('scroll', onScroll, { passive: true });
    }
    return () => {
      ro.disconnect();
      window.removeEventListener('keydown', onKeydown);
      window.removeEventListener('keyup', onKeyup);
      window.removeEventListener('blur', clearShiftPreview);
      scrollParent?.removeEventListener('scroll', onScroll);
    };
  });

  $effect(() => {
    const _tracked = items.length;
    measure();
    restoreScroll();
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
        active={asset.id === browseView.activeId}
        selected={selection.has(asset.id)}
      rangePreview={rangePreview?.has(asset.id) && !selection.has(asset.id)}
        selectionActive={selection.active}
        onToggle={() => selection.toggle(asset.id)}
      onPreview={() => (hoveredId = asset.id)}
        onPreviewEnd={() => {
        if (hoveredId === asset.id) hoveredId = null;
        }}
        onRange={() =>
          selection.range(
            items.map((a) => a.id),
            asset.id
          )}
        onActivate={() => browseView.setActive(asset.id)}
        onLoupe={() => browseView.openLoupe(asset.id)}
        onDeleteCopy={isCopy(asset.id) ? () => void removeCopy(asset.id) : undefined}
      />
    {/each}
  </div>
</div>
{#if loadingMore}
  <div class="py-4 text-center text-xs text-immich-dark-fg/30">loading…</div>
{/if}

<BulkActionBar {assets} onMulti={openMulti} />
