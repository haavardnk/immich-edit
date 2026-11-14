import type { CurveChannel, CurvePoint } from '#lib/types/edits.ts';

const MIN_GAP = 0.01;
const PICK_SNAP = 0.03;

export const CHANNEL_LABELS: Record<CurveChannel, string> = {
  composite: 'RGB',
  r: 'Red',
  g: 'Green',
  b: 'Blue',
  luma: 'Luma'
};

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

export function pickPoint(
  pts: CurvePoint[],
  x: number,
  snap = PICK_SNAP
): { points: CurvePoint[]; index: number } {
  const level = Math.max(0, Math.min(1, x));
  const nearest = pts.reduce(
    (best, p, i) => (Math.abs(p.x - level) < Math.abs((pts[best]?.x ?? 0) - level) ? i : best),
    0
  );
  const near = pts[nearest];
  if (near && Math.abs(near.x - level) <= snap) return { points: pts, index: nearest };
  return insertPoint(pts, { x: level, y: hermite(pts, level) });
}
