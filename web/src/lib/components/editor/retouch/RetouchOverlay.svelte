<script lang="ts">
  import { RETOUCH_SOURCE_HIT } from '#lib/editor/handles.ts';
  import { RETOUCH_HARDNESS, RETOUCH_SIZE } from '#lib/editor/params/brush.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { normalizedPoint } from '#lib/gestures/point.ts';
  import { held, keyHeld, latched } from '#lib/input/modifiers.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { retouchView } from '#lib/stores/editor/retouchView.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { MAX_RETOUCH_POINTS, type RetouchStroke, type Vec2f } from '#lib/types/edits.ts';
  import { brushWheel } from '#lib/utils/brushSize.ts';
  import {
    displayUvToSceneUv,
    scenePerDisplayAt,
    sceneUvToDisplayUv,
    steppedSegment,
    viewTransform
  } from '#lib/utils/canvasCoords.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { v4 as uuidv4 } from 'uuid';
  import { distToPolyline, type Px } from './retouchDraw';
  import { drawRetouch } from './retouchScene';

  let {
    img
  }: {
    img: PreviewSurface | null;
  } = $props();

  const rect = imageRect(() => img);
  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let layerEl: HTMLCanvasElement | null = null;
  let drawPts = $state<Px[]>([]);
  let drawing = $state(false);
  let dragSourceId = $state<string | null>(null);
  let dragStroke = $state<{
    id: string;
    from: [number, number];
    points: Vec2f[];
    source: Vec2f;
    reach: { left: number; right: number; top: number; bottom: number };
    moved: boolean;
  } | null>(null);
  let hoverMovable = $state(false);
  let hoverPaints = $state(false);
  let hover = $state<Px | null>(null);
  let sampleKey = $state(false);
  let strokeOffset: Px | null = null;

  const view = $derived(viewTransform(editor.edits, editor.meta ?? null, editor.lensView));
  const strokes = $derived(editor.edits.retouch);
  const anchor = $derived(editor.retouchAnchor);
  const show = $derived(ui.editorTab === 'retouch' && rect.w > 0 && rect.h > 0);
  const sampleHeld = $derived(sampleKey || latched('sample'));
  const picking = $derived(sampleHeld || !anchor || editor.retouchSampling);
  const canPick = $derived(!!anchor || sampleHeld || editor.retouchSampling);

  function minSide(): number {
    return Math.max(1, Math.min(rect.w, rect.h));
  }

  function displayRadius(sceneRadius: number, nx: number, ny: number): number {
    const j = scenePerDisplayAt(view, nx, ny);
    return (sceneRadius / j) * minSide();
  }

  function hitSourceHandle(nx: number, ny: number): string | null {
    const s = strokes.find((r) => r.id === editor.activeRetouchId);
    if (!s) return null;
    const [sx, sy] = sceneUvToDisplayUv(view, s.source.x, s.source.y);
    const r = displayRadius(s.radius, sx, sy) / minSide();
    const dx = (nx - sx) * rect.w;
    const dy = (ny - sy) * rect.h;
    return Math.hypot(dx, dy) <= Math.max(RETOUCH_SOURCE_HIT, r * minSide()) ? s.id : null;
  }

  function hitStroke(nx: number, ny: number): RetouchStroke | null {
    for (let i = strokes.length - 1; i >= 0; i--) {
      const s = strokes[i];
      if (!s) continue;
      const pts = s.points.map((p) => {
        const [x, y] = sceneUvToDisplayUv(view, p.x, p.y);
        return [x * rect.w, y * rect.h] as Px;
      });
      const rPx = displayRadius(s.radius, nx, ny);
      if (distToPolyline(pts, nx * rect.w, ny * rect.h) <= rPx) return s;
    }
    return null;
  }

  function normalise(e: PointerEvent): Px {
    if (!canvasEl) return [0, 0];
    const p = normalizedPoint(e, canvasEl);
    return [p.x, p.y];
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

  function startPointer(e: PointerEvent): true | null {
    if (!canvasEl || e.button !== 0) return null;
    e.preventDefault();
    e.stopPropagation();
    const [nx, ny] = normalise(e);
    if (held('sample', e) || editor.retouchSampling) {
      const s = displayUvToSceneUv(view, nx, ny);
      editor.setRetouchAnchor({ x: s[0], y: s[1] });
      return null;
    }
    pickTarget(nx, ny);
    return true;
  }

  function pickTarget(nx: number, ny: number): void {
    const handle = hitSourceHandle(nx, ny);
    if (handle) {
      dragSourceId = handle;
      return;
    }
    const hit = hitStroke(nx, ny);
    if (hit && hit.id === editor.activeRetouchId) {
      const all = [...hit.points, hit.source];
      dragStroke = {
        id: hit.id,
        from: displayUvToSceneUv(view, nx, ny),
        points: hit.points.map((p) => ({ ...p })),
        source: { ...hit.source },
        reach: {
          left: Math.min(...all.map((p) => p.x)),
          right: 1 - Math.max(...all.map((p) => p.x)),
          top: Math.min(...all.map((p) => p.y)),
          bottom: 1 - Math.max(...all.map((p) => p.y))
        },
        moved: false
      };
      return;
    }
    if (hit) {
      editor.activeRetouchId = hit.id;
      return;
    }
    if (editor.retouchFull) {
      retouchView.flashLimit();
      return;
    }
    if (!anchor) {
      editor.markRetouchSourceMissing();
      return;
    }
    const start = displayUvToSceneUv(view, nx, ny);
    strokeOffset = [anchor.x - start[0], anchor.y - start[1]];
    drawing = true;
    drawPts = [[nx, ny]];
  }

  function trackHover(nx: number, ny: number): void {
    const handle = !!hitSourceHandle(nx, ny);
    const hit = handle ? null : hitStroke(nx, ny);
    hoverMovable = !!editor.activeRetouchId && hit?.id === editor.activeRetouchId;
    hoverPaints = !handle && !hit;
  }

  function trackPointer(e: PointerEvent): void {
    const [nx, ny] = normalise(e);
    hover = [nx, ny];
    sampleKey = keyHeld('sample', e);
    if (dragSourceId) {
      const s = displayUvToSceneUv(view, nx, ny);
      void editor.setRetouchStroke(dragSourceId, { source: { x: s[0], y: s[1] } }, false);
      return;
    }
    if (dragStroke) {
      const [sx, sy] = displayUvToSceneUv(view, nx, ny);
      const { reach } = dragStroke;
      const dx = Math.min(reach.right, Math.max(-reach.left, sx - dragStroke.from[0]));
      const dy = Math.min(reach.bottom, Math.max(-reach.top, sy - dragStroke.from[1]));
      dragStroke.moved = true;
      void editor.setRetouchStroke(
        dragStroke.id,
        {
          points: dragStroke.points.map((p) => ({ x: p.x + dx, y: p.y + dy })),
          source: { x: dragStroke.source.x + dx, y: dragStroke.source.y + dy }
        },
        false
      );
      return;
    }
    if (!drawing) {
      trackHover(nx, ny);
      return;
    }
    if (drawPts.length >= MAX_RETOUCH_POINTS) return;
    const last = drawPts[drawPts.length - 1];
    if (!last) return;
    const radiusN = editor.retouchTool.size / scenePerDisplayAt(view, nx, ny);
    const step = Math.max(0.002, radiusN * 0.5);
    if (Math.hypot(nx - last[0], ny - last[1]) < step) return;
    const added = steppedSegment(last[0], last[1], nx, ny, step);
    drawPts = [...drawPts, ...added].slice(0, MAX_RETOUCH_POINTS);
  }

  function endPointer(): void {
    if (dragSourceId) {
      const id = dragSourceId;
      dragSourceId = null;
      editor.activeRetouchId = id;
      void editor.commitRetouch();
      return;
    }
    if (dragStroke) {
      const moved = dragStroke.moved;
      dragStroke = null;
      if (moved) void editor.commitRetouch();
      return;
    }
    if (drawing) void finishStroke();
  }

  const pointer = pointerDrag<true>({
    start: startPointer,
    move: trackPointer,
    hover: trackPointer,
    end: endPointer
  });

  function onWheel(e: WheelEvent): void {
    brushWheel(
      e,
      editor.retouchTool,
      { size: RETOUCH_SIZE, hardness: RETOUCH_HARDNESS },
      editor.setRetouchTool
    );
  }

  function draw(): void {
    if (!canvasEl) return;
    const w = Math.max(1, Math.floor(rect.w));
    const h = Math.max(1, Math.floor(rect.h));
    if (canvasEl.width !== w) canvasEl.width = w;
    if (canvasEl.height !== h) canvasEl.height = h;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    layerEl ??= document.createElement('canvas');
    const styles = getComputedStyle(canvasEl);
    drawRetouch(ctx, layerEl, {
      view,
      strokes,
      activeId: editor.activeRetouchId,
      anchor,
      toolSize: editor.retouchTool.size,
      drawPts: drawing ? drawPts : [],
      strokeOffset,
      hover,
      picking,
      ghost: hoverPaints && !editor.retouchFull,
      colours: {
        active: styles.getPropertyValue('--color-retouch-active').trim(),
        source: styles.getPropertyValue('--color-retouch-source').trim(),
        light: styles.getPropertyValue('--color-image-light').trim()
      },
      radius: displayRadius
    });
  }

  $effect(() => {
    void strokes;
    void drawPts;
    void hover;
    void hoverPaints;
    void picking;
    void anchor;
    void view;
    void rect.w;
    void rect.h;
    void editor.activeRetouchId;
    void editor.retouchFull;
    void editor.retouchTool.size;
    draw();
  });
</script>

{#if show}
  <canvas
    bind:this={canvasEl}
    aria-label="retouch canvas"
    class="absolute"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none; cursor: {dragStroke
      ? 'grabbing'
      : hoverMovable
        ? 'move'
        : canPick
          ? 'crosshair'
          : 'not-allowed'};"
    {...pointer.on()}
    onpointerleave={() => (hover = null)}
    onwheel={onWheel}
  ></canvas>
{/if}
