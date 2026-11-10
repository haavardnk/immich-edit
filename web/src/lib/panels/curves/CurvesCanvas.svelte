<script lang="ts">
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { elementPoint } from '#lib/gestures/point.ts';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import {
    CURVE_CHANNELS,
    curvesAreIdentity,
    type CurveChannel,
    type CurvePoint
  } from '#lib/types/edits.ts';
  import { untrack } from 'svelte';
  import {
    MOUSE_HIT,
    TOUCH_HIT,
    curvePath,
    fromSvg,
    gridLines,
    histLayers,
    hitTest,
    levelOf,
    plotOf,
    toSvg,
    type SvgPoint
  } from './curvePlot';
  import { CHANNEL_LABELS, dragPoint, insertPoint, nudgePoint, removePoint } from './curves';

  const FINE = 0.25;
  const FALLBACK_WIDTH = 300;

  let { channel, selected = $bindable(null) }: { channel: CurveChannel; selected?: number | null } =
    $props();

  let width = $state(0);
  let dragging: number | null = $state(null);
  let keyboardDirty = $state(false);
  let grab: { point: SvgPoint; last: SvgPoint } | null = null;
  let shownChannel = untrack(() => channel);

  const channelStroke: Record<CurveChannel, string> = {
    composite: 'var(--color-image-light)',
    r: 'var(--color-curve-red)',
    g: 'var(--color-curve-green)',
    b: 'var(--color-curve-blue)',
    luma: 'var(--color-curve-luma)'
  };

  const plot = $derived(plotOf(width > 0 ? width : FALLBACK_WIDTH));
  const label = $derived(CHANNEL_LABELS[channel]);
  const activeCurve = $derived(getCurve(channel));
  const activePath = $derived(curvePath(plot, activeCurve));
  const grid = $derived(gridLines(plot));
  const layers = $derived(histLayers(plot, editor.meta?.histogram ?? null, channel));
  const overlayPaths = $derived(
    CURVE_CHANNELS.filter((ch) => ch !== channel && !curvesAreIdentity(getCurve(ch))).map((ch) => ({
      ch,
      d: curvePath(plot, getCurve(ch))
    }))
  );
  const shown = $derived(dragging === null ? null : (activeCurve[dragging] ?? null));
  const right = $derived(plot.width - plot.pad);
  const bottom = $derived(plot.height - plot.pad);

  $effect(() => {
    const next = channel;
    if (next === shownChannel) return;
    shownChannel = next;
    untrack(() => {
      dragging = null;
      selected = null;
      grab = null;
    });
  });

  function getCurve(ch: CurveChannel): CurvePoint[] {
    return editor.edits.basic.curves[ch];
  }

  function setCurve(pts: CurvePoint[]): void {
    editor.edits.basic.curves = { ...editor.edits.basic.curves, [channel]: pts };
  }

  function commit(): void {
    void editor.onCommit(`Curve ${label}`);
  }

  function svgPoint(e: MouseEvent): SvgPoint {
    const svg = (e.currentTarget as Element).closest('svg');
    if (!svg) return { x: 0, y: 0 };
    const rect = svg.getBoundingClientRect();
    const at = elementPoint(e, rect);
    const scale = rect.width > 0 ? plot.width / rect.width : 1;
    return { x: at.x * scale, y: at.y * scale };
  }

  function hitRadius(e: PointerEvent): number {
    return e.pointerType === 'mouse' ? MOUSE_HIT : TOUCH_HIT;
  }

  const drag = pointerDrag<true>({
    start(e) {
      if (e.button !== 0) return null;
      const at = svgPoint(e);
      const pts = getCurve(channel);
      const hit = hitTest(plot, pts, at, hitRadius(e));
      if (hit >= 0) {
        dragging = hit;
        selected = hit;
        grab = { point: toSvg(plot, pts[hit] ?? fromSvg(plot, at.x, at.y)), last: at };
      } else {
        const { points, index } = insertPoint(pts, fromSvg(plot, at.x, at.y));
        setCurve(points);
        dragging = index;
        selected = index;
        grab = { point: at, last: at };
        editor.onLive();
      }
      e.preventDefault();
      return true;
    },
    move(e) {
      if (dragging === null || !grab) return;
      const at = svgPoint(e);
      const factor = held('fine', e) ? FINE : 1;
      const target = {
        x: grab.point.x + (at.x - grab.last.x) * factor,
        y: grab.point.y + (at.y - grab.last.y) * factor
      };
      grab.last = at;
      const next = dragPoint(getCurve(channel), dragging, fromSvg(plot, target.x, target.y));
      const moved = next?.[dragging];
      if (!next || !moved) return;
      grab.point = toSvg(plot, moved);
      setCurve(next);
      editor.onLive();
    },
    end() {
      grab = null;
      if (dragging === null) return;
      dragging = null;
      commit();
    }
  });

  function onDblClick(e: MouseEvent): void {
    const pts = getCurve(channel);
    const hit = hitTest(plot, pts, svgPoint(e), MOUSE_HIT, true);
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
    const step = held('coarse', e) ? 0.05 : 0.01;
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

<div class="relative w-full" bind:clientWidth={width}>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <svg
    viewBox="0 0 {plot.width} {plot.height}"
    style:height="{plot.height}px"
    class="block w-full cursor-crosshair touch-none rounded-sm bg-plot-surface ring-1 ring-hairline select-none focus:outline-none focus-visible:ring-2 focus-visible:ring-primary"
    {...drag.on()}
    ondblclick={onDblClick}
    onkeydown={onKeyDown}
    onkeyup={onKeyUp}
    onblur={commitKeyboard}
    tabindex="0"
    role="application"
    aria-label="Curves – {label}"
  >
    <rect
      x={plot.pad}
      y={plot.pad}
      width={right - plot.pad}
      height={bottom - plot.pad}
      fill="var(--color-curve-surface)"
      rx="2"
    />

    {#each layers as layer (layer.fill)}
      {#if layer.d}
        <path d={layer.d} fill={layer.fill} style:mix-blend-mode="screen" />
      {/if}
    {/each}

    {#each grid as l, i (i)}
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
      x1={plot.pad}
      y1={bottom}
      x2={right}
      y2={plot.pad}
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
        opacity="0.3"
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
      {@const sp = toSvg(plot, pt)}
      {@const active = dragging === i || selected === i}
      {#if active}
        <circle cx={sp.x} cy={sp.y} r="13" fill={channelStroke[channel]} opacity="0.18" />
      {/if}
      <circle
        cx={sp.x}
        cy={sp.y}
        r={active ? 7 : 5}
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
  {#if shown}
    <span
      data-curve-readout
      class="pointer-events-none absolute top-2 left-2 rounded-md bg-editor-chrome/85 px-1.5 py-0.5 font-mono text-[11px] text-light-950 tabular-nums"
    >
      In {levelOf(shown.x)} · Out {levelOf(shown.y)}
    </span>
  {/if}
</div>
