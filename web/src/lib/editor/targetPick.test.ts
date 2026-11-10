import { describe, expect, it } from 'vitest';
import {
  averageAt,
  bwChannelAt,
  gradeRegionAt,
  hslBandAt,
  toneAt,
  type PixelGrid,
  type Rgb
} from './targetPick';

describe('hslBandAt', () => {
  it.each<[string, Rgb, number]>([
    ['red', [0.9, 0.1, 0.1], 0],
    ['orange', [0.95, 0.55, 0.1], 1],
    ['yellow', [0.9, 0.9, 0.1], 2],
    ['green', [0.1, 0.8, 0.2], 3],
    ['aqua', [0.1, 0.8, 0.8], 4],
    ['blue', [0.1, 0.2, 0.9], 5],
    ['purple', [0.5, 0.1, 0.9], 6],
    ['magenta', [0.9, 0.1, 0.9], 7]
  ])('names %s', (_name, rgb, band) => {
    expect(hslBandAt(rgb)).toBe(band);
  });

  it('names no band for a neutral or barely tinted pixel', () => {
    expect(hslBandAt([0.5, 0.5, 0.5])).toBeNull();
    expect(hslBandAt([0.5, 0.505, 0.5])).toBeNull();
  });
});

describe('bwChannelAt', () => {
  it.each<[Rgb, string]>([
    [[0.9, 0.1, 0.1], 'red'],
    [[0.9, 0.9, 0.1], 'yellow'],
    [[0.1, 0.8, 0.2], 'green'],
    [[0.1, 0.8, 0.8], 'aqua'],
    [[0.1, 0.2, 0.9], 'blue'],
    [[0.9, 0.1, 0.9], 'magenta']
  ])('maps %j to %s', (rgb, channel) => {
    expect(bwChannelAt(rgb)).toBe(channel);
  });

  it('names no channel for grey', () => {
    expect(bwChannelAt([0.4, 0.4, 0.4])).toBeNull();
  });
});

describe('gradeRegionAt', () => {
  it('splits by lightness at the neutral balance and blend', () => {
    expect(gradeRegionAt([0.05, 0.05, 0.05], 0, 50)).toBe('shadows');
    expect(gradeRegionAt([0.45, 0.45, 0.45], 0, 50)).toBe('midtones');
    expect(gradeRegionAt([0.95, 0.95, 0.95], 0, 50)).toBe('highlights');
  });

  it('moves the split with balance', () => {
    const grey: Rgb = [0.55, 0.55, 0.55];
    expect(gradeRegionAt(grey, -100, 50)).toBe('highlights');
    expect(gradeRegionAt(grey, 100, 50)).toBe('shadows');
  });
});

describe('toneAt', () => {
  it('reads the channel or the luma', () => {
    const rgb: Rgb = [0.2, 0.4, 0.8];
    expect(toneAt(rgb, 'r')).toBe(0.2);
    expect(toneAt(rgb, 'g')).toBe(0.4);
    expect(toneAt(rgb, 'b')).toBe(0.8);
    expect(toneAt(rgb, 'composite')).toBeCloseTo(0.2126 * 0.2 + 0.7152 * 0.4 + 0.0722 * 0.8);
    expect(toneAt(rgb, 'luma')).toBe(toneAt(rgb, 'composite'));
  });
});

describe('averageAt', () => {
  function grid(width: number, height: number, at: (x: number) => number): PixelGrid {
    const data = new Uint8ClampedArray(width * height * 4);
    for (let i = 0; i < width * height; i++) {
      const value = at(i % width);
      data.set([value, value, value, 255], i * 4);
    }
    return { width, height, data };
  }

  it('averages the window around the point', () => {
    const half = grid(10, 10, (x) => (x < 5 ? 0 : 255));
    expect(averageAt(half, 0.05, 0.5)[0]).toBe(0);
    expect(averageAt(half, 0.95, 0.5)[0]).toBe(1);
    expect(averageAt(half, 0.5, 0.5)[0]).toBeCloseTo(3 / 5);
  });

  it('clamps the window at the edges', () => {
    expect(
      averageAt(
        grid(2, 2, () => 51),
        1,
        1
      )
    ).toEqual([0.2, 0.2, 0.2]);
  });
});
