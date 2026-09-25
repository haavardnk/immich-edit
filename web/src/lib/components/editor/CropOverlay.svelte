<script lang="ts">
  import { observeSize } from '$lib/actions/observeSize';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui, type CropGrid } from '$lib/stores/ui.svelte';
  import {
    rotatedBbox,
    aspectRatioFor,
    degToRad,
    angleFromLine,
    resizeCrop,
    scaleCropAboutCentre,
    type CropHandle,
    type Point
  } from '$lib/utils/geom';
  import {
    cornerOffsetsFor,
    mat3Apply,
    perspectiveCssMatrix,
    perspectiveForward
  } from '$lib/utils/perspective';
  import type { CropRect } from '$lib/types/edits';

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
  const perspCss = $derived(
    sess
      ? perspectiveCssMatrix(perspectiveForward(sess.draftPerspective), orientedW, orientedH)
      : 'none'
  );
  const cornerHandles = $derived.by<{ x: number; y: number }[]>(() => {
    if (!sess) return [];
    const f = perspectiveForward(sess.draftPerspective);
    const a = degToRad(sess.draftAngle);
    const cos = Math.cos(a);
    const sin = Math.sin(a);
    const base: [number, number][] = [
      [0, 0],
      [1, 0],
      [1, 1],
      [0, 1]
    ];
    return base.map((b) => {
      const uv = mat3Apply(f, b);
      const px = (uv[0] - 0.5) * orientedW;
      const py = (uv[1] - 0.5) * orientedH;
      return { x: px * cos - py * sin + bboxW / 2, y: px * sin + py * cos + bboxH / 2 };
    });
  });
  const quadPoints = $derived(cornerHandles.map((c) => `${c.x},${c.y}`).join(' '));
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
    ui.straightening = false;
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
  }

  function onUp(e: PointerEvent): void {
    if (line) {
      onLineUp(e);
      return;
    }
    dragKind = null;
    dragStartCrop = null;
    dragLockedRatio = null;
    panStartCrop = null;
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
  }

  let cornerDrag = $state<number | null>(null);

  function startCornerDrag(e: PointerEvent, index: number): void {
    e.preventDefault();
    e.stopPropagation();
    cornerDrag = index;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onCornerMove(e: PointerEvent): void {
    if (cornerDrag === null || !sess || !container) return;
    e.stopPropagation();
    const rect = (e.currentTarget as HTMLElement).offsetParent?.getBoundingClientRect();
    if (!rect) return;
    const a = degToRad(sess.draftAngle);
    const cos = Math.cos(a);
    const sin = Math.sin(a);
    const dx = e.clientX - rect.left - bboxW / 2;
    const dy = e.clientY - rect.top - bboxH / 2;
    const px = dx * cos + dy * sin;
    const py = -dx * sin + dy * cos;
    const uv: [number, number] = [
      px / Math.max(orientedW, 1) + 0.5,
      py / Math.max(orientedH, 1) + 0.5
    ];
    editor.updateGeometryDraftPerspective({
      corners: cornerOffsetsFor(sess.draftPerspective, cornerDrag, uv)
    });
  }

  function onCornerUp(e: PointerEvent): void {
    e.stopPropagation();
    cornerDrag = null;
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
  }
</script>

<div
  bind:this={container}
  use:observeSize={measure}
  class="absolute inset-0 flex items-center justify-center select-none"
>
  {#if sess && sess.pinnedReady && sess.pinnedUrl}
    <div
      bind:this={stage}
      class="relative"
      role="presentation"
      style="width: {bboxW}px; height: {bboxH}px; transform: translate({panOffset.x}px, {panOffset.y}px);"
      onwheel={onWheel}
    >
      <img
        src={sess.pinnedUrl}
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

      {#if ui.straightening}
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

      {#if ui.perspectiveCorners}
        <svg
          class="absolute inset-0 pointer-events-none"
          width={bboxW}
          height={bboxH}
          aria-hidden="true"
        >
          <polygon
            points={quadPoints}
            fill="none"
            stroke="var(--color-primary)"
            stroke-width="1"
            stroke-dasharray="4 3"
          />
        </svg>
        {#each cornerHandles as c, i (i)}
          <button
            class="absolute cursor-grab rounded-full border border-black/60 bg-primary active:cursor-grabbing"
            style="width: 14px; height: 14px; left: {c.x - 7}px; top: {c.y - 7}px;"
            onpointerdown={(e) => startCornerDrag(e, i)}
            onpointermove={onCornerMove}
            onpointerup={onCornerUp}
            onpointercancel={onCornerUp}
            aria-label="perspective corner {i + 1}"
          ></button>
        {/each}
      {/if}
    </div>
  {/if}
</div>
