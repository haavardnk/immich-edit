<script lang="ts">
  import { observeSize } from '$lib/actions/observeSize';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { rotatedBbox, aspectRatioFor, degToRad } from '$lib/utils/geom';
  import {
    cornerOffsetsFor,
    mat3Apply,
    perspectiveCssMatrix,
    perspectiveForward
  } from '$lib/utils/perspective';
  import type { CropRect } from '$lib/types/edits';

  let container = $state<HTMLDivElement | null>(null);
  let containerW = $state(0);
  let containerH = $state(0);

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

  type DragKind = 'move' | 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

  let dragKind = $state<DragKind | null>(null);
  let dragStartX = 0;
  let dragStartY = 0;
  let dragStartCrop: CropRect | null = null;

  function startDrag(e: PointerEvent, kind: DragKind): void {
    e.preventDefault();
    e.stopPropagation();
    if (!sess) return;
    dragKind = kind;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    dragStartCrop = { ...sess.draftCrop };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onMove(e: PointerEvent): void {
    if (!dragKind || !dragStartCrop || !sess) return;
    const dx = (e.clientX - dragStartX) / Math.max(bboxW, 1);
    const dy = (e.clientY - dragStartY) / Math.max(bboxH, 1);
    const c = { ...dragStartCrop };
    if (dragKind === 'move') {
      c.x += dx;
      c.y += dy;
    } else {
      if (dragKind.includes('w')) {
        const nx = dragStartCrop.x + dx;
        c.x = nx;
        c.w = dragStartCrop.w - dx;
      }
      if (dragKind.includes('e')) {
        c.w = dragStartCrop.w + dx;
      }
      if (dragKind.includes('n')) {
        const ny = dragStartCrop.y + dy;
        c.y = ny;
        c.h = dragStartCrop.h - dy;
      }
      if (dragKind.includes('s')) {
        c.h = dragStartCrop.h + dy;
      }
      if (c.w < 0.05) c.w = 0.05;
      if (c.h < 0.05) c.h = 0.05;
      const ratio = aspectRatioFor(sess.draftAspect, sourceW, sourceH);
      if (ratio !== null && bboxW > 0 && bboxH > 0) {
        const wPx = c.w * bboxW;
        const hPx = c.h * bboxH;
        const isCorner = dragKind.length === 2;
        const isHorzEdge = dragKind === 'n' || dragKind === 's';
        const isVertEdge = dragKind === 'e' || dragKind === 'w';
        let newW = c.w;
        let newH = c.h;
        if (isHorzEdge) {
          newW = (hPx * ratio) / bboxW;
        } else if (isVertEdge) {
          newH = wPx / ratio / bboxH;
        } else if (isCorner) {
          if (wPx / hPx > ratio) {
            newH = wPx / ratio / bboxH;
          } else {
            newW = (hPx * ratio) / bboxW;
          }
        }
        if (dragKind.includes('w')) c.x = dragStartCrop.x + dragStartCrop.w - newW;
        if (dragKind.includes('n')) c.y = dragStartCrop.y + dragStartCrop.h - newH;
        if (dragKind === 'n' || dragKind === 's') {
          c.x = dragStartCrop.x + (dragStartCrop.w - newW) / 2;
        }
        if (dragKind === 'e' || dragKind === 'w') {
          c.y = dragStartCrop.y + (dragStartCrop.h - newH) / 2;
        }
        c.w = newW;
        c.h = newH;
      }
    }
    editor.updateGeometryDraftCrop(c);
  }

  function onUp(e: PointerEvent): void {
    dragKind = null;
    dragStartCrop = null;
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
    <div class="relative" style="width: {bboxW}px; height: {bboxH}px;">
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
        class="absolute border border-white/90 cursor-move"
        style="left: {cropPx.x}px; top: {cropPx.y}px; width: {cropPx.w}px; height: {cropPx.h}px;"
        onpointerdown={(e) => startDrag(e, 'move')}
        onpointermove={onMove}
        onpointerup={onUp}
        onpointercancel={onUp}
        role="presentation"
      >
        <div class="absolute inset-0 pointer-events-none">
          <div class="absolute top-1/3 left-0 right-0 border-t border-white/30"></div>
          <div class="absolute top-2/3 left-0 right-0 border-t border-white/30"></div>
          <div class="absolute left-1/3 top-0 bottom-0 border-l border-white/30"></div>
          <div class="absolute left-2/3 top-0 bottom-0 border-l border-white/30"></div>
        </div>
        {#each ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'] as const as h (h)}
          <button
            class="absolute bg-white border border-black/60 rounded-sm"
            style="
              width: 12px; height: 12px;
              {h.includes('n') ? 'top: -6px;' : ''}
              {h.includes('s') ? 'bottom: -6px;' : ''}
              {h.includes('w') ? 'left: -6px;' : ''}
              {h.includes('e') ? 'right: -6px;' : ''}
              {h === 'n' || h === 's' ? 'left: calc(50% - 6px);' : ''}
              {h === 'w' || h === 'e' ? 'top: calc(50% - 6px);' : ''}
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
