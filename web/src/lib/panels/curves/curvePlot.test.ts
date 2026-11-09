import { describe, expect, it } from 'vitest';
import type { CurvePoint } from '#lib/types/edits.ts';
import type { Histogram } from '#lib/types/preview.ts';
import {
  MOUSE_HIT,
  curvePath,
  fromLevel,
  fromSvg,
  gridLines,
  histLayers,
  hitTest,
  levelOf,
  plotOf,
  toSvg
} from './curvePlot';

const PLOT = plotOf(300);
const S_CURVE: CurvePoint[] = [
  { x: 0, y: 0 },
  { x: 0.25, y: 0.15 },
  { x: 0.75, y: 0.85 },
  { x: 1, y: 1 }
];

describe('plot geometry', () => {
  it('is 1.5:1', () => {
    expect(PLOT).toEqual({ width: 300, height: 200, pad: 10 });
  });

  it('round trips a point', () => {
    const at = toSvg(PLOT, { x: 0.3, y: 0.7 });
    const back = fromSvg(PLOT, at.x, at.y);
    expect(back.x).toBeCloseTo(0.3, 9);
    expect(back.y).toBeCloseTo(0.7, 9);
  });

  it('maps the corners to the padded inner box', () => {
    expect(toSvg(PLOT, { x: 0, y: 0 })).toEqual({ x: 10, y: 190 });
    expect(toSvg(PLOT, { x: 1, y: 1 })).toEqual({ x: 290, y: 10 });
  });

  it('clamps outside the plot', () => {
    expect(fromSvg(PLOT, -50, 500)).toEqual({ x: 0, y: 0 });
    expect(fromSvg(PLOT, 500, -50)).toEqual({ x: 1, y: 1 });
  });

  it('draws three lines each way', () => {
    expect(gridLines(PLOT)).toHaveLength(6);
  });

  it('draws 65 segments for a curve and nothing for one point', () => {
    expect(curvePath(PLOT, S_CURVE).split(' ')).toHaveLength(65);
    expect(curvePath(PLOT, [{ x: 0, y: 0 }])).toBe('');
  });
});

describe('hitTest', () => {
  it.each([
    [false, { x: 0, y: 0 }, 0],
    [true, { x: 0, y: 0 }, -1],
    [true, { x: 0.25, y: 0.15 }, 1]
  ])('interior=%s at %o', (interior, point, idx) => {
    expect(hitTest(PLOT, S_CURVE, toSvg(PLOT, point), MOUSE_HIT, interior)).toBe(idx);
  });

  it('misses beyond the radius', () => {
    const at = toSvg(PLOT, { x: 0.25, y: 0.15 });
    expect(hitTest(PLOT, S_CURVE, { x: at.x + MOUSE_HIT, y: at.y }, MOUSE_HIT)).toBe(-1);
  });

  it('picks the nearest of two points in reach', () => {
    const close: CurvePoint[] = [
      { x: 0, y: 0 },
      { x: 0.5, y: 0.5 },
      { x: 0.53, y: 0.5 },
      { x: 1, y: 1 }
    ];
    const second = toSvg(PLOT, { x: 0.53, y: 0.5 });
    expect(hitTest(PLOT, close, { x: second.x - 1, y: second.y }, 22)).toBe(2);
  });
});

describe('levels', () => {
  it.each([
    [0, 0],
    [0.5, 128],
    [1, 255]
  ])('%f is level %i', (value, level) => {
    expect(levelOf(value)).toBe(level);
  });

  it('clamps a typed level', () => {
    expect(fromLevel(300)).toBe(1);
    expect(fromLevel(-4)).toBe(0);
    expect(fromLevel(51)).toBeCloseTo(0.2, 9);
  });
});

describe('histLayers', () => {
  const hist: Histogram = { r: [1, 2], g: [2, 1], b: [0, 1], l: [1, 1] };

  it.each([
    ['composite', 3],
    ['r', 1],
    ['luma', 1]
  ] as const)('%s', (channel, count) => {
    const layers = histLayers(PLOT, hist, channel);
    expect(layers).toHaveLength(count);
    expect(layers.every((layer) => layer.d.startsWith('M ') && layer.d.endsWith(' Z'))).toBe(true);
  });

  it('is empty without a histogram', () => {
    expect(histLayers(PLOT, null, 'composite')).toEqual([]);
  });
});
