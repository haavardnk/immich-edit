import { describe, expect, it } from 'vitest';
import type { CurvePoint } from '$lib/types/edits';
import type { Histogram } from '$lib/types/preview';
import {
  curvePath,
  dragPoint,
  fromSvg,
  hermite,
  histLayers,
  hitTest,
  insertPoint,
  nudgePoint,
  removePoint,
  toSvg
} from './curves';

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

describe('svg mapping', () => {
  it('round trips a point', () => {
    const sp = toSvg({ x: 0.3, y: 0.7 });
    const back = fromSvg(sp.x, sp.y);
    expect(back.x).toBeCloseTo(0.3, 9);
    expect(back.y).toBeCloseTo(0.7, 9);
  });

  it('clamps outside the plot', () => {
    expect(fromSvg(-50, 500)).toEqual({ x: 0, y: 0 });
    expect(fromSvg(500, -50)).toEqual({ x: 1, y: 1 });
  });

  it('draws 65 segments for a curve and nothing for one point', () => {
    expect(curvePath(S_CURVE).split(' ')).toHaveLength(65);
    expect(curvePath([{ x: 0, y: 0 }])).toBe('');
  });
});

describe('point editing', () => {
  it.each([
    [false, toSvg({ x: 0, y: 0 }), 0],
    [true, toSvg({ x: 0, y: 0 }), -1],
    [true, toSvg({ x: 0.25, y: 0.15 }), 1],
    [false, { x: 116, y: 30 }, -1]
  ])('hitTest interior=%s', (interior, at, idx) => {
    expect(hitTest(S_CURVE, at.x, at.y, interior)).toBe(idx);
  });

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

describe('histLayers', () => {
  const hist: Histogram = { r: [1, 2], g: [2, 1], b: [0, 1], l: [1, 1] };

  it.each([
    ['composite', 3],
    ['r', 1],
    ['luma', 1]
  ] as const)('%s', (channel, count) => {
    const layers = histLayers(hist, channel);
    expect(layers).toHaveLength(count);
    expect(layers.every((layer) => layer.d.startsWith('M ') && layer.d.endsWith(' Z'))).toBe(true);
  });

  it('is empty without a histogram', () => {
    expect(histLayers(null, 'composite')).toEqual([]);
  });
});
