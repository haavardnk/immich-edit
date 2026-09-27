<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import {
    CURVE_CHANNELS,
    curvesAreIdentity,
    type CurveChannel,
    type CurvePoint
  } from '$lib/types/edits';
  import {
    CHANNEL_LABELS,
    CURVE_INNER,
    CURVE_PAD,
    CURVE_SIZE,
    GRID_LINES,
    curvePath,
    dragPoint,
    fromSvg,
    histLayers,
    hitTest,
    insertPoint,
    nudgePoint,
    removePoint,
    toSvg
  } from './curves';

  let { channel }: { channel: CurveChannel } = $props();

  let dragging: number | null = $state(null);
  let selected: number | null = $state(null);
  let pointerId: number | null = null;
  let keyboardDirty = $state(false);

  const channelStroke: Record<CurveChannel, string> = {
    composite: 'var(--color-image-light)',
    r: 'var(--color-curve-red)',
    g: 'var(--color-curve-green)',
    b: 'var(--color-curve-blue)',
    luma: 'var(--color-curve-luma)'
  };

  const label = $derived(CHANNEL_LABELS[channel]);
  const activeCurve = $derived(getCurve(channel));
  const activePath = $derived(curvePath(activeCurve));
  const layers = $derived(histLayers(editor.meta?.histogram ?? null, channel));
  const overlayPaths = $derived(
    CURVE_CHANNELS.filter((ch) => ch !== channel && !curvesAreIdentity(getCurve(ch))).map((ch) => ({
      ch,
      d: curvePath(getCurve(ch))
    }))
  );

  export function clearSelection(): void {
    dragging = null;
    selected = null;
  }

  function getCurve(ch: CurveChannel): CurvePoint[] {
    return editor.edits.basic.curves[ch];
  }

  function setCurve(pts: CurvePoint[]): void {
    editor.edits.basic.curves = { ...editor.edits.basic.curves, [channel]: pts };
  }

  function commit(): void {
    void editor.onCommit(`Curve ${label}`);
  }

  function svgCoords(e: MouseEvent): { sx: number; sy: number } {
    const svg = (e.currentTarget as SVGElement).closest('svg') as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const scale = CURVE_SIZE / rect.width;
    return { sx: (e.clientX - rect.left) * scale, sy: (e.clientY - rect.top) * scale };
  }

  function onPointerDown(e: PointerEvent): void {
    if (e.button !== undefined && e.button !== 0) return;
    const { sx, sy } = svgCoords(e);
    const pts = getCurve(channel);
    const hit = hitTest(pts, sx, sy);
    if (hit >= 0) {
      dragging = hit;
      selected = hit;
    } else {
      const { points, index } = insertPoint(pts, fromSvg(sx, sy));
      setCurve(points);
      dragging = index;
      selected = index;
      editor.onLive();
    }
    pointerId = e.pointerId;
    (e.currentTarget as SVGElement).setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function onPointerMove(e: PointerEvent): void {
    if (dragging === null) return;
    const { sx, sy } = svgCoords(e);
    const next = dragPoint(getCurve(channel), dragging, fromSvg(sx, sy));
    if (!next) return;
    setCurve(next);
    editor.onLive();
  }

  function endDrag(e: PointerEvent): void {
    if (dragging === null) return;
    dragging = null;
    if (pointerId !== null) {
      (e.currentTarget as SVGElement).releasePointerCapture(pointerId);
      pointerId = null;
    }
    commit();
  }

  function onDblClick(e: MouseEvent): void {
    const { sx, sy } = svgCoords(e);
    const pts = getCurve(channel);
    const hit = hitTest(pts, sx, sy, true);
    const next = removePoint(pts, hit);
    if (!next) return;
    setCurve(next);
    if (selected === hit) selected = null;
    commit();
  }

  function nudge(idx: number, dx: number, dy: number): void {
    const next = nudgePoint(getCurve(channel), idx, dx, dy);
    if (next) setCurve(next);
    editor.onLive();
    keyboardDirty = true;
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (selected === null) return;
    const step = e.shiftKey ? 0.05 : 0.01;
    if (e.key === 'ArrowUp') {
      nudge(selected, 0, step);
    } else if (e.key === 'ArrowDown') {
      nudge(selected, 0, -step);
    } else if (e.key === 'ArrowLeft') {
      nudge(selected, -step, 0);
    } else if (e.key === 'ArrowRight') {
      nudge(selected, step, 0);
    } else if (e.key === 'Delete' || e.key === 'Backspace') {
      const next = removePoint(getCurve(channel), selected);
      if (next) {
        setCurve(next);
        selected = null;
        commit();
      }
    } else if (e.key === 'Escape') {
      selected = null;
    } else {
      return;
    }
    e.preventDefault();
  }

  function commitKeyboard(): void {
    if (!keyboardDirty) return;
    keyboardDirty = false;
    commit();
  }

  function onKeyUp(e: KeyboardEvent): void {
    if (!e.key.startsWith('Arrow')) return;
    commitKeyboard();
  }
</script>

<div class="flex flex-col items-center">
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <svg
    viewBox="0 0 {CURVE_SIZE} {CURVE_SIZE}"
    class="aspect-square w-full cursor-crosshair select-none rounded-sm bg-neutral-950 ring-1 ring-hairline focus:outline-none focus-visible:ring-2 focus-visible:ring-primary"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={endDrag}
    onpointercancel={endDrag}
    ondblclick={onDblClick}
    onkeydown={onKeyDown}
    onkeyup={onKeyUp}
    onblur={commitKeyboard}
    tabindex="0"
    role="application"
    aria-label="Curves – {label}"
  >
    <rect
      x={CURVE_PAD}
      y={CURVE_PAD}
      width={CURVE_INNER}
      height={CURVE_INNER}
      fill="var(--color-curve-surface)"
      rx="2"
    />

    {#each layers as layer (layer.fill)}
      {#if layer.d}
        <path d={layer.d} fill={layer.fill} style:mix-blend-mode="screen" />
      {/if}
    {/each}

    {#each GRID_LINES as l, i (i)}
      <line
        x1={l.x1}
        y1={l.y1}
        x2={l.x2}
        y2={l.y2}
        stroke="var(--color-curve-grid)"
        stroke-width="0.5"
      />
    {/each}

    <line
      x1={CURVE_PAD}
      y1={CURVE_PAD + CURVE_INNER}
      x2={CURVE_PAD + CURVE_INNER}
      y2={CURVE_PAD}
      stroke="var(--color-curve-diagonal)"
      stroke-width="1"
      stroke-dasharray="3,3"
    />

    {#each overlayPaths as overlay (overlay.ch)}
      <path
        d={overlay.d}
        fill="none"
        stroke={channelStroke[overlay.ch]}
        stroke-width="1.25"
        stroke-linecap="round"
        opacity="0.25"
      />
    {/each}

    <path
      d={activePath}
      fill="none"
      stroke={channelStroke[channel]}
      stroke-width="2"
      stroke-linecap="round"
    />

    {#each activeCurve as pt, i (i)}
      {@const sp = toSvg(pt)}
      <circle
        cx={sp.x}
        cy={sp.y}
        r={dragging === i || selected === i ? 7 : 5}
        fill={dragging === i
          ? channelStroke[channel]
          : selected === i
            ? 'color-mix(in srgb, var(--color-image-light) 90%, transparent)'
            : 'color-mix(in srgb, var(--color-image-dark) 90%, transparent)'}
        stroke={channelStroke[channel]}
        stroke-width="2"
      />
    {/each}
  </svg>
</div>
