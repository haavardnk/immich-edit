<script lang="ts">
  import { observeSize } from '$lib/actions/observeSize';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import CropOverlay from './CropOverlay.svelte';
  import MaskOverlay from './MaskOverlay.svelte';
  import BrushCanvas from './BrushCanvas.svelte';
  import ClickCanvas from './ClickCanvas.svelte';
  import RetouchOverlay from './RetouchOverlay.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { Icon } from '@immich/ui';
  import { mdiLoading } from '@mdi/js';
  import { fitScale, frameBox, nativeScale, placement } from '$lib/utils/view-geometry';
  import { splitPosition, viewportTransform, zoomAtAnchor } from '$lib/utils/imageViewport';

  const WHEEL_STEP = 1.1;

  let container = $state<HTMLDivElement | null>(null);
  let imgEl = $state<HTMLImageElement | null>(null);
  let viewBox = $state({ w: 0, h: 0 });
  let dpr = $state(1);
  let baseNat = $state<{ w: number; h: number } | null>(null);
  let splitWrap = $state<HTMLDivElement | null>(null);
  let splitNatW = $state(0);
  let splitNatH = $state(0);
  let dragging = $state(false);
  let splitDragging = $state(false);
  let lastX = 0;
  let lastY = 0;

  const fit = $derived(baseNat ? fitScale(viewBox.w, viewBox.h, baseNat.w, baseNat.h) : 0);
  const unit = $derived(
    baseNat ? nativeScale(Math.max(baseNat.w, baseNat.h), editor.sourceLong, dpr) || fit : 0
  );
  const scale = $derived(ui.fitMode ? fit : (ui.zoom / 100) * unit);

  const frame = $derived.by(() => {
    if (!baseNat || viewBox.w <= 0 || viewBox.h <= 0) return null;
    return frameBox(viewBox.w, viewBox.h, baseNat.w, baseNat.h, scale, ui.panX, ui.panY, dpr);
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

  function onPointerDown(e: PointerEvent): void {
    if (!ui.zoomed) return;
    dragging = true;
    lastX = e.clientX;
    lastY = e.clientY;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent): void {
    if (!dragging) return;
    ui.panX += e.clientX - lastX;
    ui.panY += e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
  }

  function onPointerUp(): void {
    dragging = false;
  }

  function onSplitPointerDown(e: PointerEvent): void {
    splitDragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    updateSplit(e);
    e.stopPropagation();
  }

  function onSplitPointerMove(e: PointerEvent): void {
    if (!splitDragging) return;
    updateSplit(e);
  }

  function onSplitPointerUp(): void {
    splitDragging = false;
  }

  function updateSplit(e: PointerEvent): void {
    if (!splitWrap) return;
    const rect = splitWrap.getBoundingClientRect();
    editor.setSplitPos(splitPosition(e.clientX, rect.left, rect.width));
  }

  function onSplitKeyDown(e: KeyboardEvent): void {
    const step = e.shiftKey ? 0.1 : 0.02;
    if (e.key === 'ArrowLeft') editor.setSplitPos(editor.splitPos - step);
    else if (e.key === 'ArrowRight') editor.setSplitPos(editor.splitPos + step);
    else if (e.key === 'Home') editor.setSplitPos(0);
    else if (e.key === 'End') editor.setSplitPos(1);
    else return;
    e.preventDefault();
  }

  function zoomAt(next: number, clientX: number, clientY: number): void {
    const before = frame;
    const prev = ui.zoom;
    ui.userZoom(next);
    if (!before || !container || ui.zoom === prev || !ui.zoomed) return;
    const rect = container.getBoundingClientRect();
    const pan = zoomAtAnchor(
      viewBox.w,
      viewBox.h,
      before,
      clientX - rect.left,
      clientY - rect.top,
      prev,
      ui.zoom
    );
    ui.panX = pan.panX;
    ui.panY = pan.panY;
  }

  function onWheel(e: WheelEvent): void {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      zoomAt(ui.zoom * (e.deltaY > 0 ? 1 / WHEEL_STEP : WHEEL_STEP), e.clientX, e.clientY);
      return;
    }
    if (!ui.zoomed) return;
    e.preventDefault();
    ui.panX -= e.deltaX;
    ui.panY -= e.deltaY;
  }

  function onDblClick(e: MouseEvent): void {
    if (ui.zoomed) {
      ui.zoomFit();
      return;
    }
    zoomAt(ui.zoomLevel, e.clientX, e.clientY);
  }

  const viewTransform = $derived.by(() =>
    viewportTransform(fit > 0 ? scale / fit : 1, ui.panX, ui.panY)
  );

  function measure(): void {
    if (!container) return;
    viewBox = { w: container.clientWidth, h: container.clientHeight };
  }

  $effect(() => {
    const update = (): void => {
      dpr = window.devicePixelRatio || 1;
    };
    update();
    window.addEventListener('resize', update);
    return () => window.removeEventListener('resize', update);
  });

  $effect(() => {
    const box = frame;
    if (!box) return;
    editor.onViewChange({ frame: box, viewW: viewBox.w, viewH: viewBox.h, dpr });
  });

  $effect(() => {
    editor.setBaseImage(imgEl);
    return () => editor.setBaseImage(null);
  });

  $effect(() => {
    ui.setFitZoom(unit > 0 ? (100 * fit) / unit : 100);
  });
</script>

<div
  bind:this={container}
  use:observeSize={measure}
  role="application"
  class="editor-stage relative flex min-h-0 flex-1 items-center justify-center overflow-hidden"
  class:cursor-grab={ui.zoomed && !dragging && !editor.geometrySession}
  class:cursor-grabbing={dragging}
  onpointerdown={editor.geometrySession ? undefined : onPointerDown}
  onpointermove={editor.geometrySession ? undefined : onPointerMove}
  onpointerup={editor.geometrySession ? undefined : onPointerUp}
  onpointercancel={editor.geometrySession ? undefined : onPointerUp}
  onwheel={editor.geometrySession ? undefined : onWheel}
  ondblclick={editor.geometrySession ? undefined : onDblClick}
>
  {#if editor.geometrySession && editor.geometrySession.pinnedReady}
    <CropOverlay />
  {:else if editor.previewUrl}
    {#if editor.splitMode && editor.originalUrl}
      <div
        bind:this={splitWrap}
        class="relative overflow-hidden shadow-image ring-1 ring-white/10"
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
        <img
          src={editor.previewUrl}
          alt={editor.asset?.originalFileName ?? ''}
          class="absolute inset-0 w-full h-full object-contain select-none"
          style="clip-path: inset(0 0 0 {editor.splitPos * 100}%); image-orientation: none;"
          draggable="false"
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
          onpointerdown={onSplitPointerDown}
          onpointermove={onSplitPointerMove}
          onpointerup={onSplitPointerUp}
          onpointercancel={onSplitPointerUp}
          onkeydown={onSplitKeyDown}
        >
          ↔
        </div>
      </div>
    {:else}
      <img
        bind:this={imgEl}
        src={editor.previewUrl}
        alt={editor.asset?.originalFileName ?? ''}
        class="max-h-none max-w-none select-none object-contain shadow-image ring-1 ring-white/10"
        style="{baseStyle} image-orientation: none;"
        draggable="false"
        onload={(e) => {
          const t = e.target as HTMLImageElement;
          if (t.naturalWidth > 0) baseNat = { w: t.naturalWidth, h: t.naturalHeight };
        }}
      />
      {#if editor.viewUrl && viewPlace}
        <img
          src={editor.viewUrl}
          alt=""
          data-testid="view-render"
          class="pointer-events-none absolute max-h-none max-w-none select-none"
          style="left: {viewPlace.left}px; top: {viewPlace.top}px; width: {viewPlace.width}px; height: {viewPlace.height}px; image-orientation: none;"
          draggable="false"
        />
      {/if}
      <MaskOverlay img={imgEl} />
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
          class="flex items-center gap-2 rounded-full bg-black/70 px-3 py-1.5 text-xs text-dark shadow-lg backdrop-blur-sm"
        >
          <Icon icon={mdiLoading} size="14px" class="animate-spin" aria-hidden="true" />
          Building mask…
        </span>
      </div>
    {/if}
  {:else if editor.error}
    <Notice message={editor.error} class="text-sm" />
  {:else}
    <div class="flex gap-1">
      <div
        class="h-2 w-2 animate-bounce rounded-full bg-primary/50"
        style="animation-delay: 0ms"
      ></div>
      <div
        class="h-2 w-2 animate-bounce rounded-full bg-primary/50"
        style="animation-delay: 150ms"
      ></div>
      <div
        class="h-2 w-2 animate-bounce rounded-full bg-primary/50"
        style="animation-delay: 300ms"
      ></div>
    </div>
  {/if}
</div>
