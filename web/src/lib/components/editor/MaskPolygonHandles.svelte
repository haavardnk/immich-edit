<script lang="ts">
  import type { MaskComponentKind, Vec2f } from '$lib/types/edits';
  import { MAX_POLYGON_POINTS } from '$lib/types/masks';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { DragKind } from '$lib/utils/maskDrag';

  let {
    kind,
    color,
    toPx,
    fromPx,
    onDrag,
    onPoints
  }: {
    kind: Extract<MaskComponentKind, { kind: 'polygon' }>;
    color: string;
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
    return toPx({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });
  }

  function startMove(e: PointerEvent): void {
    const svg = (e.currentTarget as SVGElement).ownerSVGElement;
    if (!svg) return;
    const r = svg.getBoundingClientRect();
    onDrag(e, {
      kind: 'polygon-move',
      start: kind.points.map((p) => ({ ...p })),
      downAtN: fromPx(e.clientX - r.left, e.clientY - r.top)
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
</script>

<g style="pointer-events: auto;">
  <polygon
    points={path}
    fill={color}
    fill-opacity="0.12"
    stroke={color}
    stroke-width="1.5"
    style="cursor: move;"
    role="button"
    aria-label="Move polygon"
    tabindex="-1"
    onpointerdown={startMove}
  />
  {#each kind.points as _p, i (i)}
    {@const mid = midpoint(i)}
    <circle
      cx={mid.x}
      cy={mid.y}
      r="5"
      fill="white"
      fill-opacity="0.6"
      stroke={color}
      stroke-width="1.5"
      style="cursor: copy;"
      role="button"
      aria-label="Add polygon corner"
      tabindex="-1"
      onpointerdown={(e) => insertVertex(e, i)}
    />
  {/each}
  {#each points as p, i (i)}
    <circle
      cx={p.x}
      cy={p.y}
      r="7"
      fill={color}
      stroke="white"
      stroke-width="2"
      style="cursor: move;"
      role="button"
      aria-label="Polygon corner"
      tabindex="-1"
      onpointerdown={(e) => onDrag(e, { kind: 'polygon-vertex', index: i })}
      ondblclick={(e) => deleteVertex(e, i)}
    />
  {/each}
</g>
