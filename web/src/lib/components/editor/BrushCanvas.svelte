<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { type MaskComponent, type MaskLayer } from '#lib/types/edits.ts';
  import {
    bufferToImageData,
    maskOverlayRgb,
    stampBuffer,
    type BrushBuffer
  } from '#lib/utils/brush.ts';
  import {
    displayUvToSceneUv as toSceneUv,
    scenePerDisplayAt as scenePerDisplay,
    steppedSegment,
    viewIsIdentity,
    viewTransform
  } from '#lib/utils/canvasCoords.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import { BRUSH_HARDNESS, BRUSH_SIZE } from '#lib/editor/params/brush.ts';
  import { brushWheel } from '#lib/utils/brushSize.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { elementPoint } from '#lib/gestures/point.ts';

  let {
    img
  }: {
    img: PreviewSurface | null;
  } = $props();

  const rect = imageRect(() => img);
  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let strokeActive = $state(false);
  let hover = $state<{ x: number; y: number } | null>(null);
  let lastPx: number | null = null;
  let lastPy: number | null = null;

  const view = $derived(viewTransform(editor.edits, editor.meta ?? null, editor.lensView));

  const allIdentity = $derived(viewIsIdentity(view));

  const active = $derived<MaskLayer | null>(
    editor.activeLayerId
      ? (editor.edits.masks.find((l) => l.id === editor.activeLayerId) ?? null)
      : null
  );
  const activeComp = $derived<MaskComponent | null>(
    active && editor.activeMaskComponentId
      ? (active.components.find((c) => c.id === editor.activeMaskComponentId) ?? null)
      : null
  );
  const isBrush = $derived(!!activeComp && activeComp.kind.kind === 'brush');
  const ringRadius = $derived(editor.brushTool.size * 0.5 * Math.min(rect.w, rect.h));
  const show = $derived(isBrush && rect.w > 0 && rect.h > 0);

  $effect(() => {
    if (!show || !canvasEl || !activeComp || activeComp.kind.kind !== 'brush') return;
    void activeComp.invert;
    if (editor.maskStroke === 'idle') {
      canvasEl.getContext('2d')?.clearRect(0, 0, canvasEl.width, canvasEl.height);
      return;
    }
    void repaint(activeComp.id, activeComp.kind.raster_id);
  });

  function displayUvToSceneUv(du: number, dv: number): [number, number] {
    return toSceneUv(view, du, dv);
  }

  function scenePerDisplayAt(du: number, dv: number): number {
    return scenePerDisplay(view, du, dv);
  }

  async function repaint(componentId: string, rasterId: string): Promise<void> {
    if (!canvasEl) return;
    const buf = await editor.ensureBrushBuffer(componentId, rasterId);
    if (!canvasEl || editor.maskStroke === 'idle') return;
    const w = Math.max(1, Math.floor(rect.w));
    const h = Math.max(1, Math.floor(rect.h));
    if (canvasEl.width !== w) canvasEl.width = w;
    if (canvasEl.height !== h) canvasEl.height = h;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    const color = maskOverlayRgb(canvasEl);
    const invert = activeComp?.invert === true;
    if (allIdentity) {
      const off = document.createElement('canvas');
      off.width = buf.width;
      off.height = buf.height;
      const offCtx = off.getContext('2d');
      if (!offCtx) return;
      offCtx.putImageData(bufferToImageData(buf, color, 0.6, invert), 0, 0);
      ctx.clearRect(0, 0, w, h);
      ctx.drawImage(off, 0, 0, w, h);
    } else {
      ctx.clearRect(0, 0, w, h);
      const img = ctx.createImageData(w, h);
      sampleBufferToImageData(buf, w, h, color, invert, img.data);
      ctx.putImageData(img, 0, 0);
    }
  }

  function sampleBufferToImageData(
    buf: BrushBuffer,
    w: number,
    h: number,
    color: [number, number, number],
    invert: boolean,
    out: Uint8ClampedArray
  ): void {
    const bw = buf.width;
    const bh = buf.height;
    const bytes = buf.bytes;
    const [r, g, b] = color;
    for (let y = 0; y < h; y++) {
      const dv = (y + 0.5) / h;
      for (let x = 0; x < w; x++) {
        const du = (x + 0.5) / w;
        const s = displayUvToSceneUv(du, dv);
        const bx = Math.floor(s[0] * bw);
        const by = Math.floor(s[1] * bh);
        const o = (y * w + x) * 4;
        if (bx < 0 || by < 0 || bx >= bw || by >= bh) {
          out[o + 3] = 0;
          continue;
        }
        const raw = bytes[by * bw + bx] ?? 0;
        const a = invert ? 255 - raw : raw;
        out[o] = r;
        out[o + 1] = g;
        out[o + 2] = b;
        out[o + 3] = Math.round(a * 0.6);
      }
    }
  }

  function stampAt(e: PointerEvent): void {
    if (!canvasEl || !active || !activeComp || activeComp.kind.kind !== 'brush') return;
    const buf = editor.brushBuffers[activeComp.id];
    if (!buf) return;
    const rect = canvasEl.getBoundingClientRect();
    const { x: px, y: py } = elementPoint(e, rect);
    const color = maskOverlayRgb(canvasEl);
    if (lastPx !== null && lastPy !== null) {
      const radiusPx = editor.brushTool.size * 0.5 * Math.min(rect.width, rect.height);
      for (const [sx, sy] of steppedSegment(lastPx, lastPy, px, py, Math.max(1, radiusPx * 0.5))) {
        stampAtPx(buf, rect, sx, sy, color);
      }
    } else {
      stampAtPx(buf, rect, px, py, color);
    }
    lastPx = px;
    lastPy = py;
  }

  function stampAtPx(
    buf: BrushBuffer,
    rect: DOMRect,
    px: number,
    py: number,
    [r, g, b]: [number, number, number]
  ): void {
    if (!canvasEl || !active) return;
    const nx = px / Math.max(1, rect.width);
    const ny = py / Math.max(1, rect.height);
    const scene = displayUvToSceneUv(nx, ny);
    const radiusN = editor.brushTool.size * 0.5;
    const j = scenePerDisplayAt(nx, ny);
    const radiusScene = radiusN * j;
    const cxBuf = scene[0] * buf.width;
    const cyBuf = scene[1] * buf.height;
    const radiusBuf = radiusScene * Math.min(buf.width, buf.height);
    const alphaByte = Math.round(editor.brushTool.flow * 255);
    const erase = editor.brushTool.mode === 'erase';
    stampBuffer(buf, cxBuf, cyBuf, radiusBuf, editor.brushTool.hardness, alphaByte, erase);

    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    const cxPx = nx * canvasEl.width;
    const cyPx = ny * canvasEl.height;
    const rPx = radiusN * Math.min(canvasEl.width, canvasEl.height);
    const h = Math.min(1, Math.max(0, editor.brushTool.hardness));
    const inner = Math.max(0, rPx * h);
    const grad = ctx.createRadialGradient(
      cxPx,
      cyPx,
      inner,
      cxPx,
      cyPx,
      Math.max(inner + 0.5, rPx)
    );
    const clears = erase !== (activeComp?.invert === true);
    ctx.globalCompositeOperation = clears ? 'destination-out' : 'source-over';
    grad.addColorStop(0, `rgba(${r},${g},${b},${editor.brushTool.flow * 0.6})`);
    grad.addColorStop(1, `rgba(${r},${g},${b},0)`);
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(cxPx, cyPx, Math.max(0.5, rPx), 0, Math.PI * 2);
    ctx.fill();
    ctx.globalCompositeOperation = 'source-over';
  }

  async function stampWhenLoaded(
    e: PointerEvent,
    id: string,
    rasterId: string | null
  ): Promise<void> {
    await editor.ensureBrushBuffer(id, rasterId);
    stampAt(e);
  }

  function trackHover(e: PointerEvent): void {
    hover = elementPoint(e, e.currentTarget as Element);
  }

  const stroke = pointerDrag<true>({
    start(e) {
      if (!activeComp || activeComp.kind.kind !== 'brush' || !active || e.button !== 0) return null;
      e.preventDefault();
      e.stopPropagation();
      strokeActive = true;
      lastPx = null;
      lastPy = null;
      editor.beginMaskStroke();
      void stampWhenLoaded(e, activeComp.id, activeComp.kind.raster_id);
      return true;
    },
    hover: trackHover,
    move(e) {
      trackHover(e);
      if (!strokeActive) return;
      stampAt(e);
    },
    end: () => void finishStroke()
  });

  function onWheel(e: WheelEvent): void {
    brushWheel(
      e,
      editor.brushTool,
      { size: BRUSH_SIZE, hardness: BRUSH_HARDNESS },
      editor.setBrushTool
    );
  }

  async function finishStroke(): Promise<void> {
    if (!strokeActive) return;
    strokeActive = false;
    lastPx = null;
    lastPy = null;
    if (!active || !activeComp) {
      editor.endMaskStroke();
      return;
    }
    await editor.commitBrushStroke(active.id, activeComp.id);
    editor.endMaskStroke();
  }
</script>

{#if show}
  <canvas
    bind:this={canvasEl}
    class="absolute"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none; cursor: crosshair;"
    {...stroke.on()}
    onpointerleave={() => (hover = null)}
    onwheel={onWheel}
    data-testid="brush-canvas"
  ></canvas>
  {#if hover}
    <div
      class="pointer-events-none absolute rounded-full border border-white/90 shadow-[0_0_0_1px_rgb(0_0_0/0.55)]"
      style="left: {rect.x + hover.x - ringRadius}px; top: {rect.y +
        hover.y -
        ringRadius}px; width: {ringRadius * 2}px; height: {ringRadius * 2}px;"
      data-testid="brush-cursor"
      aria-hidden="true"
    ></div>
  {/if}
{/if}
