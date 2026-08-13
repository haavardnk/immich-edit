<script lang="ts">
  import type { MaskComponent, MaskComponentKind, Vec2f } from '$lib/types/edits';
  import type { DragKind } from '$lib/utils/maskDrag';

  let {
    comp,
    kind,
    color,
    rect,
    toPx,
    fromPx,
    onSelect,
    onDrag
  }: {
    comp: MaskComponent;
    kind: Extract<MaskComponentKind, { kind: 'linear' }>;
    color: string;
    rect: { x: number; y: number; w: number; h: number };
    toPx: (v: Vec2f) => { x: number; y: number };
    fromPx: (px: number, py: number) => Vec2f;
    onSelect: (e: PointerEvent) => void;
    onDrag: (e: PointerEvent, kind: DragKind) => void;
  } = $props();

  const a = $derived(toPx(kind.p0));
  const b = $derived(toPx(kind.p1));
  const half = $derived(kind.feather * 0.5);
  const dx = $derived(b.x - a.x);
  const dy = $derived(b.y - a.y);
  const len = $derived(Math.max(1, Math.hypot(dx, dy)));
  const px = $derived(-dy / len);
  const py = $derived(dx / len);
  const mx = $derived((a.x + b.x) / 2);
  const my = $derived((a.y + b.y) / 2);
  const lo = $derived({ x: mx - half * dx, y: my - half * dy });
  const hi = $derived({ x: mx + half * dx, y: my + half * dy });
  const gradId = $derived(`mask-linear-${comp.id}`);
  const fillOp = $derived(comp.invert ? 0.0 : 0.55);
  const emptyOp = $derived(comp.invert ? 0.55 : 0.0);
  const stopLo = $derived(Math.max(0, 0.5 - half));
  const stopHi = $derived(Math.min(1, 0.5 + half));

  const GUIDE_LEN = 60;

  function startMove(e: PointerEvent): void {
    const svg = (e.currentTarget as SVGElement).ownerSVGElement;
    if (!svg) return;
    const r = svg.getBoundingClientRect();
    onDrag(e, {
      kind: 'linear-move',
      startP0: { ...kind.p0 },
      startP1: { ...kind.p1 },
      downAtN: fromPx(e.clientX - r.left, e.clientY - r.top)
    });
  }
</script>

<g style="pointer-events: auto;">
  <defs>
    <linearGradient id={gradId} gradientUnits="userSpaceOnUse" x1={a.x} y1={a.y} x2={b.x} y2={b.y}>
      <stop offset="0" stop-color={color} stop-opacity={emptyOp} />
      <stop offset={stopLo} stop-color={color} stop-opacity={emptyOp} />
      <stop offset={stopHi} stop-color={color} stop-opacity={fillOp} />
      <stop offset="1" stop-color={color} stop-opacity={fillOp} />
    </linearGradient>
  </defs>
  <rect
    x={rect.x}
    y={rect.y}
    width={rect.w}
    height={rect.h}
    fill={`url(#${gradId})`}
    style="pointer-events: none;"
  />
  <line
    x1={a.x}
    y1={a.y}
    x2={b.x}
    y2={b.y}
    stroke={color}
    stroke-width="1.5"
    opacity="0.9"
    style="cursor: pointer;"
    role="button"
    aria-label="Select linear"
    tabindex="-1"
    onpointerdown={onSelect}
  />
  <line x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke="black" stroke-width="0.5" opacity="0.5" />
  <circle
    cx={a.x}
    cy={a.y}
    r="8"
    fill={color}
    stroke="white"
    stroke-width="2"
    style="cursor: move;"
    role="button"
    aria-label="Linear start"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'linear-p0' })}
  />
  <circle
    cx={b.x}
    cy={b.y}
    r="8"
    fill={color}
    stroke="white"
    stroke-width="2"
    style="cursor: move;"
    role="button"
    aria-label="Linear end"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'linear-p1' })}
  />
  <circle
    cx={mx}
    cy={my}
    r="6"
    fill="white"
    stroke={color}
    stroke-width="2"
    style="cursor: move;"
    role="button"
    aria-label="Move linear"
    tabindex="-1"
    onpointerdown={startMove}
  />
  <line
    x1={lo.x - px * GUIDE_LEN}
    y1={lo.y - py * GUIDE_LEN}
    x2={lo.x + px * GUIDE_LEN}
    y2={lo.y + py * GUIDE_LEN}
    stroke={color}
    stroke-width="1"
    stroke-dasharray="4 4"
    opacity="0.7"
  />
  <line
    x1={hi.x - px * GUIDE_LEN}
    y1={hi.y - py * GUIDE_LEN}
    x2={hi.x + px * GUIDE_LEN}
    y2={hi.y + py * GUIDE_LEN}
    stroke={color}
    stroke-width="1"
    stroke-dasharray="4 4"
    opacity="0.7"
  />
  <circle
    cx={hi.x}
    cy={hi.y}
    r="5"
    fill={color}
    fill-opacity="0.3"
    stroke={color}
    stroke-width="1.5"
    style="cursor: move;"
    role="button"
    aria-label="Linear feather"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'linear-feather' })}
  />
  <circle
    cx={lo.x}
    cy={lo.y}
    r="5"
    fill={color}
    fill-opacity="0.3"
    stroke={color}
    stroke-width="1.5"
    style="cursor: move;"
    role="button"
    aria-label="Linear feather"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'linear-feather' })}
  />
</g>
