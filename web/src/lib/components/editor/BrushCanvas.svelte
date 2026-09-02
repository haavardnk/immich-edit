<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { type MaskComponent, type MaskLayer } from '$lib/types/edits';
  import {
    bufferToImageData,
    parseHexColor,
    stampBuffer,
    type BrushBuffer
  } from '$lib/utils/brush';
  import {
    displayUvToSceneUv as toSceneUv,
    scenePerDisplayAt as scenePerDisplay,
    steppedSegment,
    viewIsIdentity,
    viewTransform
  } from '$lib/utils/canvasCoords';
  import { imageRect } from '$lib/utils/imageRect.svelte';

  let {
    img
  }: {
    img: HTMLImageElement | null;
  } = $props();

  const rect = imageRect(() => img);
  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let strokeActive = $state(false);
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
  const show = $derived(
    editor.maskOverlayVisible &&
      isBrush &&
      editor.maskPreviewLayerId === null &&
      rect.w > 0 &&
      rect.h > 0
  );

  $effect(() => {
    if (!show || !canvasEl || !activeComp || activeComp.kind.kind !== 'brush') return;
    void activeComp.invert;
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
    if (!canvasEl) return;
    const w = Math.max(1, Math.floor(rect.w));
    const h = Math.max(1, Math.floor(rect.h));
    if (canvasEl.width !== w) canvasEl.width = w;
    if (canvasEl.height !== h) canvasEl.height = h;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    const color = active
      ? parseHexColor(active.color)
      : ([255, 60, 60] as [number, number, number]);
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
        const a = invert ? 255 - bytes[by * bw + bx] : bytes[by * bw + bx];
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
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    if (lastPx !== null && lastPy !== null) {
      const radiusPx = editor.brushTool.size * 0.5 * Math.min(rect.width, rect.height);
      for (const [sx, sy] of steppedSegment(lastPx, lastPy, px, py, Math.max(1, radiusPx * 0.5))) {
        stampAtPx(buf, rect, sx, sy);
      }
    } else {
      stampAtPx(buf, rect, px, py);
    }
    lastPx = px;
    lastPy = py;
  }

  function stampAtPx(buf: BrushBuffer, rect: DOMRect, px: number, py: number): void {
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
    const [r, g, b] = parseHexColor(active.color);
    grad.addColorStop(0, `rgba(${r},${g},${b},${editor.brushTool.flow * 0.6})`);
    grad.addColorStop(1, `rgba(${r},${g},${b},0)`);
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(cxPx, cyPx, Math.max(0.5, rPx), 0, Math.PI * 2);
    ctx.fill();
    ctx.globalCompositeOperation = 'source-over';
  }

  async function onPointerDown(e: PointerEvent): Promise<void> {
    if (!activeComp || activeComp.kind.kind !== 'brush' || !active || e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    strokeActive = true;
    lastPx = null;
    lastPy = null;
    await editor.ensureBrushBuffer(activeComp.id, activeComp.kind.raster_id);
    stampAt(e);
  }

  function onPointerMove(e: PointerEvent): void {
    if (!strokeActive) return;
    stampAt(e);
  }

  async function onPointerUp(e: PointerEvent): Promise<void> {
    if (!strokeActive) return;
    strokeActive = false;
    lastPx = null;
    lastPy = null;
    (e.currentTarget as Element).releasePointerCapture?.(e.pointerId);
    if (!active || !activeComp) return;
    await editor.commitBrushStroke(active.id, activeComp.id);
    if (canvasEl && activeComp.kind.kind === 'brush') {
      await repaint(activeComp.id, activeComp.kind.raster_id);
    }
  }
</script>

{#if show}
  <canvas
    bind:this={canvasEl}
    class="absolute"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none; cursor: crosshair;"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
  ></canvas>
{/if}
