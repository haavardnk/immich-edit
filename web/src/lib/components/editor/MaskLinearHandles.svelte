<script lang="ts">
  import type { MaskComponent, MaskComponentKind, Vec2f } from '#lib/types/edits.ts';
  import type { DragKind } from '#lib/utils/maskDrag.ts';
  import { smoothRamp } from '#lib/utils/maskRamp.ts';
  import MaskHandle from './MaskHandle.svelte';
  import { elementPoint } from '#lib/gestures/point.ts';
  import { HANDLE_DOT, spacedHandles } from '#lib/editor/handles.ts';
  import { handleHit } from '#lib/editor/handleHit.svelte.ts';
  import type { MaskHandleSpec } from '#lib/types/maskHandle.ts';

  let {
    comp,
    kind,
    rect,
    toPx,
    fromPx,
    onSelect,
    onDrag
  }: {
    comp: MaskComponent;
    kind: Extract<MaskComponentKind, { kind: 'linear' }>;
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
  const ramp = $derived(smoothRamp(stopLo, stopHi, emptyOp, fillOp));

  const GUIDE_LEN = 60;

  function startMove(e: PointerEvent): void {
    const svg = (e.currentTarget as SVGElement).ownerSVGElement;
    if (!svg) return;
    const at = elementPoint(e, svg);
    onDrag(e, {
      kind: 'linear-move',
      startP0: { ...kind.p0 },
      startP1: { ...kind.p1 },
      downAtN: fromPx(at.x, at.y)
    });
  }

  function end(at: { x: number; y: number }, label: string, drag: DragKind): MaskHandleSpec {
    return {
      cx: at.x,
      cy: at.y,
      r: HANDLE_DOT.maskEnd,
      fill: 'var(--color-mask-overlay)',
      stroke: 'var(--color-image-light)',
      strokeWidth: 2,
      cursor: 'move',
      label,
      onpointerdown: (e) => onDrag(e, drag)
    };
  }

  function feather(at: { x: number; y: number }): MaskHandleSpec {
    return {
      cx: at.x,
      cy: at.y,
      r: HANDLE_DOT.maskFeather,
      fill: 'var(--color-mask-overlay)',
      fillOpacity: 0.3,
      stroke: 'var(--color-mask-overlay)',
      strokeWidth: 1.5,
      cursor: 'move',
      label: 'Linear feather',
      onpointerdown: (e) => onDrag(e, { kind: 'linear-feather' })
    };
  }

  const handles = $derived(
    spacedHandles<MaskHandleSpec>(handleHit(), [
      end(a, 'Linear start', { kind: 'linear-p0' }),
      end(b, 'Linear end', { kind: 'linear-p1' }),
      feather(hi),
      feather(lo),
      {
        cx: mx,
        cy: my,
        r: HANDLE_DOT.maskCentre,
        fill: 'var(--color-image-light)',
        stroke: 'var(--color-mask-overlay)',
        strokeWidth: 2,
        cursor: 'move',
        label: 'Move linear',
        onpointerdown: startMove
      }
    ])
  );
</script>

<g style="pointer-events: auto;">
  <defs>
    <linearGradient id={gradId} gradientUnits="userSpaceOnUse" x1={a.x} y1={a.y} x2={b.x} y2={b.y}>
      <stop offset="0" stop-color="var(--color-mask-overlay)" stop-opacity={emptyOp} />
      {#each ramp as stop, i (i)}
        <stop
          offset={stop.offset}
          stop-color="var(--color-mask-overlay)"
          stop-opacity={stop.opacity}
        />
      {/each}
      <stop offset="1" stop-color="var(--color-mask-overlay)" stop-opacity={fillOp} />
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
    stroke="var(--color-mask-overlay)"
    stroke-width="1.5"
    opacity="0.9"
    style="cursor: pointer;"
    role="button"
    aria-label="Select linear"
    tabindex="-1"
    onpointerdown={onSelect}
  />
  <line
    x1={a.x}
    y1={a.y}
    x2={b.x}
    y2={b.y}
    stroke="var(--color-image-dark)"
    stroke-width="0.5"
    opacity="0.5"
  />
  <line
    x1={lo.x - px * GUIDE_LEN}
    y1={lo.y - py * GUIDE_LEN}
    x2={lo.x + px * GUIDE_LEN}
    y2={lo.y + py * GUIDE_LEN}
    stroke="var(--color-mask-overlay)"
    stroke-width="1"
    stroke-dasharray="4 4"
    opacity="0.7"
  />
  <line
    x1={hi.x - px * GUIDE_LEN}
    y1={hi.y - py * GUIDE_LEN}
    x2={hi.x + px * GUIDE_LEN}
    y2={hi.y + py * GUIDE_LEN}
    stroke="var(--color-mask-overlay)"
    stroke-width="1"
    stroke-dasharray="4 4"
    opacity="0.7"
  />
  {#each handles as handle, i (i)}
    <MaskHandle {...handle} />
  {/each}
</g>
