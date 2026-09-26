import { describe, expect, it } from 'vitest';
import type { PreviewMeta } from '$lib/types/preview';
import { formatBrushSize, stepBrush, wheelHardness, wheelNotches, wheelSize } from './brushSize';

const SIZE = { step: 0.01, min: 0.005, max: 0.5 };
const HARD = { step: 0.1, min: 0, max: 1 };

function meta(w: number, h: number): PreviewMeta {
  return {
    asset_id: 'a',
    width: w,
    height: h,
    source_w: w,
    source_h: h,
    renderer: 'cpu',
    is_raw: true,
    histogram: { r: [], g: [], b: [], l: [] },
    has_scopes: false
  };
}

describe('formatBrushSize', () => {
  it.each<[number, PreviewMeta | null, string]>([
    [0.05, meta(6000, 4000), '200 px'],
    [0.05, meta(4000, 6000), '200 px'],
    [0.05, null, '0.050'],
    [0.05, meta(1, 1), '0.050']
  ])('shows %s against the short edge', (size, m, expected) => {
    expect(formatBrushSize(size, m)).toBe(expected);
  });
});

describe('stepBrush', () => {
  it.each([
    ['[', 0.2, 0.19],
    ['{', 0.2, 0.19],
    [']', 0.2, 0.21],
    ['}', 0.2, 0.21]
  ])('%s steps %f to %f', (key, current, expected) => {
    expect(stepBrush(current, key, SIZE)).toBeCloseTo(expected, 6);
  });

  it.each([
    ['[', 0.005, 0.005],
    [']', 0.5, 0.5]
  ])('%s clamps at the bound', (key, current, expected) => {
    expect(stepBrush(current, key, SIZE)).toBe(expected);
  });
});

describe('wheelNotches', () => {
  it.each<[string, { deltaX: number; deltaY: number; deltaMode: number }, number]>([
    ['scrolling up grows', { deltaX: 0, deltaY: -100, deltaMode: 0 }, 1],
    ['scrolling down shrinks', { deltaX: 0, deltaY: 100, deltaMode: 0 }, -1],
    ['a trackpad nudge is a fraction', { deltaX: 0, deltaY: -10, deltaMode: 0 }, 0.1],
    ['line mode scales to pixels', { deltaX: 0, deltaY: -3, deltaMode: 1 }, 0.48],
    ['shift turns the wheel sideways', { deltaX: -100, deltaY: 0, deltaMode: 0 }, 1]
  ])('%s', (_name, event, expected) => {
    expect(wheelNotches(event)).toBeCloseTo(expected, 6);
  });
});

describe('wheel brush steps', () => {
  it('scales size by ten percent a notch', () => {
    expect(wheelSize(0.2, 1, SIZE)).toBeCloseTo(0.22, 6);
    expect(wheelSize(0.2, -1, SIZE)).toBeCloseTo(0.2 / 1.1, 6);
  });

  it('clamps size at both ends', () => {
    expect(wheelSize(0.49, 5, SIZE)).toBe(0.5);
    expect(wheelSize(0.006, -5, SIZE)).toBe(0.005);
  });

  it('moves hardness one step a notch and clamps', () => {
    expect(wheelHardness(0.5, 1, HARD)).toBeCloseTo(0.6, 6);
    expect(wheelHardness(0.95, 1, HARD)).toBe(1);
    expect(wheelHardness(0.05, -1, HARD)).toBe(0);
  });
});
