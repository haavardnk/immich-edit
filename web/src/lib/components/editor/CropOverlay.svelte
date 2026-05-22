<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { rotatedBbox, aspectRatioFor } from '$lib/utils/geom';
  import type { CropRect } from '$lib/types/edits';

  let container = $state<HTMLDivElement | null>(null);
  let containerW = $state(0);
  let containerH = $state(0);

  $effect(() => {
    if (!container) return;
    const el = container;
    const ro = new ResizeObserver(() => {
      const rect = el.getBoundingClientRect();
      containerW = rect.width;
      containerH = rect.height;
    });
    ro.observe(el);
    const rect = el.getBoundingClientRect();
    containerW = rect.width;
    containerH = rect.height;
    return () => ro.disconnect();
  });

  const sess = $derived(editor.cropSession);
  const bbox = $derived(
    sess ? rotatedBbox(sess.sourceW, sess.sourceH, sess.draftAngle) : { w: 1, h: 1 }
  );
  const scale = $derived(
    Math.min(
      (containerW * 0.92) / Math.max(bbox.w, 1),
      (containerH * 0.92) / Math.max(bbox.h, 1)
    )
  );
  const bboxW = $derived(bbox.w * scale);
  const bboxH = $derived(bbox.h * scale);
  const srcW = $derived((sess?.sourceW ?? 1) * scale);
  const srcH = $derived((sess?.sourceH ?? 1) * scale);
  const crop = $derived(sess?.draftCrop ?? { x: 0, y: 0, w: 1, h: 1 });
  const cropPx = $derived({
    x: crop.x * bboxW,
    y: crop.y * bboxH,
    w: crop.w * bboxW,
    h: crop.h * bboxH
  });

  type DragKind =
    | 'move'
    | 'nw'
    | 'n'
    | 'ne'
    | 'e'
    | 'se'
    | 's'
    | 'sw'
    | 'w';

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
      const ratio = aspectRatioFor(sess.draftAspect, sess.sourceW, sess.sourceH);
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
          newH = (wPx / ratio) / bboxH;
        } else if (isCorner) {
          if (wPx / hPx > ratio) {
            newH = (wPx / ratio) / bboxH;
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
    editor.updateCropDraftCrop(c);
  }

  function onUp(e: PointerEvent): void {
    dragKind = null;
    dragStartCrop = null;
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
  }
</script>

<div
  bind:this={container}
  class="absolute inset-0 flex items-center justify-center select-none"
>
  {#if sess}
    <div
      class="relative"
      style="width: {bboxW}px; height: {bboxH}px;"
    >
      <div
        class="absolute"
        style="left: {(bboxW - srcW) / 2}px; top: {(bboxH - srcH) / 2}px; width: {srcW}px; height: {srcH}px; transform: rotate({sess.draftAngle}deg); transform-origin: center;"
      >
        <img
          src={sess.pinnedUrl}
          alt=""
          draggable="false"
          class="w-full h-full block"
          style="image-orientation: none;"
        />
      </div>

      <div
        class="absolute inset-0 pointer-events-none"
        style="clip-path: polygon(
          0 0, 100% 0, 100% 100%, 0 100%, 0 0,
          {(cropPx.x / bboxW) * 100}% {(cropPx.y / bboxH) * 100}%,
          {(cropPx.x / bboxW) * 100}% {((cropPx.y + cropPx.h) / bboxH) * 100}%,
          {((cropPx.x + cropPx.w) / bboxW) * 100}% {((cropPx.y + cropPx.h) / bboxH) * 100}%,
          {((cropPx.x + cropPx.w) / bboxW) * 100}% {(cropPx.y / bboxH) * 100}%,
          {(cropPx.x / bboxW) * 100}% {(cropPx.y / bboxH) * 100}%
        ); background: rgba(0,0,0,0.55);"
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
        {#each ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'] as const as h}
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
              cursor: {h === 'n' || h === 's' ? 'ns-resize' : h === 'e' || h === 'w' ? 'ew-resize' : h === 'nw' || h === 'se' ? 'nwse-resize' : 'nesw-resize'};
            "
            onpointerdown={(e) => startDrag(e, h)}
            onpointermove={onMove}
            onpointerup={onUp}
            onpointercancel={onUp}
            aria-label="resize {h}"
          ></button>
        {/each}
      </div>
    </div>
  {/if}
</div>
