<script lang="ts">
  import { onDestroy, onMount, untrack, type Snippet } from 'svelte';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import {
    canDropPane,
    dropFocusedPane,
    loupeAsset,
    loupeCullActions,
    loupeFocusedId,
    loupeTagActions
  } from '#lib/browse/loupeActions.ts';
  import { loupeHost } from '#lib/commands/loupe/loupeHost.ts';
  import { persistedPreviewUrl } from '#lib/api/preview.ts';
  import { cullState } from '#lib/browse/cullState.ts';
  import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
  import { paneGridStyle, sheetInsetStyle } from '#lib/browse/loupeLayout.ts';
  import { ImagePreloader, preloadUrls, type StepFn } from '#lib/browse/loupePreload.ts';
  import { loupeTags } from '#lib/stores/loupeTags.svelte.ts';
  import { ratingHud } from '#lib/browse/ratingHud.svelte.ts';
  import { putBounded } from '#lib/utils/boundedRecord.ts';
  import { loadFaceData } from '#lib/stores/zoomTargets.ts';
  import { ZoomCycle } from '#lib/stores/zoomCycle.ts';
  import {
    FIT_VIEW,
    isZoomed,
    panByStep,
    withZoom,
    type View,
    type ViewportMetrics
  } from '#lib/utils/imageViewport.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import LoupeActionRail from '#lib/components/browse/LoupeActionRail.svelte';
  import LoupePaneGrid from '#lib/components/browse/LoupePaneGrid.svelte';
  import LoupeToolbar from '#lib/components/browse/LoupeToolbar.svelte';
  import RatingHud from '#lib/components/browse/RatingHud.svelte';
  import LoupeInfo from '#lib/components/browse/info/LoupeInfo.svelte';
  import CompactLoupeBar from '#lib/components/browse/CompactLoupeBar.svelte';
  import LoupeCullBar from '#lib/components/browse/LoupeCullBar.svelte';
  import LoupeEditPill from '#lib/components/browse/LoupeEditPill.svelte';
  import LoupeFilmstrip from '#lib/components/browse/LoupeFilmstrip.svelte';
  import LoupePaneControls from '#lib/components/browse/LoupePaneControls.svelte';
  import LoupeStepButtons from '#lib/components/browse/LoupeStepButtons.svelte';
  import LoupeViewTools from '#lib/components/browse/LoupeViewTools.svelte';
  import WideLoupeBar from '#lib/components/browse/WideLoupeBar.svelte';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';
  import { mdiFullscreenExit } from '@mdi/js';

  const MAX_EDGE = 2560;

  let paneMaxEdge = $state(MAX_EDGE);

  const currentId = $derived(browseView.loupeId);
  const multi = $derived(compare.mode !== 'single');
  const panes = $derived(multi ? compare.members : currentId ? [currentId] : []);
  const focusedId = $derived(loupeFocusedId());
  const asset = $derived(loupeAsset());
  const copyBadge = $derived(
    asset && isCopy(asset.id) ? (asset.copyLabel ?? `Copy ${copyIndex(asset.id)}`) : null
  );
  const paneView = $derived(focusedId ? compare.viewOf(focusedId) : FIT_VIEW);
  let belowPanes = $state(0);
  const gridStyle = $derived(
    paneGridStyle(multi, panes.length) +
      (browseView.loupeInfoSheet && !ui.fullscreen ? sheetInsetStyle(belowPanes) : '')
  );
  const touch = $derived(layout.mode === 'touch');
  const moreActive = $derived(ui.clipWarn || (touch && browseView.loupeAutoAdvance));
  const canDrop = $derived(canDropPane());

  let paneMetrics = $state.raw<Record<string, ViewportMetrics>>({});
  let paneImages = $state<Record<string, HTMLImageElement>>({});
  let paneImageOrder = $state<string[]>([]);
  const currentTags = $derived(loupeTags.tagsOf(focusedId));
  const focusedFitZoom = $derived((focusedId ? paneMetrics[focusedId]?.fitZoom : null) ?? 100);
  const zoom = $derived(paneView.zoom ?? focusedFitZoom);
  const zoomCycle = new ZoomCycle({
    shown: () => focusedId,
    transform: (data) => data.savedView,
    image: (id) => paneImages[id] ?? null,
    metrics: (id) => paneMetrics[id] ?? null,
    zoomLevel: () => ui.zoomLevel,
    view: (id) => compare.viewOf(id),
    apply: (id, view) => compare.applyView(id, view)
  });

  const preloader = new ImagePreloader();
  const stepShown: StepFn = (id, direction) =>
    direction > 0 ? browsing.nextOf(id) : browsing.prevOf(id);

  onDestroy(() => {
    ui.fullscreen = false;
    preloader.keep([]);
    ratingHud.clear();
  });

  onMount(() =>
    loupeHost.attach({
      cycleZoom: () => zoomCycle.next(),
      panBy: (dx, dy) => {
        const id = focusedId;
        const metrics = id ? paneMetrics[id] : null;
        if (!id || !metrics) return;
        const view = compare.viewOf(id);
        if (isZoomed(metrics, view)) compare.applyView(id, panByStep(metrics, view, [dx, dy]));
      }
    })
  );

  $effect(() => {
    const id = focusedId;
    zoomCycle.reset();
    if (id) void loadFaceData(id);
  });

  $effect(() => {
    const id = focusedId;
    if (id) void loupeTags.ensure(id);
  });

  $effect(() => {
    const id = focusedId;
    untrack(() => ratingHud.follow(id));
  });

  $effect(() => {
    const id = multi ? null : currentId;
    const render = (neighbour: string) => persistedPreviewUrl(neighbour, paneMaxEdge, ui.clipWarn);
    preloader.keep(id ? preloadUrls(browsing.find(id) ?? { id }, stepShown, render) : []);
  });

  $effect(() => {
    const id = focusedId;
    if (id) untrack(() => browsing.prefetchNear(id));
  });

  let scrim = $state<HTMLDivElement | null>(null);

  function setZoom(value: number): void {
    const id = focusedId;
    if (!id) return;
    const next = withZoom(compare.viewOf(id), value, focusedFitZoom);
    compare.applyView(id, next);
    ui.rememberZoomLevel(next.zoom, focusedFitZoom);
  }

  function fitZoom(): void {
    const id = focusedId;
    if (!id) return;
    compare.applyView(id, FIT_VIEW);
  }

  function applyPaneView(id: string, view: View, solo?: boolean): void {
    compare.applyView(id, view, solo);
  }

  function setPaneMetrics(id: string, metrics: ViewportMetrics): void {
    paneMetrics = { ...paneMetrics, [id]: metrics };
  }

  function setPaneImage(id: string, element: HTMLImageElement): void {
    if (paneImages[id] === element) return;
    const next = putBounded(paneImages, paneImageOrder, id, element, 9);
    paneImages = next.record;
    paneImageOrder = next.order;
  }
</script>

{#snippet viewTools()}
  {#if multi}
    <LoupePaneControls {canDrop} onDrop={dropFocusedPane} />
  {/if}
  <LoupeViewTools
    {zoom}
    fitZoom={focusedFitZoom}
    fitMode={paneView.zoom === null}
    onZoom={setZoom}
    onFit={fitZoom}
  />
{/snippet}

{#snippet editPill()}
  <LoupeEditPill />
{/snippet}

{#snippet cullBar(density: 'stacked' | 'row', trailing: Snippet)}
  {#if asset}
    <LoupeCullBar
      cull={cullState(asset)}
      actions={loupeCullActions}
      tags={currentTags}
      tagActions={loupeTagActions}
      fileName={asset.originalFileName}
      {density}
      {trailing}
    />
  {/if}
{/snippet}

{#if asset}
  <div
    class="fixed inset-0 z-40 flex min-w-0 flex-col {layout.compact
      ? 'pr-(--safe-right) pl-(--safe-left)'
      : 'safe-frame'}"
  >
    <div
      bind:this={scrim}
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 -z-10 bg-image-canvas"
    ></div>
    {#if !ui.fullscreen && !layout.compact}
      {#if touch}
        <WideLoupeBar filename={asset.originalFileName} {copyBadge} {moreActive} />
      {:else}
        <LoupeToolbar filename={asset.originalFileName} {copyBadge} {multi} {moreActive} />
      {/if}
    {/if}

    <div class="flex min-h-0 flex-1">
      <div class="flex min-w-0 flex-1 flex-col">
        <div
          class="flex-1 min-h-0 relative {multi ? 'grid gap-1.5 bg-neutral-950 p-1.5' : 'flex'}"
          style={gridStyle}
        >
          <LoupePaneGrid
            {panes}
            {focusedId}
            {multi}
            {canDrop}
            scrim={() => scrim}
            onView={applyPaneView}
            onSize={(size) => (paneMaxEdge = size)}
            onMetrics={setPaneMetrics}
            onImage={setPaneImage}
          />

          <RatingHud />

          {#if !ui.fullscreen}
            {#if layout.compact}
              <CompactLoupeBar filename={asset.originalFileName} {copyBadge} {moreActive} />
            {/if}
            <LoupeStepButtons {currentId} {multi} />
          {:else}
            <IconButton
              size="small"
              variant="filled"
              color="primary"
              shape="round"
              class="fixed top-[calc(0.75rem+var(--safe-top))] right-[calc(0.75rem+var(--safe-right))] z-40 shadow-lg"
              icon={mdiFullscreenExit}
              title={hint('Exit fullscreen', 'fullscreen')}
              aria-label={hint('Exit fullscreen', 'fullscreen')}
              onclick={ui.toggleFullscreen}
            />
          {/if}
        </div>

        {#if !ui.fullscreen}
          <div class="flex shrink-0 flex-col" bind:offsetHeight={belowPanes}>
            {#if !touch}
              <LoupeActionRail
                cull={cullState(asset)}
                tags={currentTags}
                fileName={asset.originalFileName}
                {multi}
                {zoom}
                fitZoom={focusedFitZoom}
                fitMode={paneView.zoom === null}
                actions={loupeCullActions}
                onAddTag={loupeTagActions.add}
                onRemoveTag={loupeTagActions.remove}
                onCreateTag={loupeTagActions.create}
                onZoom={setZoom}
                onFit={fitZoom}
                {canDrop}
                onDrop={dropFocusedPane}
              />
              <LoupeFilmstrip {focusedId} {currentId} {multi} />
            {:else if layout.compact}
              {#if browseView.loupeFilmstripShown}
                <LoupeFilmstrip {focusedId} {currentId} {multi} />
              {/if}
              {@render cullBar('stacked', editPill)}
            {:else}
              {@render cullBar('row', viewTools)}
              <LoupeFilmstrip {focusedId} {currentId} {multi} />
            {/if}
          </div>
        {/if}
      </div>
      {#if !ui.fullscreen}
        <LoupeInfo {asset} />
      {/if}
    </div>
  </div>
{/if}
