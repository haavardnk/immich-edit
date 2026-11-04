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
  import Notice from '#lib/components/Notice.svelte';
  import { Button, Icon } from '@immich/ui';
  import { mdiLoading } from '@mdi/js';
  import { frameBox, placement } from '#lib/utils/viewGeometry.ts';
  import {
    clampView,
    frameOffset,
    scaleOf,
    viewportMetrics,
    viewportTransform
  } from '#lib/utils/imageViewport.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { devicePixelRatio } from 'svelte/reactivity/window';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { clampedPoint } from '#lib/gestures/point.ts';
  import { viewportGestures } from '#lib/gestures/viewportGestures.ts';

  let container = $state<HTMLDivElement | null>(null);
  let imgEl = $state<PreviewSurface | null>(null);
  let viewBox = $state({ w: 0, h: 0 });
  const dpr = $derived(devicePixelRatio.current || 1);
  let baseNat = $state<{ w: number; h: number } | null>(null);
  let splitWrap = $state<HTMLDivElement | null>(null);
  let splitNatW = $state(0);
  let splitNatH = $state(0);
  let dragging = $state(false);

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
  const imageEdge = $derived(
    ui.greyCanvas ? 'ring-1 ring-white/10' : 'shadow-image ring-1 ring-white/10'
  );

  const gestures = viewportGestures({
    element: () => container,
    metrics: () => metrics,
    view: () => view,
    apply: (next, _e, change) => ui.applyView(next, change),
    dragging: (active) => (dragging = active),
    doubleTap(e) {
      if (ui.zoomed) ui.zoomFit();
      else gestures.zoomAt(e, ui.zoomLevel, e);
    }
  });

  const split = pointerDrag<true>({
    start(e) {
      updateSplit(e);
      e.stopPropagation();
      return true;
    },
    move: (e) => updateSplit(e)
  });

  function updateSplit(e: PointerEvent): void {
    if (!splitWrap) return;
    editor.setSplitPos(clampedPoint(e, splitWrap).x);
  }

  function onSplitKeyDown(e: KeyboardEvent): void {
    const step = held('coarse', e) ? 0.1 : 0.02;
    if (e.key === 'ArrowLeft') editor.setSplitPos(editor.splitPos - step);
    else if (e.key === 'ArrowRight') editor.setSplitPos(editor.splitPos + step);
    else if (e.key === 'Home') editor.setSplitPos(0);
    else if (e.key === 'End') editor.setSplitPos(1);
    else return;
    e.preventDefault();
  }

  const viewTransform = $derived(viewportTransform(metrics, view));

  function measure(): void {
    if (!container) return;
    viewBox = { w: container.clientWidth, h: container.clientHeight };
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
>
  {#if editor.geometrySession?.frameUrl}
    <CropOverlay />
  {:else if editor.previewUrl || editor.previewFrame}
    {#if editor.splitMode && editor.originalUrl}
      <div
        bind:this={splitWrap}
        class="relative overflow-hidden {imageEdge}"
        style="aspect-ratio: {splitNatW || 1} / {splitNatH ||
          1}; max-width: 100%; max-height: 100%; height: 100%; width: auto; {viewTransform}"
      >
        <img
          bind:this={imgEl}
          src={editor.originalUrl}
          alt="Original"
          class="absolute inset-0 w-full h-full object-contain select-none"
          style="image-orientation: none;"
          draggable="false"
          onload={(e) => {
            const t = e.target as HTMLImageElement;
            splitNatW = t.naturalWidth;
            splitNatH = t.naturalHeight;
          }}
        />
        <PreviewImage
          url={editor.previewUrl}
          frame={editor.previewFrame}
          alt={editor.asset?.originalFileName ?? ''}
          class="absolute inset-0 w-full h-full object-contain select-none"
          style="clip-path: inset(0 0 0 {editor.splitPos * 100}%);"
        />
        <div
          class="absolute top-0 bottom-0 w-0.5 bg-white/90 shadow-split pointer-events-none"
          style="left: {editor.splitPos * 100}%; transform: translateX(-50%);"
        ></div>
        <span
          class="pointer-events-none absolute left-2 top-2 rounded bg-black/65 px-2 py-1 text-[10px] font-medium uppercase tracking-wider text-white"
          >Original</span
        >
        <span
          class="pointer-events-none absolute right-2 top-2 rounded bg-black/65 px-2 py-1 text-[10px] font-medium uppercase tracking-wider text-white"
          >Edited</span
        >
        <div
          role="slider"
          tabindex="0"
          aria-label="Before/after split"
          aria-valuenow={Math.round(editor.splitPos * 100)}
          aria-valuemin="0"
          aria-valuemax="100"
          class="absolute top-1/2 w-7 h-7 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white border-2 border-black/40 shadow-lg cursor-ew-resize flex items-center justify-center text-black/70 text-xs font-bold"
          style="left: {editor.splitPos * 100}%;"
          {...split.on()}
          onkeydown={onSplitKeyDown}
        >
          ↔
        </div>
      </div>
    {:else}
      <PreviewImage
        bind:element={imgEl}
        url={editor.previewUrl}
        frame={editor.previewFrame}
        alt={editor.asset?.originalFileName ?? ''}
        class="max-h-none max-w-none select-none object-contain {imageEdge}"
        style={baseStyle}
        testid="preview-image"
        onsize={(size) => {
          if (baseNat?.w !== size.w || baseNat.h !== size.h) baseNat = size;
        }}
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
</div>
