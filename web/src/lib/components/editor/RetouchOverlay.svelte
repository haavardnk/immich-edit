<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { MAX_RETOUCH_POINTS, type RetouchStroke, type Vec2f } from '$lib/types/edits';
  import {
    displayUvToSceneUv,
    scenePerDisplayAt,
    sceneUvToDisplayUv,
    steppedSegment,
    viewTransform
  } from '$lib/utils/canvasCoords';
  import { v4 as uuidv4 } from 'uuid';

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
  let layerEl: HTMLCanvasElement | null = null;
  let drawPts = $state<[number, number][]>([]);
  let drawing = $state(false);
  let dragSourceId = $state<string | null>(null);
  let hover = $state<[number, number] | null>(null);
  let hoverAlt = $state(false);
  let strokeOffset: [number, number] | null = null;

  const view = $derived(viewTransform(editor.edits, editor.meta ?? null));
  const strokes = $derived(editor.edits.retouch);
  const anchor = $derived(editor.retouchAnchor);
  const show = $derived(ui.editorTab === 'retouch' && rectW > 0 && rectH > 0);

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

  function minSide(): number {
    return Math.max(1, Math.min(rectW, rectH));
  }

  function displayRadius(sceneRadius: number, nx: number, ny: number): number {
    const j = scenePerDisplayAt(view, nx, ny);
    return (sceneRadius / j) * minSide();
  }

  function strokeDisplayPoints(s: RetouchStroke): [number, number][] {
    return s.points.map((p) => sceneUvToDisplayUv(view, p.x, p.y));
  }

  function distToPolyline(pts: [number, number][], x: number, y: number): number {
    let best = Infinity;
    for (let i = 0; i < pts.length; i++) {
      const a = pts[i];
      const b = i + 1 < pts.length ? pts[i + 1] : pts[i];
      const dx = b[0] - a[0];
      const dy = b[1] - a[1];
      const len2 = dx * dx + dy * dy;
      const t = len2 > 0 ? Math.max(0, Math.min(1, ((x - a[0]) * dx + (y - a[1]) * dy) / len2)) : 0;
      best = Math.min(best, Math.hypot(x - (a[0] + dx * t), y - (a[1] + dy * t)));
    }
    return best;
  }

  function hitSourceHandle(nx: number, ny: number): string | null {
    const s = strokes.find((r) => r.id === editor.activeRetouchId);
    if (!s) return null;
    const [sx, sy] = sceneUvToDisplayUv(view, s.source.x, s.source.y);
    const r = displayRadius(s.radius, sx, sy) / minSide();
    const dx = (nx - sx) * rectW;
    const dy = (ny - sy) * rectH;
    return Math.hypot(dx, dy) <= Math.max(8, r * minSide()) ? s.id : null;
  }

  function hitStroke(nx: number, ny: number): RetouchStroke | null {
    for (let i = strokes.length - 1; i >= 0; i--) {
      const s = strokes[i];
      const pts = strokeDisplayPoints(s).map(
        ([x, y]) => [x * rectW, y * rectH] as [number, number]
      );
      const rPx = displayRadius(s.radius, nx, ny);
      if (distToPolyline(pts, nx * rectW, ny * rectH) <= rPx) return s;
    }
    return null;
  }

  function normalise(e: PointerEvent): [number, number] {
    if (!canvasEl) return [0, 0];
    const rect = canvasEl.getBoundingClientRect();
    return [
      (e.clientX - rect.left) / Math.max(1, rect.width),
      (e.clientY - rect.top) / Math.max(1, rect.height)
    ];
  }

  async function finishStroke(): Promise<void> {
    const pts = drawPts;
    const offset = strokeOffset;
    drawing = false;
    drawPts = [];
    strokeOffset = null;
    if (pts.length === 0 || !offset) return;
    const scenePts: Vec2f[] = pts.map((p) => {
      const s = displayUvToSceneUv(view, p[0], p[1]);
      return { x: s[0], y: s[1] };
    });
    const cx = scenePts.reduce((a, p) => a + p.x, 0) / scenePts.length;
    const cy = scenePts.reduce((a, p) => a + p.y, 0) / scenePts.length;
    await editor.addRetouchStroke({
      id: uuidv4(),
      mode: editor.retouchTool.mode,
      points: scenePts,
      radius: editor.retouchTool.size,
      hardness: editor.retouchTool.hardness,
      opacity: editor.retouchTool.opacity,
      source: { x: cx + offset[0], y: cy + offset[1] },
      enabled: true
    });
  }

  function onPointerDown(e: PointerEvent): void {
    if (!canvasEl || e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    const [nx, ny] = normalise(e);
    if (e.altKey) {
      const s = displayUvToSceneUv(view, nx, ny);
      editor.retouchAnchor = { x: s[0], y: s[1] };
      return;
    }
    canvasEl.setPointerCapture(e.pointerId);
    const handle = hitSourceHandle(nx, ny);
    if (handle) {
      dragSourceId = handle;
      return;
    }
    const hit = hitStroke(nx, ny);
    if (hit) {
      editor.activeRetouchId = hit.id;
      return;
    }
    if (editor.retouchFull || !anchor) return;
    const start = displayUvToSceneUv(view, nx, ny);
    strokeOffset = [anchor.x - start[0], anchor.y - start[1]];
    drawing = true;
    drawPts = [[nx, ny]];
  }

  function onPointerMove(e: PointerEvent): void {
    const [nx, ny] = normalise(e);
    hover = [nx, ny];
    hoverAlt = e.altKey;
    if (dragSourceId) {
      const s = displayUvToSceneUv(view, nx, ny);
      void editor.setRetouchStroke(dragSourceId, { source: { x: s[0], y: s[1] } }, false);
      return;
    }
    if (!drawing) return;
    if (drawPts.length >= MAX_RETOUCH_POINTS) return;
    const last = drawPts[drawPts.length - 1];
    const radiusN = editor.retouchTool.size / scenePerDisplayAt(view, nx, ny);
    const step = Math.max(0.002, radiusN * 0.5);
    if (Math.hypot(nx - last[0], ny - last[1]) < step) return;
    const added = steppedSegment(last[0], last[1], nx, ny, step);
    drawPts = [...drawPts, ...added].slice(0, MAX_RETOUCH_POINTS);
  }

  function onPointerUp(e: PointerEvent): void {
    canvasEl?.releasePointerCapture?.(e.pointerId);
    if (dragSourceId) {
      const id = dragSourceId;
      dragSourceId = null;
      editor.activeRetouchId = id;
      void editor.commitRetouch();
      return;
    }
    if (drawing) void finishStroke();
  }

  function onPointerLeave(): void {
    hover = null;
  }

  function ring(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    r: number,
    colour: string,
    dashed: boolean
  ): void {
    ctx.save();
    ctx.setLineDash(dashed ? [4, 3] : []);
    ctx.strokeStyle = colour;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(x, y, Math.max(1, r), 0, Math.PI * 2);
    ctx.stroke();
    ctx.restore();
  }

  function tracePath(ctx: CanvasRenderingContext2D, pts: [number, number][], rPx: number): void {
    const r = Math.max(0.5, rPx);
    if (pts.length === 1) {
      ctx.beginPath();
      ctx.arc(pts[0][0], pts[0][1], r, 0, Math.PI * 2);
      ctx.fill();
      return;
    }
    ctx.lineWidth = r * 2;
    ctx.beginPath();
    ctx.moveTo(pts[0][0], pts[0][1]);
    for (const p of pts.slice(1)) ctx.lineTo(p[0], p[1]);
    ctx.stroke();
  }

  function paintPath(
    ctx: CanvasRenderingContext2D,
    pts: [number, number][],
    rPx: number,
    fill: string,
    line: string,
    lineAlpha: number
  ): void {
    const w = ctx.canvas.width;
    const h = ctx.canvas.height;
    ctx.save();
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.fillStyle = fill;
    ctx.strokeStyle = fill;
    tracePath(ctx, pts, rPx);
    ctx.restore();

    if (!layerEl) layerEl = document.createElement('canvas');
    if (layerEl.width !== w) layerEl.width = w;
    if (layerEl.height !== h) layerEl.height = h;
    const lc = layerEl.getContext('2d');
    if (!lc) return;
    lc.clearRect(0, 0, w, h);
    lc.lineCap = 'round';
    lc.lineJoin = 'round';
    lc.fillStyle = line;
    lc.strokeStyle = line;
    tracePath(lc, pts, rPx + 0.75);
    lc.globalCompositeOperation = 'destination-out';
    lc.fillStyle = '#000';
    lc.strokeStyle = '#000';
    tracePath(lc, pts, rPx - 0.75);
    lc.globalCompositeOperation = 'source-over';

    ctx.save();
    ctx.globalAlpha = lineAlpha;
    ctx.drawImage(layerEl, 0, 0);
    ctx.restore();
  }

  function draw(): void {
    if (!canvasEl) return;
    const w = Math.max(1, Math.floor(rectW));
    const h = Math.max(1, Math.floor(rectH));
    if (canvasEl.width !== w) canvasEl.width = w;
    if (canvasEl.height !== h) canvasEl.height = h;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    ctx.clearRect(0, 0, w, h);

    for (const s of strokes) {
      if (!s.enabled) continue;
      const pts = strokeDisplayPoints(s).map(([x, y]) => [x * w, y * h] as [number, number]);
      if (pts.length === 0) continue;
      const active = s.id === editor.activeRetouchId;
      const rPx = displayRadius(s.radius, pts[0][0] / w, pts[0][1] / h);
      paintPath(
        ctx,
        pts,
        rPx,
        active ? 'rgba(66,165,245,0.2)' : 'rgba(255,255,255,0.05)',
        active ? '#42a5f5' : '#ffffff',
        active ? 0.9 : 0.45
      );
      if (!active) continue;
      const [sxN, syN] = sceneUvToDisplayUv(view, s.source.x, s.source.y);
      const sx = sxN * w;
      const sy = syN * h;
      ctx.save();
      ctx.strokeStyle = 'rgba(66,165,245,0.5)';
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(pts[0][0], pts[0][1]);
      ctx.lineTo(sx, sy);
      ctx.stroke();
      ctx.restore();
      ring(ctx, sx, sy, rPx, 'rgba(120,220,140,0.95)', true);
    }

    if (anchor && !drawing) {
      const [axN, ayN] = sceneUvToDisplayUv(view, anchor.x, anchor.y);
      const rPx = displayRadius(editor.retouchTool.size, axN, ayN);
      ring(ctx, axN * w, ayN * h, rPx, 'rgba(120,220,140,0.95)', true);
      crosshair(ctx, axN * w, ayN * h, 'rgba(120,220,140,0.95)');
    }

    if (drawing && drawPts.length > 0) {
      const pts = drawPts.map(([x, y]) => [x * w, y * h] as [number, number]);
      const rPx = displayRadius(editor.retouchTool.size, drawPts[0][0], drawPts[0][1]);
      paintPath(ctx, pts, rPx, 'rgba(66,165,245,0.25)', '#42a5f5', 0.9);
      if (strokeOffset) {
        const last = drawPts[drawPts.length - 1];
        const sc = displayUvToSceneUv(view, last[0], last[1]);
        const [sxN, syN] = sceneUvToDisplayUv(
          view,
          sc[0] + strokeOffset[0],
          sc[1] + strokeOffset[1]
        );
        ring(ctx, sxN * w, syN * h, rPx, 'rgba(120,220,140,0.95)', true);
        crosshair(ctx, sxN * w, syN * h, 'rgba(120,220,140,0.95)');
      }
      return;
    }

    if (!hover) return;
    const rPx = displayRadius(editor.retouchTool.size, hover[0], hover[1]);
    const picking = hoverAlt || !anchor;
    ring(
      ctx,
      hover[0] * w,
      hover[1] * h,
      rPx,
      picking ? 'rgba(120,220,140,0.9)' : 'rgba(255,255,255,0.8)',
      picking
    );
    if (picking) crosshair(ctx, hover[0] * w, hover[1] * h, 'rgba(120,220,140,0.9)');
  }

  function crosshair(ctx: CanvasRenderingContext2D, x: number, y: number, colour: string): void {
    ctx.save();
    ctx.strokeStyle = colour;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(x - 5, y);
    ctx.lineTo(x + 5, y);
    ctx.moveTo(x, y - 5);
    ctx.lineTo(x, y + 5);
    ctx.stroke();
    ctx.restore();
  }

  $effect(() => {
    void strokes;
    void drawPts;
    void hover;
    void hoverAlt;
    void anchor;
    void view;
    void rectW;
    void rectH;
    void editor.activeRetouchId;
    void editor.retouchTool.size;
    draw();
  });
</script>

{#if show}
  <canvas
    bind:this={canvasEl}
    aria-label="retouch canvas"
    class="absolute"
    style="left: {rectX}px; top: {rectY}px; width: {rectW}px; height: {rectH}px; touch-action: none; cursor: crosshair;"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
    onpointerleave={onPointerLeave}
  ></canvas>
{/if}
