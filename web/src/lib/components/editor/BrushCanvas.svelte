<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { type MaskComponent, type MaskLayer } from '$lib/types/edits';
  import { bufferToImageData, parseHexColor, stampBuffer, type BrushBuffer } from '$lib/utils/brush';
  import {
    lensWarpFromEdits,
    lensWarpActive,
    maskUvToSceneUv,
    type LensWarpParams
  } from '$lib/utils/lensWarp';
  import {
    displayUvToMaskUv,
    geometryIsIdentity,
    type GeometryTransform,
    type RotateQuarter
  } from '$lib/utils/geomTransform';

  let {
    img
  }: {
    img: HTMLImageElement | null;
  } = $props();

  let rectX = $state(0);
  let rectY = $state(0);
  let rectW = $state(0);
  let rectH = $state(0);
  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let strokeActive = $state(false);
  let lastCompId = $state<string | null>(null);
  let lastPx: number | null = null;
  let lastPy: number | null = null;

  function recompute(): void {
    if (!img) return;
    const parent = img.parentElement;
    if (!parent) return;
    const p = parent.getBoundingClientRect();
    const r = img.getBoundingClientRect();
    rectX = r.left - p.left;
    rectY = r.top - p.top;
    rectW = r.width;
    rectH = r.height;
  }

  $effect(() => {
    if (!img) return;
    recompute();
    const ro = new ResizeObserver(recompute);
    ro.observe(img);
    if (img.parentElement) ro.observe(img.parentElement);
    img.addEventListener('load', recompute);
    window.addEventListener('resize', recompute);
    return () => {
      ro.disconnect();
      img.removeEventListener('load', recompute);
      window.removeEventListener('resize', recompute);
    };
  });

  $effect(() => {
    void ui.zoom;
    void ui.panX;
    void ui.panY;
    if (!img) return;
    const id = requestAnimationFrame(recompute);
    return () => cancelAnimationFrame(id);
  });

  const geomT = $derived.by<GeometryTransform>(() => {
    const g = editor.edits.geometry;
    const sw = editor.meta?.source_w ?? 1;
    const sh = editor.meta?.source_h ?? 1;
    const dw = editor.meta?.width ?? sw;
    const dh = editor.meta?.height ?? sh;
    return {
      inputW: sw,
      inputH: sh,
      rotateQuarter: g.rotate as RotateQuarter,
      flipH: g.flip_h,
      flipV: g.flip_v,
      angleDeg: g.rotate_angle,
      crop: g.crop ?? { x: 0, y: 0, w: 1, h: 1 },
      outputW: dw,
      outputH: dh
    };
  });

  const lensP = $derived.by<LensWarpParams>(() =>
    lensWarpFromEdits(
      editor.edits.lens,
      editor.meta?.source_w ?? 1,
      editor.meta?.source_h ?? 1
    )
  );

  const allIdentity = $derived(geometryIsIdentity(geomT) && !lensWarpActive(lensP));

  const active = $derived<MaskLayer | null>(
    editor.activeLayerId
      ? editor.edits.masks.find((l) => l.id === editor.activeLayerId) ?? null
      : null
  );
  const activeComp = $derived<MaskComponent | null>(
    active && editor.activeMaskComponentId
      ? active.components.find((c) => c.id === editor.activeMaskComponentId) ?? null
      : null
  );
  const isBrush = $derived(!!activeComp && activeComp.kind.kind === 'brush');
  const show = $derived(
    editor.maskOverlayVisible &&
      isBrush &&
      editor.maskPreviewLayerId === null &&
      rectW > 0 &&
      rectH > 0
  );

  $effect(() => {
    if (!show || !canvasEl || !activeComp || activeComp.kind.kind !== 'brush') return;
    void repaint(activeComp.id, activeComp.kind.raster_id);
  });

  function displayUvToSceneUv(du: number, dv: number): [number, number] {
    const m = displayUvToMaskUv(geomT, [du, dv]);
    return maskUvToSceneUv(lensP, m);
  }

  function scenePerDisplayAt(du: number, dv: number): number {
    const eps = 1e-3;
    const s0 = displayUvToSceneUv(du, dv);
    const sx = displayUvToSceneUv(du + eps, dv);
    const sy = displayUvToSceneUv(du, dv + eps);
    const jx = Math.hypot(sx[0] - s0[0], sx[1] - s0[1]) / eps;
    const jy = Math.hypot(sy[0] - s0[0], sy[1] - s0[1]) / eps;
    return Math.max(1e-6, (jx + jy) * 0.5);
  }

  async function repaint(componentId: string, rasterId: string): Promise<void> {
    if (!canvasEl) return;
    const buf = await editor.ensureBrushBuffer(componentId, rasterId);
    if (!canvasEl) return;
    const w = Math.max(1, Math.floor(rectW));
    const h = Math.max(1, Math.floor(rectH));
    if (canvasEl.width !== w) canvasEl.width = w;
    if (canvasEl.height !== h) canvasEl.height = h;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    const color = active ? parseHexColor(active.color) : ([255, 60, 60] as [number, number, number]);
    if (allIdentity) {
      const off = document.createElement('canvas');
      off.width = buf.width;
      off.height = buf.height;
      const offCtx = off.getContext('2d');
      if (!offCtx) return;
      offCtx.putImageData(bufferToImageData(buf, color, 0.6), 0, 0);
      ctx.clearRect(0, 0, w, h);
      ctx.drawImage(off, 0, 0, w, h);
    } else {
      ctx.clearRect(0, 0, w, h);
      const img = ctx.createImageData(w, h);
      sampleBufferToImageData(buf, w, h, color, img.data);
      ctx.putImageData(img, 0, 0);
    }
    lastCompId = componentId;
  }

  function sampleBufferToImageData(
    buf: BrushBuffer,
    w: number,
    h: number,
    color: [number, number, number],
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
        const a = bytes[by * bw + bx];
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
      const dx = px - lastPx;
      const dy = py - lastPy;
      const dist = Math.hypot(dx, dy);
      const radiusPx = editor.brushTool.size * 0.5 * Math.min(rect.width, rect.height);
      const step = Math.max(1, radiusPx * 0.5);
      const n = Math.max(1, Math.ceil(dist / step));
      for (let i = 1; i <= n; i++) {
        const t = i / n;
        stampAtPx(buf, rect, lastPx + dx * t, lastPy + dy * t);
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
    const grad = ctx.createRadialGradient(cxPx, cyPx, inner, cxPx, cyPx, Math.max(inner + 0.5, rPx));
    if (erase) {
      ctx.globalCompositeOperation = 'destination-out';
      grad.addColorStop(0, `rgba(0,0,0,${editor.brushTool.flow * 0.6})`);
      grad.addColorStop(1, 'rgba(0,0,0,0)');
    } else {
      ctx.globalCompositeOperation = 'source-over';
      const [r, g, b] = parseHexColor(active.color);
      grad.addColorStop(0, `rgba(${r},${g},${b},${editor.brushTool.flow * 0.6})`);
      grad.addColorStop(1, `rgba(${r},${g},${b},0)`);
    }
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(cxPx, cyPx, Math.max(0.5, rPx), 0, Math.PI * 2);
    ctx.fill();
    ctx.globalCompositeOperation = 'source-over';
  }

  async function onPointerDown(e: PointerEvent): Promise<void> {
    if (!activeComp || activeComp.kind.kind !== 'brush' || !active) return;
    e.preventDefault();
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
    style="left: {rectX}px; top: {rectY}px; width: {rectW}px; height: {rectH}px; touch-action: none; cursor: crosshair;"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
  ></canvas>
{/if}
