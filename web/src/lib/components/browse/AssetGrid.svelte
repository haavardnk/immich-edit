<script lang="ts">
  import { onMount } from 'svelte';
  import { observeSize } from '#lib/actions/observeSize.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import GridDayHeaders from '#lib/components/browse/GridDayHeaders.svelte';
  import MarqueeBox from '#lib/components/browse/MarqueeBox.svelte';
  import GridSizeHud from '#lib/components/browse/GridSizeHud.svelte';
  import GridTiles from '#lib/components/browse/GridTiles.svelte';
  import RangeBubble from '#lib/components/browse/RangeBubble.svelte';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { hides } from '#lib/browse/loadedFilter.ts';
  import { gridHost } from '#lib/commands/grid/gridHost.ts';
  import { visibleAssetRange, type AssetGridBox } from '#lib/browse/assetGridLayout.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { createDayGridLayout } from '#lib/browse/dayLayout.ts';
  import { groupDays } from '#lib/browse/dayGroups.ts';
  import { GridMarquee } from '#lib/browse/gridMarquee.svelte.ts';
  import { GridScroll } from '#lib/browse/gridScroll.svelte.ts';
  import { GridSelectAll } from '#lib/browse/gridSelectAll.svelte.ts';
  import { gridMover } from '#lib/browse/gridMove.ts';
  import { gridPinch, watchGridZoom } from '#lib/browse/gridPinch.ts';
  import { TouchRange } from '#lib/browse/touchRange.svelte.ts';
  import { gestures } from '#lib/gestures/gestures.ts';

  let {
    assets,
    total,
    loadingMore = false,
    loadingPrevious = false,
    dayHeaders = false,
    onLoadMore,
    onLoadPrevious,
    onLoadAll
  }: {
    assets: AssetSummary[];
    total?: number;
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

  const idle = (): boolean => browseView.loupeId === null && !selectAll.busy;

  const marquee = new GridMarquee({
    items: () => items,
    layout: () => layout,
    pad: PAD,
    enabled: idle
  });

  const range = new TouchRange({
    items: () => items,
    layout: () => layout,
    pad: PAD,
    root: () => root ?? null,
    scroller: () => scroll.parent
  });

  const rangeBox = $derived.by(() => {
    const target = range.target;
    if (!target || range.added <= 0) return null;
    return layout.boxes[items.findIndex((asset) => asset.id === target)] ?? null;
  });

  const touchGestures = gestures({
    touchAction: 'pan-y',
    pointerTypes: ['touch'],
    recognizers: [gridPinch(idle), range.recognizer(idle)]
  });

  onMount(() =>
    gridHost.attach({
      items: () => items,
      selectingAll: () => selectAll.busy,
      selectAll: selectAll.run,
      cancelSelectAll: selectAll.cancel,
      move,
      reveal: scroll.ensureVisible,
      scrollTo: scroll.scrollToAsset,
      holdAnchor: scroll.holdAnchor,
      dragging: () => marquee.dragging || range.dragging,
      cancelDrag: () => {
        marquee.cancel();
        range.cancel();
      }
    })
  );

  onMount(() =>
    bulkScope.attach({
      items: () => items,
      selectingAll: () => selectAll.busy,
      selectAll: selectAll.run,
      cancelSelectAll: selectAll.cancel,
      partial: () => onLoadMore !== undefined || onLoadPrevious !== undefined,
      loadingMore: () => loadingMore,
      total: () => total,
      preview: () => range.preview
    })
  );

  onMount(() => {
    if (!root) return;
    const detach = scroll.attach(root);
    const scroller = scroll.parent;
    const stopZoom = scroller ? watchGridZoom(scroller) : undefined;
    const stopMarquee = scroller ? marquee.attach(root, scroller) : undefined;
    return () => {
      stopMarquee?.();
      stopZoom?.();
      range.cancel();
      selection.exitMode();
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
  {@attach touchGestures}
  {@attach range.guard}
  class="relative select-none"
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
  <GridTiles {items} {tiles} pad={PAD} selectingAll={selectAll.busy} dragPreview={range.preview} />
  <MarqueeBox {marquee} />
  {#if rangeBox}
    <RangeBubble box={rangeBox} pad={PAD} added={range.added} />
  {/if}
</div>
{#if loadingMore}
  <div class="py-4 text-center text-xs text-muted" role="status">Loading more photos…</div>
{/if}
<GridSizeHud />
