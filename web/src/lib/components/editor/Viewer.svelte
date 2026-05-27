<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import CropOverlay from './CropOverlay.svelte';
  import MaskOverlay from './MaskOverlay.svelte';
  import BrushCanvas from './BrushCanvas.svelte';

  let container = $state<HTMLDivElement | null>(null);
  let imgEl = $state<HTMLImageElement | null>(null);
  let splitWrap = $state<HTMLDivElement | null>(null);
  let splitNatW = $state(0);
  let splitNatH = $state(0);
  let dragging = $state(false);
  let splitDragging = $state(false);
  let lastX = 0;
  let lastY = 0;

  function onPointerDown(e: PointerEvent): void {
    if (ui.zoom <= 100) return;
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
    editor.setSplitPos((e.clientX - rect.left) / rect.width);
  }

  function onWheel(e: WheelEvent): void {
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -10 : 10;
    ui.setZoom(ui.zoom + delta);
  }

  function onDblClick(): void {
    if (ui.zoom > 100) {
      ui.zoomFit();
    } else {
      ui.setZoom(200);
    }
  }

  $effect(() => {
    editor.onZoomChange(ui.zoom);
  });
</script>

<div
  bind:this={container}
  role="application"
  class="flex-1 min-h-0 flex items-center justify-center bg-black/40 relative overflow-hidden"
  class:cursor-grab={ui.zoom > 100 && !dragging && !editor.cropSession}
  class:cursor-grabbing={dragging}
  onpointerdown={editor.cropSession ? undefined : onPointerDown}
  onpointermove={editor.cropSession ? undefined : onPointerMove}
  onpointerup={editor.cropSession ? undefined : onPointerUp}
  onpointercancel={editor.cropSession ? undefined : onPointerUp}
  onwheel={editor.cropSession ? undefined : onWheel}
  ondblclick={editor.cropSession ? undefined : onDblClick}
>
  {#if editor.cropSession && editor.cropSession.pinnedReady}
    <CropOverlay />
  {:else if editor.previewUrl}
    {#if editor.splitMode && editor.originalUrl}
      <div
        bind:this={splitWrap}
        class="relative shadow-2xl rounded overflow-hidden"
        style="aspect-ratio: {splitNatW || 1} / {splitNatH || 1}; max-width: 100%; max-height: 100%; height: 100%; width: auto; transform: scale({ui.zoom / 100}) translate({ui.panX / (ui.zoom / 100)}px, {ui.panY / (ui.zoom / 100)}px); transform-origin: center;"
      >
        <img
          bind:this={imgEl}
          src={editor.originalUrl}
          alt="original"
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
          class="absolute top-0 bottom-0 w-0.5 bg-white/90 shadow-[0_0_4px_rgba(0,0,0,0.6)] pointer-events-none"
          style="left: {editor.splitPos * 100}%; transform: translateX(-50%);"
        ></div>
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
        >↔</div>
      </div>
    {:else}
      <img
        bind:this={imgEl}
        src={editor.previewUrl}
        alt={editor.asset?.originalFileName ?? ''}
        class="max-w-full max-h-full object-contain shadow-2xl rounded select-none"
        style="transform: scale({ui.zoom / 100}) translate({ui.panX / (ui.zoom / 100)}px, {ui.panY / (ui.zoom / 100)}px); transform-origin: center; image-orientation: none;"
        draggable="false"
      />
      <MaskOverlay img={imgEl} />
      <BrushCanvas img={imgEl} />
    {/if}
  {:else if editor.error}
    <div class="text-red-400 text-sm">{editor.error}</div>
  {:else}
    <div class="flex gap-1">
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 0ms"></div>
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 150ms"></div>
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 300ms"></div>
    </div>
  {/if}
</div>
