<script lang="ts">
  import { onMount, untrack } from 'svelte';
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
  import { rateAsset, toggleFavorite, toggleReject, clearFlags, setLabel } from '$lib/cull';
  import { labelOf, nextLabelFromKey } from '$lib/labels';
  import { nextRatingFromKey, ratingFromCode } from '$lib/ratingShortcuts';
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

  const items = $derived(assets.filter((a) => !browseControls.hides(a)));

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
    return [...selection.selected];
  }

  function current(): string | null {
    const id = browseView.activeId;
    if (id && selection.has(id)) return id;
    return items.find((asset) => selection.has(asset.id))?.id ?? null;
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

  function applyLabelKey(key: string): boolean {
    const ids = targets();
    if (ids.length === 0) return false;
    const idSet = new Set(ids);
    const labels = items.filter((a) => idSet.has(a.id)).map((a) => labelOf(a));
    const common = labels.every((l) => l === labels[0]) ? (labels[0] ?? null) : null;
    const next = nextLabelFromKey(key, common);
    if (next === undefined) return false;
    ids.forEach((id) => void setLabel(id, next));
    return true;
  }

  function moveIndex(key: string): number {
    if (key === 'Home') return 0;
    if (key === 'End') return items.length - 1;
    const cursor = selection.active ? current() : browseView.activeId;
    const index = cursor ? items.findIndex((asset) => asset.id === cursor) : -1;
    if (index < 0 || !selection.active) return Math.max(0, index);
    if (key === 'ArrowRight') return index + 1;
    if (key === 'ArrowLeft') return index - 1;
    const rows = key.startsWith('Page')
      ? Math.max(1, Math.floor(parentHeight / browseView.minTile))
      : 1;
    return verticalAssetIndex(layout, index, key === 'ArrowUp' || key === 'PageUp' ? -rows : rows);
  }

  function selectMove(e: KeyboardEvent): void {
    e.preventDefault();
    const target = items[Math.min(items.length - 1, Math.max(0, moveIndex(e.key)))];
    if (!target) return;
    if (e.shiftKey && selection.active)
      selection.range(
        items.map((asset) => asset.id),
        target.id
      );
    else selection.selectLoaded([target.id]);
    browseView.setActive(target.id);
    ensureVisible(target.id);
  }

  function openMulti(mode: MultiMode): void {
    const members = multiMembers(
      mode,
      items.map((a) => a.id),
      selection.selected,
      current()
    );
    const first = members[0];
    if (members.length < 2 || !first) {
      toasts.push('info', `${mode} needs two photos`);
      return;
    }
    browseView.openLoupe(first);
    compare.enter(mode, members);
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

  async function removeCopy(id: string): Promise<void> {
    try {
      await deleteCopy(id);
    } catch (e) {
      toasts.fail('delete copy', e);
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
    if (selectingAll) {
      e.preventDefault();
      return;
    }

    switch (bind) {
      case 'help':
        e.preventDefault();
        return ui.toggleKeybindsHelp();
      case 'gridClearSelection':
        if (!selection.active) return;
        e.preventDefault();
        return selection.clear();
      case 'focusBulkBar':
        if (!selection.active) return;
        e.preventDefault();
        return bulkBar?.focusFirst();
      case 'gridSelectAll':
        e.preventDefault();
        void selectAll();
        return;
      case 'gridMove':
      case 'gridEdge':
      case 'gridPage':
      case 'gridExtend':
        return selectMove(e);
      case 'gridSize':
        e.preventDefault();
        return browseView.stepGridSize(e.key === '-' || e.key === '_' ? -1 : 1);
      case 'gridTileInfo':
        e.preventDefault();
        return browseView.cycleTileInfo();
      case 'favorite':
        e.preventDefault();
        return applyFavorite();
      case 'reject':
        e.preventDefault();
        return applyReject();
      case 'unflag':
        e.preventDefault();
        return applyUnflag();
      case 'label':
        if (applyLabelKey(e.key)) e.preventDefault();
        return;
      case 'enterCompare':
        e.preventDefault();
        return openMulti('compare');
      case 'enterSurvey':
        e.preventDefault();
        return openMulti('survey');
      case 'openEditor': {
        const id = current();
        if (!id) return;
        e.preventDefault();
        void goto(editorHref(id, `${window.location.pathname}${window.location.search}`));
        return;
      }
      case 'openLoupe': {
        const id = current();
        if (!id) return;
        e.preventDefault();
        browseView.openLoupe(id);
        return;
      }
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
      case 'rateAdvance': {
        const next = ratingFromCode(e.code);
        const ids = targets();
        if (next === undefined || ids.length === 0) return;
        e.preventDefault();
        const [only] = ids;
        if (ids.length > 1 || !only) return applyRating(next);
        const following = items[items.findIndex((asset) => asset.id === only) + 1];
        void rateAsset(only, next).then((ok) => {
          if (!ok || !following) return;
          selection.selectLoaded([following.id]);
          browseView.setActive(following.id);
          ensureVisible(following.id);
        });
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

  afterNavigate(() => {
    restoreScroll();
  });

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
    window.addEventListener('keydown', onKeydown);
    window.addEventListener('keyup', onKeyup);
    window.addEventListener('blur', clearShiftPreview);
    if (scrollParent) {
      scrollParent.addEventListener('scroll', onScroll, { passive: true });
    }
    return () => {
      parentResize?.destroy?.();
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
    if (revealOnReturn && pendingRestore === null && items.length > 0)
      revealOnReturn = !untrack(revealActive);
  });

  $effect(() => {
    const open = browseView.loupeId !== null;
    if (loupeWasOpen && !open) untrack(revealActive);
    loupeWasOpen = open;
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
        onRate={(rating) => void rateAsset(item.asset.id, rating)}
        onDeleteCopy={isCopy(item.asset.id) ? () => void removeCopy(item.asset.id) : undefined}
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
  onMulti={openMulti}
  onSelectAll={selectAll}
  hasMore={onLoadMore !== undefined}
  {loadingMore}
  {selectingAll}
/>
