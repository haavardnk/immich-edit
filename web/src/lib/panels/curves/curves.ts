import type { CurveChannel, CurvePoint } from '$lib/types/edits';
import type { Histogram } from '$lib/types/preview';

export const CURVE_SIZE = 232;
export const CURVE_PAD = 16;
export const CURVE_INNER = CURVE_SIZE - 2 * CURVE_PAD;

const HIT_RADIUS = 12;
const MIN_GAP = 0.01;
const PATH_STEPS = 64;

export const CHANNEL_LABELS: Record<CurveChannel, string> = {
  composite: 'RGB',
  r: 'Red',
  g: 'Green',
  b: 'Blue',
  luma: 'Luma'
};

export type SvgPoint = { x: number; y: number };
export type GridLine = { x1: number; y1: number; x2: number; y2: number };
export type HistLayer = { d: string; fill: string };
type HistSource = { key: keyof Histogram; fill: string };

const CHANNEL_HIST: Record<Exclude<CurveChannel, 'composite'>, HistSource> = {
  r: { key: 'r', fill: 'color-mix(in srgb, var(--color-curve-red) 35%, transparent)' },
  g: { key: 'g', fill: 'color-mix(in srgb, var(--color-curve-green) 35%, transparent)' },
  b: { key: 'b', fill: 'color-mix(in srgb, var(--color-curve-blue) 35%, transparent)' },
  luma: { key: 'l', fill: 'color-mix(in srgb, var(--color-channel-luma) 18%, transparent)' }
};

const COMPOSITE_HIST: HistSource[] = [
  { key: 'r', fill: 'color-mix(in srgb, var(--color-curve-red) 22%, transparent)' },
  { key: 'g', fill: 'color-mix(in srgb, var(--color-curve-green) 22%, transparent)' },
  { key: 'b', fill: 'color-mix(in srgb, var(--color-curve-blue) 22%, transparent)' }
];

export const GRID_LINES: GridLine[] = [1, 2, 3].flatMap((i) => {
  const pos = CURVE_PAD + (i / 4) * CURVE_INNER;
  return [
    { x1: pos, y1: CURVE_PAD, x2: pos, y2: CURVE_PAD + CURVE_INNER },
    { x1: CURVE_PAD, y1: pos, x2: CURVE_PAD + CURVE_INNER, y2: pos }
  ];
});

export function toSvg(p: CurvePoint): SvgPoint {
  return { x: CURVE_PAD + p.x * CURVE_INNER, y: CURVE_PAD + (1 - p.y) * CURVE_INNER };
}

export function fromSvg(sx: number, sy: number): CurvePoint {
  const x = Math.max(0, Math.min(1, (sx - CURVE_PAD) / CURVE_INNER));
  const y = Math.max(0, Math.min(1, 1 - (sy - CURVE_PAD) / CURVE_INNER));
  return { x, y };
}

function slope(a: CurvePoint, b: CurvePoint): number {
  return (b.y - a.y) / Math.max(b.x - a.x, 1e-10);
}

function tangent(pts: CurvePoint[], i: number): number {
  const prev = pts[i - 1];
  const cur = pts[i];
  const next = pts[i + 1];
  if (!cur) return 0;
  if (!prev) return next ? slope(cur, next) : 0;
  if (!next) return slope(prev, cur);
  const d0 = slope(prev, cur);
  const d1 = slope(cur, next);
  if (Math.sign(d0) !== Math.sign(d1)) return 0;
  return (d0 + d1) * 0.5;
}

export function hermite(pts: CurvePoint[], x: number): number {
  const first = pts[0];
  const last = pts[pts.length - 1];
  if (!first || !last || pts.length < 2) return x;
  if (x <= first.x) return first.y;
  if (x >= last.x) return last.y;
  const found = pts.findIndex((a, i) => {
    const b = pts[i + 1];
    return !!b && x >= a.x && x <= b.x;
  });
  const idx = Math.max(0, found);
  const p0 = pts[idx];
  const p1 = pts[idx + 1];
  if (!p0 || !p1) return x;
  const dx = p1.x - p0.x;
  if (dx < 1e-10) return p0.y;
  const t = (x - p0.x) / dx;
  const m0 = tangent(pts, idx);
  const m1 = tangent(pts, idx + 1);
  const t2 = t * t;
  const t3 = t2 * t;
  const v =
    (2 * t3 - 3 * t2 + 1) * p0.y +
    (t3 - 2 * t2 + t) * dx * m0 +
    (-2 * t3 + 3 * t2) * p1.y +
    (t3 - t2) * dx * m1;
  return Math.max(0, Math.min(1, v));
}

export function curvePath(pts: CurvePoint[]): string {
  if (pts.length < 2) return '';
  return Array.from({ length: PATH_STEPS + 1 }, (_, i) => {
    const t = i / PATH_STEPS;
    const sv = toSvg({ x: t, y: hermite(pts, t) });
    return `${i === 0 ? 'M' : 'L'}${sv.x.toFixed(1)},${sv.y.toFixed(1)}`;
  }).join(' ');
}

function histAreaPath(data: number[] | undefined): string {
  if (!data || data.length === 0) return '';
  const max = Math.max(...data, 1);
  const base = CURVE_PAD + CURVE_INNER;
  const steps = data.map((count, i) => {
    const x = CURVE_PAD + (i / (data.length - 1)) * CURVE_INNER;
    const y = base - (count / max) * CURVE_INNER * 0.8;
    return ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
  });
  return `M ${CURVE_PAD} ${base}${steps.join('')} L ${base} ${base} Z`;
}

export function histLayers(hist: Histogram | null, channel: CurveChannel): HistLayer[] {
  if (!hist) return [];
  const sources = channel === 'composite' ? COMPOSITE_HIST : [CHANNEL_HIST[channel]];
  return sources.map(({ key, fill }) => ({ d: histAreaPath(hist[key]), fill }));
}

export function hitTest(pts: CurvePoint[], sx: number, sy: number, interior = false): number {
  return pts.findIndex((p, i) => {
    if (interior && (i === 0 || i === pts.length - 1)) return false;
    const sp = toSvg(p);
    return Math.hypot(sx - sp.x, sy - sp.y) < HIT_RADIUS;
  });
}

export function insertPoint(
  pts: CurvePoint[],
  p: CurvePoint
): { points: CurvePoint[]; index: number } {
  const after = pts.findIndex((q) => q.x > p.x);
  const index = after < 0 ? pts.length : after;
  return { points: [...pts.slice(0, index), p, ...pts.slice(index)], index };
}

function between(pts: CurvePoint[], idx: number, x: number): number | null {
  const before = pts[idx - 1];
  const after = pts[idx + 1];
  if (!before || !after) return null;
  return Math.max(before.x + MIN_GAP, Math.min(after.x - MIN_GAP, x));
}

export function dragPoint(pts: CurvePoint[], idx: number, p: CurvePoint): CurvePoint[] | null {
  const next = [...pts];
  if (idx === 0) {
    next[0] = { x: 0, y: p.y };
    return next;
  }
  if (idx === pts.length - 1) {
    next[idx] = { x: 1, y: p.y };
    return next;
  }
  const x = between(pts, idx, p.x);
  if (x === null) return null;
  next[idx] = { x, y: p.y };
  return next;
}

export function nudgePoint(
  pts: CurvePoint[],
  idx: number,
  dx: number,
  dy: number
): CurvePoint[] | null {
  const p = pts[idx];
  if (!p) return null;
  const y = Math.max(0, Math.min(1, p.y + dy));
  const next = [...pts];
  if (idx === 0 || idx === pts.length - 1) {
    next[idx] = { x: p.x, y };
    return next;
  }
  const x = between(pts, idx, p.x + dx);
  if (x === null) return null;
  next[idx] = { x, y };
  return next;
}

export function removePoint(pts: CurvePoint[], idx: number): CurvePoint[] | null {
  if (idx <= 0 || idx >= pts.length - 1) return null;
  return pts.filter((_, i) => i !== idx);
}
