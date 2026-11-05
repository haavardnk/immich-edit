<script lang="ts">
  import type { MaskComponentKind, Vec2f } from '#lib/types/edits.ts';
  import { MAX_POLYGON_POINTS } from '#lib/types/masks.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { DragKind } from '#lib/utils/maskDrag.ts';
  import MaskHandle from './MaskHandle.svelte';
  import { elementPoint } from '#lib/gestures/point.ts';
  import { HANDLE_DOT, spacedHandles } from '#lib/editor/handles.ts';
  import { handleHit } from '#lib/editor/handleHit.svelte.ts';
  import type { MaskHandleSpec } from '#lib/types/maskHandle.ts';

  let {
    kind,
    toPx,
    fromPx,
    onDrag,
    onPoints
  }: {
    kind: Extract<MaskComponentKind, { kind: 'polygon' }>;
    toPx: (v: Vec2f) => { x: number; y: number };
    fromPx: (px: number, py: number) => Vec2f;
    onDrag: (e: PointerEvent, kind: DragKind) => void;
    onPoints: (points: Vec2f[]) => void;
  } = $props();

  const points = $derived(kind.points.map((p) => toPx(p)));
  const path = $derived(points.map((p) => `${p.x},${p.y}`).join(' '));

  function midpoint(i: number): { x: number; y: number } {
    const a = kind.points[i];
    const b = kind.points[(i + 1) % kind.points.length];
    if (!a || !b) return toPx({ x: 0, y: 0 });
    return toPx({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });
  }

  function startMove(e: PointerEvent): void {
    const svg = (e.currentTarget as SVGElement).ownerSVGElement;
    if (!svg) return;
    const at = elementPoint(e, svg);
    onDrag(e, {
      kind: 'polygon-move',
      start: kind.points.map((p) => ({ ...p })),
      downAtN: fromPx(at.x, at.y)
    });
  }

  function insertVertex(e: PointerEvent, after: number): void {
    e.preventDefault();
    e.stopPropagation();
    if (kind.points.length >= MAX_POLYGON_POINTS) {
      toasts.push('info', `A polygon can have at most ${MAX_POLYGON_POINTS} corners.`);
      return;
    }
    const a = kind.points[after];
    const b = kind.points[(after + 1) % kind.points.length];
    if (!a || !b) return;
    const next = [...kind.points];
    next.splice(after + 1, 0, { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });
    onPoints(next);
  }

  function deleteVertex(e: MouseEvent, index: number): void {
    e.preventDefault();
    e.stopPropagation();
    if (kind.points.length <= 3) return;
    onPoints(kind.points.filter((_, i) => i !== index));
  }

  const handles = $derived(
    spacedHandles<MaskHandleSpec>(handleHit(), [
      ...kind.points.map((_, i) => {
        const mid = midpoint(i);
        return {
          cx: mid.x,
          cy: mid.y,
          r: HANDLE_DOT.polygonMid,
          hit: HANDLE_DOT.polygonMid,
          fill: 'var(--color-image-light)',
          fillOpacity: 0.6,
          stroke: 'var(--color-mask-overlay)',
          strokeWidth: 1.5,
          cursor: 'copy',
          label: 'Add polygon corner',
          onpointerdown: (e: PointerEvent) => insertVertex(e, i)
        };
      }),
      ...points.map((p, i) => ({
        cx: p.x,
        cy: p.y,
        r: HANDLE_DOT.polygonCorner,
        fill: 'var(--color-mask-overlay)',
        stroke: 'var(--color-image-light)',
        strokeWidth: 2,
        cursor: 'move',
        label: 'Polygon corner',
        onpointerdown: (e: PointerEvent) =>
          onDrag(e, {
            kind: 'polygon-vertex',
            index: i,
            origin: kind.points[i] ?? { x: 0, y: 0 }
          }),
        ondblclick: (e: MouseEvent) => deleteVertex(e, i)
      }))
    ])
  );
</script>

<g style="pointer-events: auto;">
  <polygon
    points={path}
    fill="var(--color-mask-overlay)"
    fill-opacity="0.12"
    stroke="var(--color-mask-overlay)"
    stroke-width="1.5"
    style="cursor: move;"
    role="button"
    aria-label="Move polygon"
    tabindex="-1"
    onpointerdown={startMove}
  />
  {#each handles as handle, i (i)}
    <MaskHandle {...handle} />
  {/each}
</g>
