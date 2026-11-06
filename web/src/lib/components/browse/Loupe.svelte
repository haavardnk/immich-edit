<script lang="ts">
  import { onDestroy, onMount, untrack } from 'svelte';
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
    openInEditor
  } from '#lib/browse/loupeActions.ts';
  import { loupeHost } from '#lib/commands/loupe/loupeHost.ts';
  import { persistedPreviewUrl } from '#lib/api/preview.ts';
  import { cullState } from '#lib/browse/cullState.ts';
  import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
  import { paneGridStyle, sheetInsetStyle } from '#lib/browse/loupeLayout.ts';
  import { pickFromStrip, stepLoupe } from '#lib/browse/loupeNav.ts';
  import { ImagePreloader, preloadUrls, type StepFn } from '#lib/browse/loupePreload.ts';
  import { loupeTags } from '#lib/stores/loupeTags.svelte.ts';
  import { ratingHud } from '#lib/browse/ratingHud.svelte.ts';
  import { putBounded } from '#lib/utils/boundedRecord.ts';
  import { loadFaceData } from '#lib/stores/zoomTargets.ts';
  import { ZoomCycle } from '#lib/stores/zoomCycle.ts';
  import { FIT_VIEW, withZoom, type View, type ViewportMetrics } from '#lib/utils/imageViewport.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import Filmstrip from '#lib/components/shell/Filmstrip.svelte';
  import LoupeActionRail from '#lib/components/browse/LoupeActionRail.svelte';
  import LoupePaneGrid from '#lib/components/browse/LoupePaneGrid.svelte';
  import LoupeToolbar from '#lib/components/browse/LoupeToolbar.svelte';
  import RatingHud from '#lib/components/browse/RatingHud.svelte';
  import LoupeInfo from '#lib/components/browse/info/LoupeInfo.svelte';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';
  import ContextMenuItem from '#lib/components/ContextMenuItem.svelte';
  import { mdiChevronLeft, mdiChevronRight, mdiFullscreenExit } from '@mdi/js';

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
  const moreActive = $derived(ui.clipWarn);
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
    (direction > 0 ? browsing.nextOf(id) : browsing.prevOf(id))?.id ?? null;

  onDestroy(() => {
    ui.fullscreen = false;
    preloader.keep([]);
    ratingHud.clear();
  });

  onMount(() =>
    loupeHost.attach({
      cycleZoom: () => zoomCycle.next(),
      panBy: (dx, dy) => {
        if (focusedId) paneGrid?.panBy(focusedId, dx, dy);
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

  function addTag(tag: TagRef): Promise<void> {
    return focusedId ? loupeTags.add(focusedId, tag) : Promise.resolve();
  }

  function removeTag(tagId: string): Promise<void> {
    return focusedId ? loupeTags.remove(focusedId, tagId) : Promise.resolve();
  }

  function createAndAddTag(value: string): Promise<TagRef | null> {
    return focusedId ? loupeTags.createAndAdd(focusedId, value) : Promise.resolve(null);
  }

  $effect(() => {
    const id = multi ? null : currentId;
    const render = (neighbour: string) => persistedPreviewUrl(neighbour, paneMaxEdge, ui.clipWarn);
    preloader.keep(id ? preloadUrls(id, stepShown, render) : []);
  });

  $effect(() => {
    const id = focusedId;
    if (id) untrack(() => browsing.prefetchNear(id));
  });

  let paneGrid = $state<ReturnType<typeof LoupePaneGrid>>();

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

  const hasPrev = $derived(
    !multi && currentId ? browsing.prevOf(currentId) !== null || browsing.hasPrevious : false
  );
  const atLoadedEnd = $derived(currentId ? browsing.nextOf(currentId) === null : false);
  const hasNext = $derived(!multi && currentId !== null && (!atLoadedEnd || browsing.hasMore));
</script>

{#snippet stripMenu(id: string)}
  {@const paneAction = multi && compare.members.includes(id) ? 'Remove from' : 'Add to'}
  {@const paneLabel = `${paneAction} ${compare.mode === 'survey' ? 'survey' : 'compare'}`}
  <ContextMenuItem title={hint('Open in editor', 'openEditor')} onSelect={() => openInEditor(id)}>
    Open in editor
  </ContextMenuItem>
  <ContextMenuItem disabled={!multi && id === currentId} onSelect={() => pickFromStrip(id, true)}>
    {paneLabel}
  </ContextMenuItem>
{/snippet}

{#if asset}
  <div class="safe-frame fixed inset-0 z-40 flex min-w-0 flex-col bg-image-canvas">
    {#if !ui.fullscreen}
      <LoupeToolbar filename={asset.originalFileName} {copyBadge} {multi} {moreActive} />
    {/if}

    <div class="flex min-h-0 flex-1">
      <div class="flex min-w-0 flex-1 flex-col">
        <div
          class="flex-1 min-h-0 relative {multi ? 'grid gap-1.5 bg-neutral-950 p-1.5' : 'flex'}"
          style={gridStyle}
        >
          <LoupePaneGrid
            bind:this={paneGrid}
            {panes}
            {focusedId}
            {multi}
            {canDrop}
            onView={applyPaneView}
            onSize={(size) => (paneMaxEdge = size)}
            onMetrics={setPaneMetrics}
            onImage={setPaneImage}
          />

          <RatingHud />

          {#if hasPrev && !ui.fullscreen}
            <IconButton
              type="button"
              size="medium"
              variant="ghost"
              color="secondary"
              shape="round"
              class="absolute top-1/2 left-2 -translate-y-1/2 bg-black/40 text-white hover:bg-black/70"
              icon={mdiChevronLeft}
              title={hint('Previous', 'loupeNav')}
              aria-label="Previous"
              onclick={() => stepLoupe(-1)}
            />
          {/if}
          {#if hasNext && !ui.fullscreen}
            <IconButton
              type="button"
              size="medium"
              variant="ghost"
              color="secondary"
              shape="round"
              class="absolute top-1/2 right-2 -translate-y-1/2 bg-black/40 text-white hover:bg-black/70"
              icon={mdiChevronRight}
              title={hint('Next', 'loupeNav')}
              aria-label="Next"
              loading={atLoadedEnd && browsing.loadingMore}
              onclick={() => stepLoupe(1)}
            />
          {/if}

          {#if ui.fullscreen}
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
            <LoupeActionRail
              cull={cullState(asset)}
              tags={currentTags}
              fileName={asset.originalFileName}
              {multi}
              {zoom}
              fitZoom={focusedFitZoom}
              fitMode={paneView.zoom === null}
              actions={loupeCullActions}
              onAddTag={addTag}
              onRemoveTag={removeTag}
              onCreateTag={createAndAddTag}
              onZoom={setZoom}
              onFit={fitZoom}
              {canDrop}
              onDrop={dropFocusedPane}
            />

            <Filmstrip
              currentId={focusedId}
              highlightIds={compare.members}
              onSelect={pickFromStrip}
              resizable
              size={72}
              showBadges
              collapsed={ui.loupeFilmstripCollapsed}
              menu={stripMenu}
            />
          </div>
        {/if}
      </div>
      {#if !ui.fullscreen}
        <LoupeInfo {asset} />
      {/if}
    </div>
  </div>
{/if}
