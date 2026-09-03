<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { isKeybind, keyLabel } from '$lib/keybinds';
  import type { Vec2f } from '$lib/types/edits';
  import { MAX_POLYGON_POINTS } from '$lib/types/masks';

  let {
    draft,
    rect,
    toPx,
    fromPx
  }: {
    draft: { points: Vec2f[] };
    rect: { x: number; y: number };
    toPx: (v: Vec2f) => { x: number; y: number };
    fromPx: (px: number, py: number) => Vec2f;
  } = $props();

  let cursor = $state<{ x: number; y: number } | null>(null);

  const points = $derived(draft.points.map((p) => toPx(p)));

  function pointerMove(e: PointerEvent): void {
    const r = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    cursor = { x: e.clientX - r.left, y: e.clientY - r.top };
  }

  function place(e: PointerEvent): void {
    e.preventDefault();
    e.stopPropagation();
    const r = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    const p = fromPx(e.clientX - r.left, e.clientY - r.top);
    if (p.x < 0 || p.y < 0 || p.x > 1 || p.y > 1) return;
    if (draft.points.length >= MAX_POLYGON_POINTS) {
      toasts.push('info', `A polygon can have at most ${MAX_POLYGON_POINTS} corners.`);
      return;
    }
    editor.addPolygonPoint(p);
  }

  function close(e: PointerEvent, index: number): void {
    if (index !== 0 || draft.points.length < 3) return;
    e.preventDefault();
    e.stopPropagation();
    void editor.finishPolygon();
    cursor = null;
  }

  function onKey(e: KeyboardEvent): void {
    if (isKeybind(e, 'maskCancelDraw')) {
      e.preventDefault();
      editor.cancelPolygon();
      cursor = null;
    } else if (isKeybind(e, 'maskClosePolygon')) {
      e.preventDefault();
      void editor.finishPolygon();
      cursor = null;
    } else if (isKeybind(e, 'maskDelete')) {
      e.preventDefault();
      editor.undoPolygonPoint();
    }
  }

  $effect(() => {
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

<svg
  class="absolute inset-0"
  width="100%"
  height="100%"
  role="presentation"
  style="cursor: crosshair;"
  onpointermove={pointerMove}
  onpointerdown={place}
  oncontextmenu={(e) => {
    e.preventDefault();
    editor.undoPolygonPoint();
  }}
>
  {#if points.length > 0}
    {@const last = points[points.length - 1]}
    <polyline
      points={points.map((p) => `${p.x},${p.y}`).join(' ')}
      fill="none"
      stroke="var(--color-image-light)"
      stroke-width="1.5"
      stroke-dasharray="4 3"
      style="pointer-events: none;"
    />
    {#if cursor && last}
      <line
        x1={last.x}
        y1={last.y}
        x2={cursor.x}
        y2={cursor.y}
        stroke="var(--color-image-light)"
        stroke-width="1.5"
        stroke-dasharray="4 3"
        opacity="0.7"
        style="pointer-events: none;"
      />
    {/if}
    {#each points as p, i (i)}
      {@const closable = i === 0 && draft.points.length >= 3}
      <circle
        cx={p.x}
        cy={p.y}
        r={i === 0 ? 8 : 5}
        fill={i === 0 ? 'var(--color-image-light)' : 'var(--color-image-dark)'}
        stroke="var(--color-image-light)"
        stroke-width="2"
        style={closable ? 'cursor: pointer;' : 'pointer-events: none;'}
        role={closable ? 'button' : undefined}
        aria-label={closable ? 'Close polygon' : undefined}
        onpointerdown={(e) => close(e, i)}
      />
    {/each}
  {/if}
  <text
    x={rect.x + 12}
    y={rect.y + 22}
    fill="var(--color-image-light)"
    font-size="12"
    style="pointer-events: none; paint-order: stroke; stroke: var(--color-control-shadow); stroke-width: 3px;"
  >
    {draft.points.length < 3
      ? `Click to place corners (${draft.points.length}/${MAX_POLYGON_POINTS})`
      : `Click the first corner or press ${keyLabel('Enter')} to close. ${keyLabel('Escape')} cancels.`}
  </text>
</svg>
