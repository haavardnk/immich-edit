<script lang="ts">
  import { onMount } from 'svelte';
  import { afterNavigate, goto } from '$app/navigation';
  import { observeSize } from '$lib/actions/observeSize';
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
  import { editorHref } from '$lib/editorNavigation';
  import { matchKeybind, type KeybindContext } from '$lib/keybinds';
  import {
    createAssetGridLayout,
    verticalAssetIndex,
    visibleAssetRange,
    type AssetGridBox
  } from '$lib/assetGridLayout';

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

  const items = $derived(
    browseControls.excludeRejected ? assets.filter((a) => !isRejected(a)) : assets
  );

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

  function moveAndShow(e: KeyboardEvent, index: number): void {
    e.preventDefault();
    const target = items[Math.min(items.length - 1, Math.max(0, index))];
    if (!target) return;
    const id = target.id;
    browseView.setActive(id);
    if (id) ensureVisible(id);
  }

  function activeIndex(): number {
    return browseView.activeId ? items.findIndex((asset) => asset.id === browseView.activeId) : -1;
  }

  function openMulti(mode: MultiMode): void {
    const members = multiMembers(
      mode,
      items.map((a) => a.id),
      selection.selected,
      browseView.activeId
    );
    const first = members[0];
    if (members.length < 2 || !first) {
      toasts.push('info', `${mode} needs two photos`);
      return;
    }
    browseView.openLoupe(first);
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
        const current = activeIndex();
        if (current < 0) return moveAndShow(e, 0);
        if (e.key === 'ArrowRight') return moveAndShow(e, current + 1);
        if (e.key === 'ArrowLeft') return moveAndShow(e, current - 1);
        return moveAndShow(e, verticalAssetIndex(layout, current, e.key === 'ArrowDown' ? 1 : -1));
      }
      case 'gridEdge':
        return moveAndShow(e, e.key === 'Home' ? 0 : items.length - 1);
      case 'gridPage': {
        const current = activeIndex();
        if (current < 0) return moveAndShow(e, 0);
        const rows = Math.max(1, Math.floor(parentHeight / browseView.minTile));
        return moveAndShow(
          e,
          verticalAssetIndex(layout, current, e.key === 'PageUp' ? -rows : rows)
        );
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
        void goto(
          editorHref(browseView.activeId, `${window.location.pathname}${window.location.search}`)
        );
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

  afterNavigate(() => {
    restoreScroll();
  });

  onMount(() => {
    const path = `${window.location.pathname}${window.location.search}`;
    if (browseView.lastGridPath !== path) selection.clear();
    if (!root) return;
    scrollKey = path;
    browseView.setLastGridPath(scrollKey);
    const savedTop = browseView.getGridScroll(scrollKey);
    pendingRestore = savedTop > 0 ? savedTop : null;
    scrollParent = findScrollParent(root);
    const parentResize = scrollParent ? observeSize(scrollParent, onResize) : undefined;
    window.addEventListener('keydown', onKeydown);
    if (scrollParent) {
      scrollParent.addEventListener('scroll', onScroll, { passive: true });
    }
    return () => {
      parentResize?.destroy?.();
      window.removeEventListener('keydown', onKeydown);
      scrollParent?.removeEventListener('scroll', onScroll);
    };
  });

  $effect(() => {
    const _tracked = items.length;
    measure();
    restoreScroll();
  });
</script>

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
        active={item.asset.id === browseView.activeId}
        selected={selection.has(item.asset.id)}
        selectionActive={selection.active}
        onToggle={() => selection.toggle(item.asset.id)}
        onRange={() =>
          selection.range(
            items.map((a) => a.id),
            item.asset.id
          )}
        onActivate={() => browseView.setActive(item.asset.id)}
        onLoupe={() => browseView.openLoupe(item.asset.id)}
        onDeleteCopy={isCopy(item.asset.id) ? () => void removeCopy(item.asset.id) : undefined}
      />
    </div>
  {/each}
</div>
{#if loadingMore}
  <div class="py-4 text-center text-xs text-muted" role="status">Loading more photos…</div>
{/if}

<BulkActionBar {assets} onMulti={openMulti} />
