<script lang="ts">
  import { observeSize } from '#lib/actions/observeSize.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
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
  import CropGuide from './crop/CropGuide.svelte';
  import CropHandles from './crop/CropHandles.svelte';
  import PhotoPill from './PhotoPill.svelte';
  import { cropDragReadout } from '#lib/panels/geometry/cropReadout.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { CropRect } from '#lib/types/edits.ts';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { elementPoint } from '#lib/gestures/point.ts';
  import { boxHandleHit } from '#lib/editor/handles.ts';
  import { handleHit } from '#lib/editor/handleHit.svelte.ts';

  let container = $state<HTMLDivElement | null>(null);
  let stage = $state<HTMLDivElement | null>(null);
  let containerW = $state(0);
  let containerH = $state(0);
  let line = $state<{ from: Point; to: Point } | null>(null);

  const MIN_LINE_PX = 10;

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

  type CropDrag =
    | { kind: 'line' }
    | {
        kind: DragKind;
        startX: number;
        startY: number;
        startCrop: CropRect;
        ratio: number | null;
      };

  const hit = $derived(boxHandleHit(handleHit(), cropPx.w, cropPx.h));
  const touch = $derived(layout.mode === 'touch');
  let active = $state<DragKind | null>(null);
  const activeHandle = $derived(
    active === null || active === 'move' || active === 'pan' ? null : active
  );
  const dragReadout = $derived(active && !touch ? cropDragReadout() : null);
  let panStartCrop = $state<CropRect | null>(null);
  const panOffset = $derived(
    panStartCrop
      ? { x: (panStartCrop.x - crop.x) * bboxW, y: (panStartCrop.y - crop.y) * bboxH }
      : { x: 0, y: 0 }
  );

  function stagePoint(e: PointerEvent): Point {
    return stage ? elementPoint(e, stage) : { x: e.clientX, y: e.clientY };
  }

  function startLine(e: PointerEvent): CropDrag {
    e.preventDefault();
    e.stopPropagation();
    const at = stagePoint(e);
    line = { from: at, to: at };
    return { kind: 'line' };
  }

  function endLine(): void {
    const drawn = line;
    line = null;
    if (!drawn || !sess) return;
    const length = Math.hypot(drawn.to.x - drawn.from.x, drawn.to.y - drawn.from.y);
    if (length < MIN_LINE_PX) return;
    editor.updateGeometryDraftAngle(sess.draftAngle + angleFromLine(drawn.from, drawn.to));
    if (ui.geometryTool === 'straighten') ui.geometryTool = null;
    void editor.commitGeometry('Straighten');
  }

  function startCrop(e: PointerEvent, kind: 'line' | 'move' | CropHandle): CropDrag | null {
    if (kind === 'move' && e.button === 2) return startLine(e);
    if (e.button !== 0) return null;
    if (kind === 'line' || (kind === 'move' && held('straighten', e))) return startLine(e);
    e.preventDefault();
    e.stopPropagation();
    if (!sess) return null;
    const panning = kind === 'move' && held('pan', e);
    const startCrop = { ...sess.draftCrop };
    panStartCrop = panning ? { ...sess.draftCrop } : null;
    active = panning ? 'pan' : kind;
    const aspectRatio = aspectRatioFor(sess.draftAspect, sourceW, sourceH);
    const ratio =
      aspectRatio ??
      (held('constrain', e) ? (startCrop.w * bboxW) / Math.max(startCrop.h * bboxH, 1) : null);
    return {
      kind: panning ? 'pan' : kind,
      startX: e.clientX,
      startY: e.clientY,
      startCrop,
      ratio
    };
  }

  function moveCrop(e: PointerEvent, drag: CropDrag): void {
    if (drag.kind === 'line') {
      if (line) line = { from: line.from, to: stagePoint(e) };
      return;
    }
    if (!sess) return;
    const dx = (e.clientX - drag.startX) / Math.max(bboxW, 1);
    const dy = (e.clientY - drag.startY) / Math.max(bboxH, 1);
    if (drag.kind === 'move' || drag.kind === 'pan') {
      const sign = drag.kind === 'pan' ? -1 : 1;
      editor.updateGeometryDraftCrop({
        ...drag.startCrop,
        x: drag.startCrop.x + sign * dx,
        y: drag.startCrop.y + sign * dy
      });
      return;
    }
    editor.updateGeometryDraftCrop(
      resizeCrop(drag.startCrop, drag.kind, dx, dy, {
        ratio: drag.ratio,
        fromCentre: held('fromCentre', e),
        stageAspect: bboxW / Math.max(bboxH, 1)
      })
    );
  }

  function endCrop(_: PointerEvent, drag: CropDrag): void {
    if (drag.kind === 'line') {
      endLine();
      return;
    }
    panStartCrop = null;
    active = null;
    void editor.commitGeometry('Crop');
  }

  const cropDrag = pointerDrag<CropDrag, 'line' | 'move' | CropHandle>({
    start: startCrop,
    move: moveCrop,
    end: endCrop
  });

  function onWheel(e: WheelEvent): void {
    if (!held('zoom', e) || !sess) return;
    e.preventDefault();
    editor.updateGeometryDraftCrop(
      scaleCropAboutCentre(sess.draftCrop, Math.exp(e.deltaY * 0.002))
    );
    editor.scheduleCropCommit();
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
        {...cropDrag.on('move')}
        oncontextmenu={(e) => e.preventDefault()}
        role="presentation"
      >
        <CropGuide
          grid={ui.cropGrid}
          turn={ui.cropGuideTurn}
          aspect={cropPx.w / Math.max(cropPx.h, 1)}
        />
        <CropHandles {hit} {touch} active={activeHandle} onbegin={(e, h) => cropDrag.begin(e, h)} />
        {#if dragReadout}
          <PhotoPill {...dragReadout} placement="centre" />
        {/if}
      </div>

      {#if ui.geometryTool === 'straighten'}
        <div
          class="absolute inset-0 cursor-crosshair touch-none"
          data-testid="straighten-surface"
          role="presentation"
          {...cropDrag.on('line')}
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
            stroke="var(--color-image-accent)"
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
