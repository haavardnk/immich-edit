<script lang="ts">
  import { observeSize } from '#lib/actions/observeSize.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import CropOverlay from './CropOverlay.svelte';
  import MaskWeight from './MaskWeight.svelte';
  import MaskOverlay from './MaskOverlay.svelte';
  import WhiteBalancePicker from './WhiteBalancePicker.svelte';
  import BrushCanvas from './BrushCanvas.svelte';
  import ClickCanvas from './ClickCanvas.svelte';
  import RetouchOverlay from './RetouchOverlay.svelte';
  import PreviewImage from './PreviewImage.svelte';
  import CompareSideBySide from './compare/CompareSideBySide.svelte';
  import CompareSplit from './compare/CompareSplit.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import { Button, Icon } from '@immich/ui';
  import { mdiLoading } from '@mdi/js';
  import { frameBox, placement } from '#lib/utils/viewGeometry.ts';
  import { clampView, frameOffset, scaleOf, viewportMetrics } from '#lib/utils/imageViewport.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { devicePixelRatio } from 'svelte/reactivity/window';
  import type { Box, ClientPoint } from '#lib/gestures/point.ts';
  import { viewportGestures } from '#lib/gestures/viewportGestures.ts';
  import { comparePane, compareViewport } from '#lib/editor/compare.ts';
  import { viewerGestures, type ViewerAdjust } from '#lib/editor/viewerGestures.ts';
  import { runCommand } from '#lib/commands/command.ts';
  import { holdOriginal } from '#lib/commands/editor/view.ts';
  import TargetOverlay from './TargetOverlay.svelte';
  import PhotoPill from './PhotoPill.svelte';
  import { presetStrip } from '#lib/stores/presetStrip.svelte.ts';
  import VersionHoldImage from './versions/VersionHoldImage.svelte';
  import { lutPreview } from '#lib/stores/lutPreview.svelte.ts';
  import Navigator from './Navigator.svelte';

  let {
    ontap = () => {},
    adjust = () => null
  }: { ontap?: () => void; adjust?: () => ViewerAdjust | null } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let imgEl = $state<PreviewSurface | null>(null);
  let stage = $state({ w: 0, h: 0 });
  const viewBox = $derived(compareViewport(stage, editor.compare.mode));
  const dpr = $derived(devicePixelRatio.current || 1);
  let baseNat = $state<{ w: number; h: number } | null>(null);
  let dragging = $state(false);
  const holdUrl = $derived(editor.versions.hold.url);
  const pill = $derived(
    editor.versions.hold.pill ??
      presetStrip.pill ??
      lutPreview.pill ??
      (editor.showingOriginal ? null : (editor.historySteps.pill ?? presetStrip.appliedPill))
  );

  const metrics = $derived(
    viewportMetrics(viewBox, baseNat ?? { w: 0, h: 0 }, editor.sourceLong, dpr)
  );
  const view = $derived(clampView(metrics, ui.view));

  const frame = $derived.by(() => {
    if (!baseNat || viewBox.w <= 0 || viewBox.h <= 0) return null;
    const offset = frameOffset(metrics, view);
    const scale = scaleOf(metrics, view.zoom);
    return frameBox(viewBox.w, viewBox.h, baseNat.w, baseNat.h, scale, offset.x, offset.y, dpr);
  });

  const viewPlace = $derived.by(() => {
    if (!frame || !editor.viewRoi || !editor.viewNat) return null;
    return placement(editor.viewRoi, frame, editor.viewNat.w, editor.viewNat.h, dpr);
  });

  const baseStyle = $derived(
    frame
      ? `position: absolute; left: ${frame.left}px; top: ${frame.top}px; width: ${frame.width}px; height: ${frame.height}px;`
      : 'max-width: 100%; max-height: 100%;'
  );
  const showNavigator = $derived(
    ui.zoomed && frame !== null && editor.compare.mode !== 'side' && !editor.geometrySession
  );
  const imageEdge = $derived(
    ui.greyCanvas ? 'ring-1 ring-white/10' : 'shadow-image ring-1 ring-white/10'
  );

  const gestures = viewportGestures({
    element: () => container,
    anchor: paneAt,
    metrics: () => metrics,
    view: () => view,
    apply: (next, _e, change) => ui.applyView(next, change),
    dragging: (active) => (dragging = active),
    doubleTap(e) {
      if (ui.zoomed) ui.zoomFit();
      else gestures.zoomAt(e, ui.zoomLevel, e);
    },
    pointerTypes: ['mouse', 'pen']
  });

  const touchGestures = viewerGestures({
    element: () => container,
    anchor: paneAt,
    metrics: () => metrics,
    view: () => view,
    apply: (next, change) => ui.applyView(next, change),
    zoomLevel: () => ui.zoomLevel,
    enabled: () => !editor.geometrySession,
    tap: () => ontap(),
    hold: { start: () => runCommand(holdOriginal), end: () => holdOriginal.release() },
    adjust: () => adjust()
  });

  function paneAt(point: ClientPoint): Box | null {
    return container
      ? comparePane(container.getBoundingClientRect(), point, editor.compare.mode)
      : null;
  }

  function measured(size: { w: number; h: number }): void {
    if (baseNat?.w !== size.w || baseNat.h !== size.h) baseNat = size;
  }

  function measure(): void {
    if (!container) return;
    stage = { w: container.clientWidth, h: container.clientHeight };
  }

  $effect(() => {
    const box = frame;
    if (!box) return;
    editor.onViewChange({ frame: box, viewW: viewBox.w, viewH: viewBox.h, dpr }, metrics);
  });

  $effect(() => {
    editor.setBaseImage(imgEl);
    return () => editor.setBaseImage(null);
  });

  $effect(() => {
    ui.setFitZoom(metrics.fitZoom);
  });
</script>

<div
  bind:this={container}
  use:observeSize={measure}
  role="application"
  class="editor-stage no-callout relative flex min-h-0 flex-1 items-center justify-center overflow-hidden"
  class:cursor-grab={ui.zoomed && !dragging && !editor.geometrySession}
  class:cursor-grabbing={dragging}
  {...editor.geometrySession ? {} : gestures.handlers}
  {@attach touchGestures}
>
  {#if editor.geometrySession?.frameUrl}
    <CropOverlay />
  {:else if editor.previewUrl || editor.previewFrame}
    {#if editor.compare.mode === 'split' && editor.compareUrl}
      <CompareSplit bind:surface={imgEl} frame={baseStyle} edge={imageEdge} onsize={measured} />
    {:else if editor.compare.mode === 'side'}
      <CompareSideBySide
        bind:surface={imgEl}
        frame={baseStyle}
        edge={imageEdge}
        onsize={measured}
      />
    {:else}
      <PreviewImage
        bind:element={imgEl}
        url={editor.previewUrl}
        frame={editor.previewFrame}
        alt={editor.asset?.originalFileName ?? ''}
        class="max-h-none max-w-none select-none object-contain {imageEdge}"
        style={baseStyle}
        testid="preview-image"
        onsize={measured}
      />
      {#if (editor.viewUrl || editor.viewFrame) && viewPlace}
        <PreviewImage
          url={editor.viewUrl}
          frame={editor.viewFrame}
          alt=""
          testid="view-render"
          class="pointer-events-none absolute max-h-none max-w-none select-none"
          style="left: {viewPlace.left}px; top: {viewPlace.top}px; width: {viewPlace.width}px; height: {viewPlace.height}px;"
        />
      {/if}
      <MaskWeight img={imgEl} />
      <MaskOverlay img={imgEl} />
      <WhiteBalancePicker img={imgEl} />
      <TargetOverlay img={imgEl} />
      <BrushCanvas img={imgEl} />
      <ClickCanvas img={imgEl} />
      <RetouchOverlay img={imgEl} />
    {/if}
    {#if editor.maskGenerating}
      <div
        class="pointer-events-none absolute inset-0 flex items-center justify-center"
        role="status"
        aria-live="polite"
      >
        <span
          class="flex items-center gap-2 rounded-full bg-black/70 px-3 py-1.5 text-xs text-white/85 shadow-lg backdrop-blur-sm"
        >
          <Icon icon={mdiLoading} size="14px" class="animate-spin" aria-hidden="true" />
          Building mask…
        </span>
      </div>
    {/if}
  {:else if editor.error}
    <Notice message={editor.error} class="text-sm">
      <Button
        size="tiny"
        variant="outline"
        color="danger"
        class="ms-1"
        onclick={editor.retryPreview}
      >
        Retry
      </Button>
    </Notice>
  {:else}
    <div class="flex gap-1">
      <div
        class="h-2 w-2 animate-bounce rounded-full bg-image-accent/50"
        style="animation-delay: 0ms"
      ></div>
      <div
        class="h-2 w-2 animate-bounce rounded-full bg-image-accent/50"
        style="animation-delay: 150ms"
      ></div>
      <div
        class="h-2 w-2 animate-bounce rounded-full bg-image-accent/50"
        style="animation-delay: 300ms"
      ></div>
    </div>
  {/if}
  {#if holdUrl}
    {#key holdUrl}
      <VersionHoldImage url={holdUrl} alt={editor.asset?.originalFileName ?? ''} />
    {/key}
  {/if}
  {#if pill}
    <PhotoPill {...pill} class="[[data-arrangement=side]_&]:top-15" />
  {/if}
  {#if showNavigator}
    <Navigator
      {metrics}
      {view}
      onpan={(next) => ui.applyView(next, 'pan')}
      class="right-3.5 bottom-3.5"
    />
  {/if}
</div>
