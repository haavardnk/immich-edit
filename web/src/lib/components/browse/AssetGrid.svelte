<script lang="ts">
  import { onMount } from 'svelte';
  import { observeSize } from '#lib/actions/observeSize.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import GridDayHeaders from '#lib/components/browse/GridDayHeaders.svelte';
  import GridSizeHud from '#lib/components/browse/GridSizeHud.svelte';
  import GridTiles from '#lib/components/browse/GridTiles.svelte';
  import BulkActionBar from '#lib/components/browse/bulk/BulkActionBar.svelte';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { hides } from '#lib/browse/loadedFilter.ts';
  import { openMulti } from '#lib/browse/gridActions.ts';
  import { gridHost } from '#lib/commands/grid/gridHost.ts';
  import { visibleAssetRange, type AssetGridBox } from '#lib/browse/assetGridLayout.ts';
  import { createDayGridLayout } from '#lib/browse/dayLayout.ts';
  import { groupDays } from '#lib/browse/dayGroups.ts';
  import { GridScroll } from '#lib/browse/gridScroll.svelte.ts';
  import { GridSelectAll } from '#lib/browse/gridSelectAll.svelte.ts';
  import { gridMover } from '#lib/browse/gridMove.ts';
  import { gridPinch, watchGridZoom } from '#lib/browse/gridPinch.ts';

  let {
    assets,
    loadingMore = false,
    loadingPrevious = false,
    dayHeaders = false,
    onLoadMore,
    onLoadPrevious,
    onLoadAll
  }: {
    assets: AssetSummary[];
    loadingMore?: boolean;
    loadingPrevious?: boolean;
    dayHeaders?: boolean;
    onLoadMore?: () => void;
    onLoadPrevious?: () => void;
    onLoadAll?: (signal: AbortSignal) => Promise<boolean>;
  } = $props();

  const GAP = 4;
  const PAD = 8;
  const OVERSCAN = 2;

  let root: HTMLDivElement | undefined = $state();
  let bulkBar: BulkActionBar | undefined = $state();

  const items = $derived(assets.filter((a) => !hides(browseControls.filters, a)));

  const days = $derived(dayHeaders ? groupDays(items) : null);

  const scroll = new GridScroll({
    items: () => items,
    layout: () => layout,
    pad: PAD,
    gap: GAP,
    loadMore: () => onLoadMore,
    loadPrevious: () => onLoadPrevious,
    loadingMore: () => loadingMore,
    loadingPrevious: () => loadingPrevious
  });

  const layout = $derived.by(() => {
    const geometry = createDayGridLayout(items, days, {
      width: Math.max(0, scroll.width - PAD * 2),
      rowHeight: browseView.rowHeight,
      rowTolerance: browseView.rowTolerance,
      gap: GAP,
      headerHeight: browseView.dayHeaderHeight
    });
    return { ...geometry, totalHeight: PAD * 2 + geometry.height };
  });

  const view = $derived(
    visibleAssetRange(layout, scroll.viewTop - PAD, scroll.viewTop + scroll.height - PAD, OVERSCAN)
  );

  const tiles = $derived(
    items
      .slice(view.startIndex, view.endIndex)
      .map((asset, offset) => ({ asset, box: layout.boxes[view.startIndex + offset] }))
      .filter((item): item is { asset: AssetSummary; box: AssetGridBox } => item.box !== undefined)
  );

  const selectAll = new GridSelectAll({
    items: () => items,
    loadingMore: () => loadingMore,
    loadAll: () => onLoadAll
  });

  const move = gridMover({
    items: () => items,
    layout: () => layout,
    viewHeight: () => scroll.height,
    reveal: scroll.ensureVisible
  });

  const pinchZoom = gridPinch(() => browseView.loupeId === null && !selectAll.busy);

  onMount(() =>
    gridHost.attach({
      items: () => items,
      selectingAll: () => selectAll.busy,
      selectAll: selectAll.run,
      cancelSelectAll: selectAll.cancel,
      move,
      reveal: scroll.ensureVisible,
      scrollTo: scroll.scrollToAsset,
      focusBulkBar: () => bulkBar?.focusFirst(),
      holdAnchor: scroll.holdAnchor
    })
  );

  onMount(() => {
    if (!root) return;
    const detach = scroll.attach(root);
    const scroller = scroll.parent;
    const stopZoom = scroller ? watchGridZoom(scroller) : undefined;
    return () => {
      stopZoom?.();
      detach();
    };
  });
</script>

{#if loadingPrevious}
  <div class="pointer-events-none sticky top-0 z-10 h-0">
    <div
      class="mx-auto mt-2 w-max rounded-full bg-light-50 px-3 py-1 text-xs text-dark/65 shadow-md"
      role="status"
    >
      Loading earlier photos…
    </div>
  </div>
{/if}
<div
  bind:this={root}
  use:observeSize={scroll.onResize}
  {@attach pinchZoom}
  class="relative"
  style:height="{layout.totalHeight}px"
>
  {#if days}
    <GridDayHeaders
      headers={layout.headers}
      {days}
      {items}
      startIndex={view.startIndex}
      endIndex={view.endIndex}
      pad={PAD}
      locked={selectAll.busy}
    />
  {/if}
  <GridTiles {items} {tiles} pad={PAD} selectingAll={selectAll.busy} />
</div>
{#if loadingMore}
  <div class="py-4 text-center text-xs text-muted" role="status">Loading more photos…</div>
{/if}
<GridSizeHud />

<BulkActionBar
  bind:this={bulkBar}
  assets={items}
  selectedIds={[...selection.selected]}
  onClear={selection.clear}
  onMulti={(mode) => openMulti(mode, items)}
  onSelectAll={selectAll.run}
  hasMore={onLoadMore !== undefined || onLoadPrevious !== undefined}
  {loadingMore}
  selectingAll={selectAll.busy}
/>
