<script lang="ts">
  import type { MaskComponent, MaskComponentKind, Vec2f } from '$lib/types/edits';
  import type { DragKind } from '$lib/utils/maskDrag';

  let {
    comp,
    kind,
    color,
    rect,
    toPx,
    onSelect,
    onDrag
  }: {
    comp: MaskComponent;
    kind: Extract<MaskComponentKind, { kind: 'radial' }>;
    color: string;
    rect: { x: number; y: number; w: number; h: number };
    toPx: (v: Vec2f) => { x: number; y: number };
    onSelect: (e: PointerEvent) => void;
    onDrag: (e: PointerEvent, kind: DragKind) => void;
  } = $props();

  const c = $derived(toPx(kind.center));
  const rxEnd = $derived(toPx({ x: kind.center.x + kind.radius_xy.x, y: kind.center.y }));
  const ryEnd = $derived(toPx({ x: kind.center.x, y: kind.center.y + kind.radius_xy.y }));
  const rxDx = $derived(rxEnd.x - c.x);
  const rxDy = $derived(rxEnd.y - c.y);
  const ryDx = $derived(ryEnd.x - c.x);
  const ryDy = $derived(ryEnd.y - c.y);
  const rx = $derived(Math.hypot(rxDx, rxDy));
  const ry = $derived(Math.hypot(ryDx, ryDy));
  const tilt = $derived((Math.atan2(rxDy, rxDx) * 180) / Math.PI);
  const innerScale = $derived(1 - kind.feather);
  const gradId = $derived(`mask-radial-${comp.id}`);
  const fillOp = $derived(comp.invert ? 0.0 : 0.55);
  const emptyOp = $derived(comp.invert ? 0.55 : 0.0);
  const rMax = $derived(Math.max(rx, ry, 1));
</script>

<g style="pointer-events: auto;">
  <defs>
    <radialGradient
      id={gradId}
      gradientUnits="userSpaceOnUse"
      cx={c.x}
      cy={c.y}
      r={rMax}
      gradientTransform={`translate(${c.x} ${c.y}) rotate(${tilt}) scale(${rx / rMax} ${ry / rMax}) translate(${-c.x} ${-c.y})`}
    >
      <stop offset="0" stop-color={color} stop-opacity={fillOp} />
      <stop offset={innerScale} stop-color={color} stop-opacity={fillOp} />
      <stop offset="1" stop-color={color} stop-opacity={emptyOp} />
    </radialGradient>
  </defs>
  <rect
    x={rect.x}
    y={rect.y}
    width={rect.w}
    height={rect.h}
    fill={`url(#${gradId})`}
    style="pointer-events: none;"
  />
  <ellipse
    cx={c.x}
    cy={c.y}
    {rx}
    {ry}
    transform={`rotate(${tilt} ${c.x} ${c.y})`}
    fill="none"
    stroke={color}
    stroke-width="1.5"
    opacity="0.9"
    style="cursor: pointer;"
    role="button"
    aria-label="Select radial"
    tabindex="-1"
    onpointerdown={onSelect}
  />
  <ellipse
    cx={c.x}
    cy={c.y}
    {rx}
    {ry}
    transform={`rotate(${tilt} ${c.x} ${c.y})`}
    fill="none"
    stroke="black"
    stroke-width="0.5"
    opacity="0.5"
  />
  <circle
    cx={c.x}
    cy={c.y}
    r="6"
    fill="white"
    stroke={color}
    stroke-width="2"
    style="cursor: move;"
    role="button"
    aria-label="Radial center"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'radial-center' })}
  />
  <circle
    cx={c.x + rxDx}
    cy={c.y + rxDy}
    r="6"
    fill={color}
    stroke="white"
    stroke-width="2"
    style="cursor: ew-resize;"
    role="button"
    aria-label="Radial radius x"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'radial-rx', sign: 1 })}
  />
  <circle
    cx={c.x - rxDx}
    cy={c.y - rxDy}
    r="6"
    fill={color}
    stroke="white"
    stroke-width="2"
    style="cursor: ew-resize;"
    role="button"
    aria-label="Radial radius x"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'radial-rx', sign: -1 })}
  />
  <circle
    cx={c.x + ryDx}
    cy={c.y + ryDy}
    r="6"
    fill={color}
    stroke="white"
    stroke-width="2"
    style="cursor: ns-resize;"
    role="button"
    aria-label="Radial radius y"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'radial-ry', sign: 1 })}
  />
  <circle
    cx={c.x - ryDx}
    cy={c.y - ryDy}
    r="6"
    fill={color}
    stroke="white"
    stroke-width="2"
    style="cursor: ns-resize;"
    role="button"
    aria-label="Radial radius y"
    tabindex="-1"
    onpointerdown={(e) => onDrag(e, { kind: 'radial-ry', sign: -1 })}
  />
  {#if innerScale > 0.001}
    <ellipse
      cx={c.x}
      cy={c.y}
      rx={rx * innerScale}
      ry={ry * innerScale}
      transform={`rotate(${tilt} ${c.x} ${c.y})`}
      fill="none"
      stroke={color}
      stroke-width="1"
      stroke-dasharray="4 4"
      opacity="0.7"
    />
    <circle
      cx={c.x + rxDx * innerScale}
      cy={c.y + rxDy * innerScale}
      r="5"
      fill={color}
      fill-opacity="0.3"
      stroke={color}
      stroke-width="1.5"
      style="cursor: move;"
      role="button"
      aria-label="Radial feather"
      tabindex="-1"
      onpointerdown={(e) => onDrag(e, { kind: 'radial-feather' })}
    />
  {/if}
</g>
