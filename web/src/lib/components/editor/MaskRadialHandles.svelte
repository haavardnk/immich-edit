<script lang="ts">
  import type { MaskComponent, MaskComponentKind, Vec2f } from '#lib/types/edits.ts';
  import { radialAxes, type DragKind } from '#lib/utils/maskDrag.ts';
  import { smoothRamp } from '#lib/utils/maskRamp.ts';
  import MaskHandle from './MaskHandle.svelte';
  import { HANDLE_DOT, spacedHandles } from '#lib/editor/handles.ts';
  import { handleHit } from '#lib/editor/handleHit.svelte.ts';
  import type { MaskHandleSpec } from '#lib/types/maskHandle.ts';

  const MIN_FEATHER_HANDLE_SCALE = 0.08;
  const ROTATE_GRIP_GAP = 22;

  let {
    comp,
    kind,
    rect,
    toPx,
    aspect,
    onSelect,
    onDrag
  }: {
    comp: MaskComponent;
    kind: Extract<MaskComponentKind, { kind: 'radial' }>;
    rect: { x: number; y: number; w: number; h: number };
    toPx: (v: Vec2f) => { x: number; y: number };
    aspect: number;
    onSelect: (e: PointerEvent) => void;
    onDrag: (e: PointerEvent, kind: DragKind) => void;
  } = $props();

  const c = $derived(toPx(kind.center));
  const axes = $derived(radialAxes(kind, aspect));
  const rxEnd = $derived(toPx({ x: kind.center.x + axes.x.x, y: kind.center.y + axes.x.y }));
  const ryEnd = $derived(toPx({ x: kind.center.x + axes.y.x, y: kind.center.y + axes.y.y }));
  const rxDx = $derived(rxEnd.x - c.x);
  const rxDy = $derived(rxEnd.y - c.y);
  const ryDx = $derived(ryEnd.x - c.x);
  const ryDy = $derived(ryEnd.y - c.y);
  const rx = $derived(Math.hypot(rxDx, rxDy));
  const ry = $derived(Math.hypot(ryDx, ryDy));
  const tilt = $derived((Math.atan2(rxDy, rxDx) * 180) / Math.PI);
  const innerScale = $derived(1 - kind.feather);
  const featherHandleScale = $derived(Math.max(innerScale, MIN_FEATHER_HANDLE_SCALE));
  const gradId = $derived(`mask-radial-${comp.id}`);
  const fillOp = $derived(comp.invert ? 0.0 : 0.55);
  const emptyOp = $derived(comp.invert ? 0.55 : 0.0);
  const rMax = $derived(Math.max(rx, ry, 1));
  const ramp = $derived(smoothRamp(innerScale, 1, fillOp, emptyOp));
  const grip = $derived({
    x: c.x + (rxDx / Math.max(rx, 1)) * (rx + ROTATE_GRIP_GAP),
    y: c.y + (rxDy / Math.max(rx, 1)) * (rx + ROTATE_GRIP_GAP)
  });

  function edge(sign: 1 | -1, axis: 'x' | 'y'): MaskHandleSpec {
    const dx = axis === 'x' ? rxDx : ryDx;
    const dy = axis === 'x' ? rxDy : ryDy;
    return {
      cx: c.x + sign * dx,
      cy: c.y + sign * dy,
      r: HANDLE_DOT.maskEdge,
      fill: 'var(--color-mask-overlay)',
      stroke: 'var(--color-image-light)',
      strokeWidth: 2,
      cursor: axis === 'x' ? 'ew-resize' : 'ns-resize',
      label: `Radial radius ${axis}`,
      onpointerdown: (e) =>
        onDrag(e, axis === 'x' ? { kind: 'radial-rx', sign } : { kind: 'radial-ry', sign })
    };
  }

  const handles = $derived(
    spacedHandles<MaskHandleSpec>(handleHit(), [
      edge(1, 'x'),
      edge(-1, 'x'),
      edge(1, 'y'),
      edge(-1, 'y'),
      {
        cx: grip.x,
        cy: grip.y,
        r: HANDLE_DOT.maskGrip,
        fill: 'var(--color-image-light)',
        stroke: 'var(--color-mask-overlay)',
        strokeWidth: 2,
        cursor: 'grab',
        label: 'Radial rotation',
        onpointerdown: (e) => onDrag(e, { kind: 'radial-rotate' })
      },
      {
        cx: c.x + rxDx * featherHandleScale,
        cy: c.y + rxDy * featherHandleScale,
        r: HANDLE_DOT.maskFeather,
        fill: 'var(--color-mask-overlay)',
        fillOpacity: 0.3,
        stroke: 'var(--color-mask-overlay)',
        strokeWidth: 1.5,
        cursor: 'move',
        label: 'Radial feather',
        onpointerdown: (e) => onDrag(e, { kind: 'radial-feather' })
      },
      {
        cx: c.x,
        cy: c.y,
        r: HANDLE_DOT.maskCentre,
        fill: 'var(--color-image-light)',
        stroke: 'var(--color-mask-overlay)',
        strokeWidth: 2,
        cursor: 'move',
        label: 'Radial center',
        onpointerdown: (e) => onDrag(e, { kind: 'radial-center' })
      }
    ])
  );
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
      <stop offset="0" stop-color="var(--color-mask-overlay)" stop-opacity={fillOp} />
      {#each ramp as stop, i (i)}
        <stop
          offset={stop.offset}
          stop-color="var(--color-mask-overlay)"
          stop-opacity={stop.opacity}
        />
      {/each}
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
    stroke="var(--color-mask-overlay)"
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
    stroke="var(--color-image-dark)"
    stroke-width="0.5"
    opacity="0.5"
  />
  {#if innerScale > 0.001}
    <ellipse
      cx={c.x}
      cy={c.y}
      rx={rx * innerScale}
      ry={ry * innerScale}
      transform={`rotate(${tilt} ${c.x} ${c.y})`}
      fill="none"
      stroke="var(--color-mask-overlay)"
      stroke-width="1"
      stroke-dasharray="4 4"
      opacity="0.7"
    />
  {/if}
  <line
    x1={c.x + rxDx}
    y1={c.y + rxDy}
    x2={grip.x}
    y2={grip.y}
    stroke="var(--color-mask-overlay)"
    stroke-width="1"
    opacity="0.7"
    style="pointer-events: none;"
  />
  {#each handles as handle, i (i)}
    <MaskHandle {...handle} />
  {/each}
</g>
