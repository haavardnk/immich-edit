<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { isFullCrop, type MaskComponent, type MaskLayer } from '$lib/types/edits';
  import { bufferToImageData, parseHexColor, stampBuffer } from '$lib/utils/brush';

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

  const geomIdentity = $derived.by(() => {
    const g = editor.edits.geometry;
    return (
      g.rotate === 0 &&
      g.rotate_angle === 0 &&
      !g.flip_h &&
      !g.flip_v &&
      isFullCrop(g.crop)
    );
  });

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
      geomIdentity &&
      isBrush &&
      editor.maskPreviewLayerId === null &&
      rectW > 0 &&
      rectH > 0
  );

  $effect(() => {
    if (!show || !canvasEl || !activeComp || activeComp.kind.kind !== 'brush') return;
    void repaint(activeComp.id, activeComp.kind.raster_id);
  });

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
    const off = document.createElement('canvas');
    off.width = buf.width;
    off.height = buf.height;
    const offCtx = off.getContext('2d');
    if (!offCtx) return;
    const color = active ? parseHexColor(active.color) : ([255, 60, 60] as [number, number, number]);
    offCtx.putImageData(bufferToImageData(buf, color, 0.6), 0, 0);
    ctx.clearRect(0, 0, w, h);
    ctx.drawImage(off, 0, 0, w, h);
    lastCompId = componentId;
  }

  function stampAt(e: PointerEvent): void {
    if (!canvasEl || !active || !activeComp || activeComp.kind.kind !== 'brush') return;
    const buf = editor.brushBuffers[activeComp.id];
    if (!buf) return;
    const rect = canvasEl.getBoundingClientRect();
    const nx = (e.clientX - rect.left) / Math.max(1, rect.width);
    const ny = (e.clientY - rect.top) / Math.max(1, rect.height);
    const radiusN = editor.brushTool.size * 0.5;
    const cxBuf = nx * buf.width;
    const cyBuf = ny * buf.height;
    const radiusBuf = radiusN * Math.min(buf.width, buf.height);
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
    (e.currentTarget as Element).releasePointerCapture?.(e.pointerId);
    if (!active || !activeComp) return;
    await editor.commitBrushStroke(active.id, activeComp.id);
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
