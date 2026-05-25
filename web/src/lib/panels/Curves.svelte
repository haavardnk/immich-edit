<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    CURVE_CHANNELS,
    identityCurve,
    neutralCurves,
    type CurveChannel,
    type CurvePoint
  } from '$lib/types/edits';

  let activeChannel: CurveChannel = $state('composite');
  let dragging: number | null = $state(null);
  let selected: number | null = $state(null);
  let pointerId: number | null = null;

  const size = 232;
  const pad = 16;
  const inner = size - 2 * pad;

  const hist = $derived(editor.meta?.histogram ?? null);

  const channelLabels: Record<CurveChannel, string> = {
    composite: 'RGB',
    r: 'Red',
    g: 'Green',
    b: 'Blue',
    luma: 'Luma'
  };

  const channelStroke: Record<CurveChannel, string> = {
    composite: 'rgb(240,240,240)',
    r: 'rgb(255,90,90)',
    g: 'rgb(90,220,120)',
    b: 'rgb(110,160,255)',
    luma: 'rgb(240,210,90)'
  };

  const channelSwatchStyle: Record<CurveChannel, string> = {
    composite:
      'background: linear-gradient(135deg, rgba(255,90,90,0.55), rgba(90,220,120,0.55), rgba(110,160,255,0.55))',
    r: 'background: rgb(220,60,60)',
    g: 'background: rgb(60,180,90)',
    b: 'background: rgb(70,120,220)',
    luma: 'background: linear-gradient(90deg, #000, #fff)'
  };

  function getCurve(ch: CurveChannel): CurvePoint[] {
    return editor.edits.basic.curves[ch];
  }

  function setCurve(ch: CurveChannel, pts: CurvePoint[]): void {
    editor.edits.basic.curves = { ...editor.edits.basic.curves, [ch]: pts };
  }

  function toSvg(p: CurvePoint): { x: number; y: number } {
    return { x: pad + p.x * inner, y: pad + (1 - p.y) * inner };
  }

  function fromSvg(sx: number, sy: number): CurvePoint {
    const x = Math.max(0, Math.min(1, (sx - pad) / inner));
    const y = Math.max(0, Math.min(1, 1 - (sy - pad) / inner));
    return { x, y };
  }

  function hermite(pts: CurvePoint[], x: number): number {
    if (pts.length < 2) return x;
    if (x <= pts[0].x) return pts[0].y;
    if (x >= pts[pts.length - 1].x) return pts[pts.length - 1].y;
    let idx = 0;
    for (let i = 0; i < pts.length - 1; i++) {
      if (x >= pts[i].x && x <= pts[i + 1].x) {
        idx = i;
        break;
      }
    }
    const x0 = pts[idx].x;
    const y0 = pts[idx].y;
    const x1 = pts[idx + 1].x;
    const y1 = pts[idx + 1].y;
    const dx = x1 - x0;
    if (dx < 1e-10) return y0;
    const t = (x - x0) / dx;
    const m0 = tangent(pts, idx);
    const m1 = tangent(pts, idx + 1);
    const t2 = t * t;
    const t3 = t2 * t;
    const v =
      (2 * t3 - 3 * t2 + 1) * y0 +
      (t3 - 2 * t2 + t) * dx * m0 +
      (-2 * t3 + 3 * t2) * y1 +
      (t3 - t2) * dx * m1;
    return Math.max(0, Math.min(1, v));
  }

  function tangent(pts: CurvePoint[], i: number): number {
    if (i === 0) return (pts[1].y - pts[0].y) / Math.max(pts[1].x - pts[0].x, 1e-10);
    if (i === pts.length - 1) {
      const n = pts.length;
      return (pts[n - 1].y - pts[n - 2].y) / Math.max(pts[n - 1].x - pts[n - 2].x, 1e-10);
    }
    const d0 = (pts[i].y - pts[i - 1].y) / Math.max(pts[i].x - pts[i - 1].x, 1e-10);
    const d1 = (pts[i + 1].y - pts[i].y) / Math.max(pts[i + 1].x - pts[i].x, 1e-10);
    if (Math.sign(d0) !== Math.sign(d1)) return 0;
    return (d0 + d1) * 0.5;
  }

  function curvePathFor(pts: CurvePoint[]): string {
    if (pts.length < 2) return '';
    const steps = 64;
    const parts: string[] = [];
    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      const y = hermite(pts, t);
      const sv = toSvg({ x: t, y });
      parts.push(`${i === 0 ? 'M' : 'L'}${sv.x.toFixed(1)},${sv.y.toFixed(1)}`);
    }
    return parts.join(' ');
  }

  function isIdentityCurve(pts: CurvePoint[]): boolean {
    return (
      pts.length === 2 &&
      Math.abs(pts[0].x) < 1e-10 &&
      Math.abs(pts[0].y) < 1e-10 &&
      Math.abs(pts[1].x - 1) < 1e-10 &&
      Math.abs(pts[1].y - 1) < 1e-10
    );
  }

  const activeCurve = $derived(getCurve(activeChannel));
  const activePath = $derived(curvePathFor(activeCurve));

  const overlayPaths = $derived.by(() => {
    const out: { ch: CurveChannel; d: string }[] = [];
    for (const ch of CURVE_CHANNELS) {
      if (ch === activeChannel) continue;
      const pts = getCurve(ch);
      if (isIdentityCurve(pts)) continue;
      out.push({ ch, d: curvePathFor(pts) });
    }
    return out;
  });

  const histPath = $derived.by(() => {
    if (!hist) return '';
    const lum = hist.l;
    if (lum.length === 0) return '';
    const max = Math.max(...lum, 1);
    let d = `M ${pad} ${pad + inner}`;
    for (let i = 0; i < lum.length; i++) {
      const x = pad + (i / (lum.length - 1)) * inner;
      const y = pad + inner - (lum[i] / max) * inner * 0.8;
      d += ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
    }
    d += ` L ${pad + inner} ${pad + inner} Z`;
    return d;
  });

  const gridLines = $derived.by(() => {
    const lines: { x1: number; y1: number; x2: number; y2: number }[] = [];
    for (let i = 1; i < 4; i++) {
      const pos = pad + (i / 4) * inner;
      lines.push({ x1: pos, y1: pad, x2: pos, y2: pad + inner });
      lines.push({ x1: pad, y1: pos, x2: pad + inner, y2: pos });
    }
    return lines;
  });

  function svgCoords(e: PointerEvent, target: SVGElement): { sx: number; sy: number } {
    const svg = target.closest('svg') as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const scale = size / rect.width;
    return { sx: (e.clientX - rect.left) * scale, sy: (e.clientY - rect.top) * scale };
  }

  function hitTest(pts: CurvePoint[], sx: number, sy: number): number {
    for (let i = 0; i < pts.length; i++) {
      const sp = toSvg(pts[i]);
      if (Math.hypot(sx - sp.x, sy - sp.y) < 12) return i;
    }
    return -1;
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== undefined && e.button !== 0) return;
    const target = e.currentTarget as SVGElement;
    const { sx, sy } = svgCoords(e, target);
    const pts = getCurve(activeChannel);
    const hit = hitTest(pts, sx, sy);
    if (hit >= 0) {
      dragging = hit;
      selected = hit;
    } else {
      const np = fromSvg(sx, sy);
      let insertIdx = pts.length;
      for (let i = 0; i < pts.length; i++) {
        if (pts[i].x > np.x) {
          insertIdx = i;
          break;
        }
      }
      const newPts = [...pts];
      newPts.splice(insertIdx, 0, np);
      setCurve(activeChannel, newPts);
      dragging = insertIdx;
      selected = insertIdx;
      editor.onLive();
    }
    pointerId = e.pointerId;
    target.setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function onPointerMove(e: PointerEvent) {
    if (dragging === null) return;
    const target = e.currentTarget as SVGElement;
    const { sx, sy } = svgCoords(e, target);
    const pts = [...getCurve(activeChannel)];
    const np = fromSvg(sx, sy);
    if (dragging === 0) {
      pts[0] = { x: 0, y: np.y };
    } else if (dragging === pts.length - 1) {
      pts[pts.length - 1] = { x: 1, y: np.y };
    } else {
      const minX = pts[dragging - 1].x + 0.01;
      const maxX = pts[dragging + 1].x - 0.01;
      pts[dragging] = { x: Math.max(minX, Math.min(maxX, np.x)), y: np.y };
    }
    setCurve(activeChannel, pts);
    editor.onLive();
  }

  function endDrag(e: PointerEvent) {
    if (dragging === null) return;
    dragging = null;
    if (pointerId !== null) {
      const target = e.currentTarget as SVGElement;
      target.releasePointerCapture(pointerId);
      pointerId = null;
    }
    editor.onCommit();
  }

  function onDblClick(e: MouseEvent) {
    const target = e.currentTarget as SVGElement;
    const svg = target.closest('svg') as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const scale = size / rect.width;
    const sx = (e.clientX - rect.left) * scale;
    const sy = (e.clientY - rect.top) * scale;
    const pts = getCurve(activeChannel);
    for (let i = 1; i < pts.length - 1; i++) {
      const sp = toSvg(pts[i]);
      if (Math.hypot(sx - sp.x, sy - sp.y) < 12) {
        const newPts = [...pts];
        newPts.splice(i, 1);
        setCurve(activeChannel, newPts);
        if (selected === i) selected = null;
        editor.onCommit();
        return;
      }
    }
  }

  function nudge(idx: number, dy: number, dx: number) {
    const pts = [...getCurve(activeChannel)];
    const isEndpoint = idx === 0 || idx === pts.length - 1;
    const p = pts[idx];
    const ny = Math.max(0, Math.min(1, p.y + dy));
    if (isEndpoint) {
      pts[idx] = { x: p.x, y: ny };
    } else {
      const minX = pts[idx - 1].x + 0.01;
      const maxX = pts[idx + 1].x - 0.01;
      const nx = Math.max(minX, Math.min(maxX, p.x + dx));
      pts[idx] = { x: nx, y: ny };
    }
    setCurve(activeChannel, pts);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (selected === null) return;
    const pts = getCurve(activeChannel);
    const step = e.shiftKey ? 0.05 : 0.01;
    if (e.key === 'ArrowUp') {
      nudge(selected, step, 0);
      editor.onLive();
      editor.onCommit();
      e.preventDefault();
    } else if (e.key === 'ArrowDown') {
      nudge(selected, -step, 0);
      editor.onLive();
      editor.onCommit();
      e.preventDefault();
    } else if (e.key === 'ArrowLeft') {
      nudge(selected, 0, -step);
      editor.onLive();
      editor.onCommit();
      e.preventDefault();
    } else if (e.key === 'ArrowRight') {
      nudge(selected, 0, step);
      editor.onLive();
      editor.onCommit();
      e.preventDefault();
    } else if (e.key === 'Delete' || e.key === 'Backspace') {
      if (selected > 0 && selected < pts.length - 1) {
        const newPts = [...pts];
        newPts.splice(selected, 1);
        setCurve(activeChannel, newPts);
        selected = null;
        editor.onCommit();
      }
      e.preventDefault();
    } else if (e.key === 'Escape') {
      selected = null;
      e.preventDefault();
    }
  }

  function selectChannel(ch: CurveChannel) {
    if (activeChannel === ch) return;
    activeChannel = ch;
    dragging = null;
    selected = null;
  }

  function resetActive() {
    setCurve(activeChannel, identityCurve());
    selected = null;
    editor.onCommit();
  }

  function resetAll() {
    editor.edits.basic.curves = neutralCurves();
    selected = null;
    editor.onCommit();
  }
</script>

<div class="flex flex-col gap-2.5">
  <div class="grid grid-cols-5 gap-1">
    {#each CURVE_CHANNELS as ch (ch)}
      <button
        type="button"
        class="h-7 rounded ring-1 ring-white/10 hover:ring-white/40 transition-shadow {activeChannel === ch ? 'ring-2 ring-white/80' : ''}"
        style={channelSwatchStyle[ch]}
        title={channelLabels[ch]}
        aria-label="Edit {channelLabels[ch]} curve"
        aria-pressed={activeChannel === ch}
        onclick={() => selectChannel(ch)}
      ></button>
    {/each}
  </div>

  <div class="flex items-center justify-between px-1">
    <div class="text-[11px] text-immich-dark-fg/70">{channelLabels[activeChannel]}</div>
    <div class="flex items-center gap-3">
      <button
        type="button"
        class="flex items-center gap-1 text-[10px] text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors"
        title="Reset {channelLabels[activeChannel]} curve"
        onclick={resetActive}
      >
        <Icon path={mdiRestore} size={12} />
        Curve
      </button>
      <button
        type="button"
        class="flex items-center gap-1 text-[10px] text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors"
        title="Reset all curves"
        onclick={resetAll}
      >
        <Icon path={mdiRestore} size={12} />
        All
      </button>
    </div>
  </div>

  <div class="flex flex-col items-center">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <svg
      viewBox="0 0 {size} {size}"
      class="w-full aspect-square rounded-lg cursor-crosshair select-none bg-neutral-900/80 focus:outline-none focus-visible:ring-2 focus-visible:ring-white/60"
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={endDrag}
      onpointercancel={endDrag}
      ondblclick={onDblClick}
      onkeydown={onKeyDown}
      tabindex="0"
      role="application"
      aria-label="Curves – {channelLabels[activeChannel]}"
    >
      <rect x={pad} y={pad} width={inner} height={inner} fill="rgba(0,0,0,0.3)" rx="2" />

      {#if histPath}
        <path d={histPath} fill="rgba(255,255,255,0.08)" />
      {/if}

      {#each gridLines as l}
        <line x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2} stroke="rgba(255,255,255,0.06)" stroke-width="0.5" />
      {/each}

      <line x1={pad} y1={pad + inner} x2={pad + inner} y2={pad} stroke="rgba(255,255,255,0.15)" stroke-width="1" stroke-dasharray="3,3" />

      {#each overlayPaths as overlay (overlay.ch)}
        <path d={overlay.d} fill="none" stroke={channelStroke[overlay.ch]} stroke-width="1.25" stroke-linecap="round" opacity="0.25" />
      {/each}

      <path d={activePath} fill="none" stroke={channelStroke[activeChannel]} stroke-width="2" stroke-linecap="round" />

      {#each activeCurve as pt, i}
        {@const sp = toSvg(pt)}
        <circle
          cx={sp.x}
          cy={sp.y}
          r={dragging === i || selected === i ? 7 : 5}
          fill={dragging === i ? channelStroke[activeChannel] : selected === i ? 'rgba(255,255,255,0.9)' : 'rgba(30,30,30,0.9)'}
          stroke={channelStroke[activeChannel]}
          stroke-width="2"
        />
      {/each}
    </svg>
  </div>
</div>

