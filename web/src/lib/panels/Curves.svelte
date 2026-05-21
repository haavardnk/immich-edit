<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import type { CurvePoint } from '$lib/types/edits';

  let dragging: number | null = $state(null);

  const size = 232;
  const pad = 16;
  const inner = size - 2 * pad;

  const hist = $derived(editor.meta?.histogram ?? null);

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
      if (x >= pts[i].x && x <= pts[i + 1].x) { idx = i; break; }
    }
    const x0 = pts[idx].x, y0 = pts[idx].y;
    const x1 = pts[idx + 1].x, y1 = pts[idx + 1].y;
    const dx = x1 - x0;
    if (dx < 1e-10) return y0;
    const t = (x - x0) / dx;

    const m0 = tangent(pts, idx);
    const m1 = tangent(pts, idx + 1);
    const t2 = t * t, t3 = t2 * t;
    const v = (2 * t3 - 3 * t2 + 1) * y0
      + (t3 - 2 * t2 + t) * dx * m0
      + (-2 * t3 + 3 * t2) * y1
      + (t3 - t2) * dx * m1;
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

  const curvePath = $derived.by(() => {
    const pts = editor.edits.basic.curves;
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

  function onMouseDown(e: MouseEvent) {
    const svg = (e.currentTarget as SVGElement).closest('svg')!;
    const rect = svg.getBoundingClientRect();
    const scale = size / rect.width;
    const sx = (e.clientX - rect.left) * scale;
    const sy = (e.clientY - rect.top) * scale;
    const pts = editor.edits.basic.curves;

    for (let i = 0; i < pts.length; i++) {
      const sp = toSvg(pts[i]);
      if (Math.hypot(sx - sp.x, sy - sp.y) < 12) {
        dragging = i;
        return;
      }
    }

    const np = fromSvg(sx, sy);
    let insertIdx = pts.length;
    for (let i = 0; i < pts.length; i++) {
      if (pts[i].x > np.x) { insertIdx = i; break; }
    }
    const newPts = [...pts];
    newPts.splice(insertIdx, 0, np);
    editor.edits.basic.curves = newPts;
    dragging = insertIdx;
    editor.onLive();
  }

  function onMouseMove(e: MouseEvent) {
    if (dragging === null) return;
    const svg = (e.currentTarget as SVGElement).closest('svg')!;
    const rect = svg.getBoundingClientRect();
    const scale = size / rect.width;
    const sx = (e.clientX - rect.left) * scale;
    const sy = (e.clientY - rect.top) * scale;
    const pts = [...editor.edits.basic.curves];
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
    editor.edits.basic.curves = pts;
    editor.onLive();
  }

  function onMouseUp() {
    if (dragging !== null) {
      dragging = null;
      editor.onCommit();
    }
  }

  function onDblClick(e: MouseEvent) {
    const svg = (e.currentTarget as SVGElement).closest('svg')!;
    const rect = svg.getBoundingClientRect();
    const scale = size / rect.width;
    const sx = (e.clientX - rect.left) * scale;
    const sy = (e.clientY - rect.top) * scale;
    const pts = editor.edits.basic.curves;

    for (let i = 1; i < pts.length - 1; i++) {
      const sp = toSvg(pts[i]);
      if (Math.hypot(sx - sp.x, sy - sp.y) < 12) {
        const newPts = [...pts];
        newPts.splice(i, 1);
        editor.edits.basic.curves = newPts;
        editor.onCommit();
        return;
      }
    }
  }
</script>

<div class="flex flex-col items-center gap-1.5">
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <svg
    viewBox="0 0 {size} {size}"
    class="w-full aspect-square rounded-lg cursor-crosshair select-none bg-neutral-900/80"
    onmousedown={onMouseDown}
    onmousemove={onMouseMove}
    onmouseup={onMouseUp}
    onmouseleave={onMouseUp}
    ondblclick={onDblClick}
    role="application"
  >
    <rect x={pad} y={pad} width={inner} height={inner} fill="rgba(0,0,0,0.3)" rx="2" />

    {#if histPath}
      <path d={histPath} fill="rgba(255,255,255,0.08)" />
    {/if}

    {#each gridLines as l}
      <line x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2} stroke="rgba(255,255,255,0.06)" stroke-width="0.5" />
    {/each}

    <line x1={pad} y1={pad + inner} x2={pad + inner} y2={pad} stroke="rgba(255,255,255,0.15)" stroke-width="1" stroke-dasharray="3,3" />

    <path d={curvePath} fill="none" stroke="white" stroke-width="2" stroke-linecap="round" />

    {#each editor.edits.basic.curves as pt, i}
      {@const sp = toSvg(pt)}
      <circle
        cx={sp.x}
        cy={sp.y}
        r={dragging === i ? 7 : 5}
        fill={dragging === i ? 'white' : 'rgba(30,30,30,0.9)'}
        stroke="white"
        stroke-width="2"
      />
    {/each}
  </svg>
  <p class="text-[10px] text-base-content/40">Click add · Double-click remove</p>
</div>
