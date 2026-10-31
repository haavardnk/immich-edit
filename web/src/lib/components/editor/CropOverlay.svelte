<script lang="ts">
  import { observeSize } from '#lib/actions/observeSize.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui, type CropGrid } from '#lib/stores/ui.svelte.ts';
  import {
    rotatedBbox,
    aspectRatioFor,
    angleFromLine,
    resizeCrop,
    scaleCropAboutCentre,
    type CropHandle,
    type Point
  } from '#lib/utils/geom.ts';
  import { neutralPerspective, perspectiveCssMatrix } from '#lib/utils/perspective.ts';
  import { stageFrame } from '#lib/editor/geometryStage.ts';
  import PerspectiveCorners from './PerspectiveCorners.svelte';
  import UprightGuides from './UprightGuides.svelte';
  import type { CropRect } from '#lib/types/edits.ts';

  let container = $state<HTMLDivElement | null>(null);
  let stage = $state<HTMLDivElement | null>(null);
  let containerW = $state(0);
  let containerH = $state(0);
  let line = $state<{ from: Point; to: Point } | null>(null);

  const MIN_LINE_PX = 10;
  const GOLDEN = 1 - 2 / (1 + Math.sqrt(5));

  type GuideLine = [number, number, number, number];

  function axisLines(stops: number[]): GuideLine[] {
    return stops.flatMap((t): GuideLine[] => [
      [t, 0, t, 1],
      [0, t, 1, t]
    ]);
  }

  const GUIDE_LINES: Record<CropGrid, GuideLine[]> = {
    thirds: axisLines([1 / 3, 2 / 3]),
    golden: axisLines([GOLDEN, 1 - GOLDEN]),
    diagonal: [
      [0, 0, 1, 1],
      [1, 0, 0, 1]
    ],
    grid: axisLines([1, 2, 3, 4, 5, 6, 7].map((i) => i / 8)),
    off: []
  };

  function measure(): void {
    if (!container) return;
    const rect = container.getBoundingClientRect();
    containerW = rect.width;
    containerH = rect.height;
  }

  const sess = $derived(editor.geometrySession);
  const swapped = $derived(sess ? sess.draftRotate === 90 || sess.draftRotate === 270 : false);
  const sourceW = $derived(sess ? (swapped ? sess.srcH : sess.srcW) : 1);
  const sourceH = $derived(sess ? (swapped ? sess.srcW : sess.srcH) : 1);
  const bbox = $derived(sess ? rotatedBbox(sourceW, sourceH, sess.draftAngle) : { w: 1, h: 1 });
  const scale = $derived(
    Math.min((containerW * 0.92) / Math.max(bbox.w, 1), (containerH * 0.92) / Math.max(bbox.h, 1))
  );
  const bboxW = $derived(bbox.w * scale);
  const bboxH = $derived(bbox.h * scale);
  const imgW = $derived((sess?.srcW ?? 1) * scale);
  const imgH = $derived((sess?.srcH ?? 1) * scale);
  const orientedW = $derived(swapped ? imgH : imgW);
  const orientedH = $derived(swapped ? imgW : imgH);
  const frame = $derived(
    stageFrame({
      bboxW,
      bboxH,
      orientedW,
      orientedH,
      angle: sess?.draftAngle ?? 0,
      perspective: sess?.draftPerspective ?? neutralPerspective()
    })
  );
  const perspCss = $derived(
    sess ? perspectiveCssMatrix(frame.forward, orientedW, orientedH) : 'none'
  );
  const crop = $derived(sess?.draftCrop ?? { x: 0, y: 0, w: 1, h: 1 });
  const cropPx = $derived({
    x: crop.x * bboxW,
    y: crop.y * bboxH,
    w: crop.w * bboxW,
    h: crop.h * bboxH
  });

  type DragKind = 'move' | 'pan' | CropHandle;

  let dragKind = $state<DragKind | null>(null);
  let dragStartX = 0;
  let dragStartY = 0;
  let dragStartCrop: CropRect | null = null;
  let dragLockedRatio: number | null = null;
  let panStartCrop = $state<CropRect | null>(null);
  const panOffset = $derived(
    panStartCrop
      ? { x: (panStartCrop.x - crop.x) * bboxW, y: (panStartCrop.y - crop.y) * bboxH }
      : { x: 0, y: 0 }
  );

  function stagePoint(e: PointerEvent): Point {
    const rect = stage?.getBoundingClientRect();
    return { x: e.clientX - (rect?.left ?? 0), y: e.clientY - (rect?.top ?? 0) };
  }

  function startLine(e: PointerEvent): void {
    e.preventDefault();
    e.stopPropagation();
    const at = stagePoint(e);
    line = { from: at, to: at };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onLineMove(e: PointerEvent): void {
    if (!line) return;
    line = { from: line.from, to: stagePoint(e) };
  }

  function onLineUp(e: PointerEvent): void {
    const drawn = line;
    line = null;
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    if (!drawn || !sess) return;
    const length = Math.hypot(drawn.to.x - drawn.from.x, drawn.to.y - drawn.from.y);
    if (length < MIN_LINE_PX) return;
    editor.updateGeometryDraftAngle(sess.draftAngle + angleFromLine(drawn.from, drawn.to));
    if (ui.geometryTool === 'straighten') ui.geometryTool = null;
    void editor.commitGeometry('Straighten');
  }

  function startDrag(e: PointerEvent, kind: 'move' | CropHandle): void {
    if (kind === 'move' && e.shiftKey) {
      startLine(e);
      return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (!sess) return;
    const panning = kind === 'move' && (e.ctrlKey || e.metaKey);
    dragKind = panning ? 'pan' : kind;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    dragStartCrop = { ...sess.draftCrop };
    panStartCrop = panning ? { ...sess.draftCrop } : null;
    const aspectRatio = aspectRatioFor(sess.draftAspect, sourceW, sourceH);
    dragLockedRatio =
      aspectRatio ??
      (e.shiftKey ? (dragStartCrop.w * bboxW) / Math.max(dragStartCrop.h * bboxH, 1) : null);
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onMove(e: PointerEvent): void {
    if (line) {
      onLineMove(e);
      return;
    }
    if (!dragKind || !dragStartCrop || !sess) return;
    const dx = (e.clientX - dragStartX) / Math.max(bboxW, 1);
    const dy = (e.clientY - dragStartY) / Math.max(bboxH, 1);
    if (dragKind === 'move' || dragKind === 'pan') {
      const sign = dragKind === 'pan' ? -1 : 1;
      editor.updateGeometryDraftCrop({
        ...dragStartCrop,
        x: dragStartCrop.x + sign * dx,
        y: dragStartCrop.y + sign * dy
      });
      return;
    }
    editor.updateGeometryDraftCrop(
      resizeCrop(dragStartCrop, dragKind, dx, dy, {
        ratio: dragLockedRatio,
        fromCentre: e.altKey,
        stageAspect: bboxW / Math.max(bboxH, 1)
      })
    );
  }

  function onWheel(e: WheelEvent): void {
    if (!(e.ctrlKey || e.metaKey) || !sess) return;
    e.preventDefault();
    editor.updateGeometryDraftCrop(
      scaleCropAboutCentre(sess.draftCrop, Math.exp(e.deltaY * 0.002))
    );
    editor.scheduleCropCommit();
  }

  function onUp(e: PointerEvent): void {
    if (line) {
      onLineUp(e);
      return;
    }
    const dragged = dragKind !== null;
    dragKind = null;
    dragStartCrop = null;
    dragLockedRatio = null;
    panStartCrop = null;
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    if (dragged) void editor.commitGeometry('Crop');
  }
</script>

<div
  bind:this={container}
  use:observeSize={measure}
  class="absolute inset-0 flex items-center justify-center select-none"
>
  {#if sess && sess.frameUrl}
    <div
      bind:this={stage}
      class="relative"
      role="presentation"
      style="width: {bboxW}px; height: {bboxH}px; transform: translate({panOffset.x}px, {panOffset.y}px);"
      onwheel={onWheel}
    >
      <img
        src={sess.frameUrl}
        alt=""
        draggable="false"
        class="absolute block"
        style="top: 50%; left: 50%; width: {imgW}px; height: {imgH}px; max-width: none; max-height: none; transform: translate(-50%, -50%) rotate({sess.draftAngle}deg) {perspCss} scaleY({sess.draftFlipV
          ? -1
          : 1}) scaleX({sess.draftFlipH
          ? -1
          : 1}) rotate({sess.draftRotate}deg); transform-origin: center; image-orientation: none;"
      />

      <div
        class="absolute inset-0 pointer-events-none"
        style="clip-path: polygon(
          0 0, 100% 0, 100% 100%, 0 100%, 0 0,
          {(cropPx.x / bboxW) * 100}% {(cropPx.y / bboxH) * 100}%,
          {(cropPx.x / bboxW) * 100}% {((cropPx.y + cropPx.h) / bboxH) * 100}%,
          {((cropPx.x + cropPx.w) / bboxW) * 100}% {((cropPx.y + cropPx.h) / bboxH) * 100}%,
          {((cropPx.x + cropPx.w) / bboxW) * 100}% {(cropPx.y / bboxH) * 100}%,
          {(cropPx.x / bboxW) * 100}% {(cropPx.y / bboxH) * 100}%
        ); background: var(--color-crop-shade);"
      ></div>

      <div
        class="absolute border border-white/90 cursor-move touch-none"
        style="left: {cropPx.x}px; top: {cropPx.y}px; width: {cropPx.w}px; height: {cropPx.h}px;"
        onpointerdown={(e) => startDrag(e, 'move')}
        onpointermove={onMove}
        onpointerup={onUp}
        onpointercancel={onUp}
        role="presentation"
      >
        <svg
          class="absolute inset-0 size-full pointer-events-none"
          viewBox="0 0 1 1"
          preserveAspectRatio="none"
          data-testid="crop-guide"
          data-mode={ui.cropGrid}
          aria-hidden="true"
        >
          {#each GUIDE_LINES[ui.cropGrid] as [x1, y1, x2, y2], i (i)}
            <line
              {x1}
              {y1}
              {x2}
              {y2}
              stroke="rgb(255 255 255 / 0.3)"
              stroke-width="1"
              vector-effect="non-scaling-stroke"
            />
          {/each}
        </svg>
        {#each ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'] as const as h (h)}
          <button
            class="absolute touch-none before:absolute before:inset-0.5 before:rounded-sm before:border before:border-black/60 before:bg-white"
            style="
              width: 16px; height: 16px;
              {h.includes('n') ? 'top: -8px;' : ''}
              {h.includes('s') ? 'bottom: -8px;' : ''}
              {h.includes('w') ? 'left: -8px;' : ''}
              {h.includes('e') ? 'right: -8px;' : ''}
              {h === 'n' || h === 's' ? 'left: calc(50% - 8px);' : ''}
              {h === 'w' || h === 'e' ? 'top: calc(50% - 8px);' : ''}
              cursor: {h === 'n' || h === 's'
              ? 'ns-resize'
              : h === 'e' || h === 'w'
                ? 'ew-resize'
                : h === 'nw' || h === 'se'
                  ? 'nwse-resize'
                  : 'nesw-resize'};
            "
            onpointerdown={(e) => startDrag(e, h)}
            onpointermove={onMove}
            onpointerup={onUp}
            onpointercancel={onUp}
            aria-label="resize {h}"
          ></button>
        {/each}
      </div>

      {#if ui.geometryTool === 'straighten'}
        <div
          class="absolute inset-0 cursor-crosshair touch-none"
          data-testid="straighten-surface"
          role="presentation"
          onpointerdown={startLine}
          onpointermove={onLineMove}
          onpointerup={onLineUp}
          onpointercancel={onLineUp}
        ></div>
      {/if}

      {#if ui.geometryTool === 'upright'}
        <UprightGuides {frame} guides={sess.guides} />
      {/if}

      {#if line}
        <svg
          class="absolute inset-0 pointer-events-none"
          width={bboxW}
          height={bboxH}
          aria-hidden="true"
        >
          <line
            x1={line.from.x}
            y1={line.from.y}
            x2={line.to.x}
            y2={line.to.y}
            stroke="var(--color-primary)"
            stroke-width="2"
          />
        </svg>
      {/if}

      {#if ui.geometryTool === 'corners'}
        <PerspectiveCorners {frame} perspective={sess.draftPerspective} />
      {/if}
    </div>
  {/if}
</div>
