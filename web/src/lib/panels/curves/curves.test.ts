import { describe, expect, it } from 'vitest';
import type { CurvePoint } from '#lib/types/edits.ts';
import { dragPoint, hermite, insertPoint, nudgePoint, pickPoint, removePoint } from './curves';

const IDENTITY: CurvePoint[] = [
  { x: 0, y: 0 },
  { x: 1, y: 1 }
];
const S_CURVE: CurvePoint[] = [
  { x: 0, y: 0 },
  { x: 0.25, y: 0.15 },
  { x: 0.75, y: 0.85 },
  { x: 1, y: 1 }
];
const PEAK: CurvePoint[] = [
  { x: 0, y: 0 },
  { x: 0.5, y: 1 },
  { x: 1, y: 0 }
];
const RAISED: CurvePoint[] = [
  { x: 0.2, y: 0.1 },
  { x: 0.8, y: 0.9 }
];

describe('hermite', () => {
  it.each([
    [IDENTITY, 0.3, 0.3],
    [IDENTITY, 0.75, 0.75],
    [S_CURVE, 0.25, 0.15],
    [S_CURVE, 0.75, 0.85],
    [PEAK, 0.5, 1],
    [RAISED, 0, 0.1],
    [RAISED, 1, 0.9],
    [[{ x: 0, y: 0.5 }], 0.3, 0.3]
  ])('curve %# at %f', (pts, x, y) => {
    expect(hermite(pts, x)).toBeCloseTo(y, 6);
  });

  it('stays in range around a peak', () => {
    const samples = Array.from({ length: 101 }, (_, i) => hermite(PEAK, i / 100));
    expect(Math.max(...samples)).toBeLessThanOrEqual(1);
    expect(Math.min(...samples)).toBeGreaterThanOrEqual(0);
  });
});

describe('point editing', () => {
  it('inserts in x order', () => {
    const { points, index } = insertPoint(S_CURVE, { x: 0.5, y: 0.5 });
    expect(index).toBe(2);
    expect(points.map((p) => p.x)).toEqual([0, 0.25, 0.5, 0.75, 1]);
  });

  it.each([
    [0, { x: 0.4, y: 0.2 }, { x: 0, y: 0.2 }],
    [3, { x: 0.4, y: 0.6 }, { x: 1, y: 0.6 }],
    [1, { x: 0.9, y: 0.3 }, { x: 0.74, y: 0.3 }],
    [2, { x: 0.1, y: 0.8 }, { x: 0.26, y: 0.8 }]
  ])('dragPoint %i', (idx, to, expected) => {
    const moved = dragPoint(S_CURVE, idx, to)?.[idx];
    expect(moved?.x).toBeCloseTo(expected.x, 9);
    expect(moved?.y).toBeCloseTo(expected.y, 9);
  });

  it.each([
    [0, 0.1, 0.1, { x: 0, y: 0.1 }],
    [3, -0.1, 0.5, { x: 1, y: 1 }],
    [1, 0.05, -0.5, { x: 0.3, y: 0 }],
    [1, 1, 0, { x: 0.74, y: 0.15 }]
  ])('nudgePoint %i', (idx, dx, dy, expected) => {
    const moved = nudgePoint(S_CURVE, idx, dx, dy)?.[idx];
    expect(moved?.x).toBeCloseTo(expected.x, 9);
    expect(moved?.y).toBeCloseTo(expected.y, 9);
  });

  it.each([
    [0, null],
    [3, null],
    [1, [0, 0.75, 1]]
  ])('removePoint %i', (idx, xs) => {
    expect(removePoint(S_CURVE, idx)?.map((p) => p.x) ?? null).toEqual(xs);
  });
});

describe('pickPoint', () => {
  it('selects the point nearest the tone when it is close', () => {
    expect(pickPoint(S_CURVE, 0.27)).toEqual({ points: S_CURVE, index: 1 });
    expect(pickPoint(S_CURVE, 0.99)).toEqual({ points: S_CURVE, index: 3 });
  });

  it('adds a point on the curve when none is close', () => {
    const { points, index } = pickPoint(S_CURVE, 0.5);
    expect(index).toBe(2);
    expect(points).toHaveLength(5);
    expect(points[2]).toEqual({ x: 0.5, y: hermite(S_CURVE, 0.5) });
  });
});
