import { describe, expect, it } from 'vitest';
import {
  areaSample,
  colorWindow,
  displayLuma,
  lumaWindow,
  pointSample,
  type RangeSample
} from './rangeSample';
import type { PixelGrid } from './targetPick';

function grid(pixels: Array<[number, number, number]>): PixelGrid {
  const data = new Uint8ClampedArray(pixels.length * 4);
  pixels.forEach(([r, g, b], i) => data.set([r, g, b, 255], i * 4));
  return { width: pixels.length, height: 1, data };
}

function grey(level: number, count: number): Array<[number, number, number]> {
  return Array.from({ length: count }, () => [level, level, level]);
}

describe('pointSample', () => {
  it('averages the patch and has no spread', () => {
    const sample = pointSample(grid(grey(128, 25)));
    expect(sample.lo).toBeCloseTo(128 / 255, 3);
    expect(sample.hi).toBe(sample.lo);
    expect(sample.spread).toBe(0);
    expect(sample.rgb[0]).toBeCloseTo(128 / 255, 3);
  });
});

describe('areaSample', () => {
  it('takes the inner luminance percentiles of the area', () => {
    const levels = Array.from(
      { length: 101 },
      (_, i) => [i * 2, i * 2, i * 2] as [number, number, number]
    );
    const sample = areaSample(grid(levels));
    expect(sample.lo).toBeCloseTo(10 / 255, 2);
    expect(sample.hi).toBeCloseTo(190 / 255, 2);
  });

  it('measures how far the colours spread from their mean', () => {
    const flat = areaSample(grid(grey(100, 20)));
    const mixed = areaSample(grid([...grey(40, 10), ...grey(200, 10)]));
    expect(flat.spread).toBeCloseTo(0, 5);
    expect(mixed.spread).toBeGreaterThan(0.1);
  });
});

describe('windows', () => {
  const dark = pointSample(grid(grey(40, 4)));
  const light = pointSample(grid(grey(200, 4)));

  it('covers every luminance sample, snapped outward to the slider step', () => {
    const window = lumaWindow([dark, light]);
    expect(window?.min).toBeLessThanOrEqual(displayLuma(dark.rgb));
    expect(window?.max).toBeGreaterThanOrEqual(displayLuma(light.rgb));
    expect(window?.min).toBe(0.15);
    expect(window?.max).toBe(0.79);
  });

  it('keeps one colour sample and the tolerance', () => {
    const window = colorWindow([dark], 0.1);
    expect(window?.tolerance).toBe(0.1);
    expect(window?.sample_rgb[0]).toBeCloseTo(40 / 255, 3);
  });

  it('centres several colour samples and widens the tolerance to reach them', () => {
    const window = colorWindow([dark, light], 0.1);
    expect(window?.sample_rgb[0]).toBeGreaterThan(40 / 255);
    expect(window?.sample_rgb[0]).toBeLessThan(200 / 255);
    expect(window?.tolerance).toBeGreaterThan(0.1);
  });

  it('has no window without samples', () => {
    const none: RangeSample[] = [];
    expect(lumaWindow(none)).toBeNull();
    expect(colorWindow(none, 0.1)).toBeNull();
  });
});
