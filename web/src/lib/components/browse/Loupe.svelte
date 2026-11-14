<script lang="ts">
  import { onDestroy, onMount, untrack, type Snippet } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import {
    loupeAsset,
    loupeCullActions,
    loupeFocusedId,
    loupeTagActions
  } from '#lib/browse/loupeActions.ts';
  import { loupeHost } from '#lib/commands/loupe/loupeHost.ts';
  import { persistedPreviewUrl } from '#lib/api/preview.ts';
  import { cullState } from '#lib/browse/cullState.ts';
  import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
  import { INFO_SHEET_INSET, paneGridStyle, paneWindow } from '#lib/browse/loupeLayout.ts';
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
  import { zoomIn } from '#lib/browse/loupeZoom.ts';
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
  import LoupePaneRail from '#lib/components/browse/LoupePaneRail.svelte';
  import LoupeStepButtons from '#lib/components/browse/LoupeStepButtons.svelte';
  import LoupeViewTools from '#lib/components/browse/LoupeViewTools.svelte';
  import WideLoupeBar from '#lib/components/browse/WideLoupeBar.svelte';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { paneDrop, panePromote } from '#lib/commands/loupe/panes.ts';
  import { IconButton } from '@immich/ui';
  import { mdiFullscreenExit } from '@mdi/js';

  const MAX_EDGE = 2560;

  let paneMaxEdge = $state(MAX_EDGE);

  const currentId = $derived(browseView.loupeId);
  const multi = $derived(compare.mode !== 'single');
  const shown = $derived(paneWindow(compare.members.length, compare.focusIndex, compare.paneLimit));
  const panes = $derived(
    multi ? compare.members.slice(shown.start, shown.end) : currentId ? [currentId] : []
  );
  const focusedId = $derived(loupeFocusedId());
  const asset = $derived(loupeAsset());
  const copyBadge = $derived(
    asset && isCopy(asset.id) ? (asset.copyLabel ?? `Copy ${copyIndex(asset.id)}`) : null
  );
  const paneView = $derived(focusedId ? compare.viewOf(focusedId) : FIT_VIEW);
  const touch = $derived(layout.mode === 'touch');
  const stacked = $derived(layout.compact && !layout.landscape);
  const underBar = $derived(layout.compact && !ui.fullscreen);
  const paneArea = $derived(
    `${multi ? 'grid gap-1.5 bg-neutral-950 p-1.5' : 'flex'} ${underBar ? 'pt-[calc(3.5rem+var(--safe-top))]' : ''}`
  );
  const gridStyle = $derived(
    paneGridStyle(multi, panes.length, stacked) +
      (browseView.loupeInfoSheet && !ui.fullscreen ? INFO_SHEET_INSET : '')
  );
  const moreActive = $derived(ui.clipWarn || (touch && browseView.loupeAutoAdvance));

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

  const enter: Attachment<HTMLElement> = (root) => untrack(() => zoomIn(currentId, root));

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
    <LoupePaneControls variant="pill" />
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
      : 'safe-frame'} {touch ? '[--color-editor-chrome:var(--color-light)]' : ''}"
    {@attach enter}
  >
    <div
      bind:this={scrim}
      data-loupe-scrim
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
        <div class="relative min-h-0 flex-1 {paneArea}" style={gridStyle}>
          <LoupePaneGrid
            {panes}
            {focusedId}
            first={multi ? shown.start : 0}
            {multi}
            paneActions={touch ? [panePromote] : [panePromote, paneDrop]}
            pills={touch}
            scrim={() => scrim}
            onView={applyPaneView}
            onSize={(size) => (paneMaxEdge = size)}
            onMetrics={setPaneMetrics}
            onImage={setPaneImage}
          />

          <RatingHud />

          {#if !ui.fullscreen}
            {#if layout.compact}
              <CompactLoupeBar filename={asset.originalFileName} {copyBadge} {moreActive} {multi} />
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

        {#if !ui.fullscreen && !browseView.loupeInfoSheet}
          <div class="flex shrink-0 flex-col">
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
              />
              <LoupeFilmstrip {focusedId} {currentId} {multi} />
            {:else if layout.compact}
              {#if multi}
                <LoupePaneRail fileName={asset.originalFileName} />
              {/if}
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
