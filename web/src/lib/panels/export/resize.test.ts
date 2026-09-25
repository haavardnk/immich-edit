import { describe, expect, it } from 'vitest';
import { linkedEdge, resizeError, resizedSize, type ExportResize } from './resize';

const crop = { w: 6000, h: 4000 };

function box(width: number | null, height: number | null, enlarge = false): ExportResize {
  return { mode: 'dimensions', width, height, enlarge };
}

function percent(value: number): ExportResize {
  return { mode: 'percent', percent: value };
}

function megapixels(value: number, enlarge = false): ExportResize {
  return { mode: 'megapixels', megapixels: value, enlarge };
}

describe('resizedSize', () => {
  it.each<[string, ExportResize | null, { w: number; h: number }]>([
    ['full size', null, { w: 6000, h: 4000 }],
    ['a square box', box(2048, 2048), { w: 2048, h: 1365 }],
    ['a box limited by height', box(2000, 1000), { w: 1500, h: 1000 }],
    ['width only', box(3000, null), { w: 3000, h: 2000 }],
    ['height only', box(null, 1080), { w: 1620, h: 1080 }],
    ['larger without enlarge', box(9000, 9000), { w: 6000, h: 4000 }],
    ['larger with enlarge', box(9000, 9000, true), { w: 9000, h: 6000 }],
    ['half', percent(50), { w: 3000, h: 2000 }],
    ['past 100 percent', percent(150), { w: 9000, h: 6000 }],
    ['megapixels', megapixels(6), { w: 3000, h: 2000 }],
    ['more megapixels without enlarge', megapixels(96), { w: 6000, h: 4000 }],
    ['more megapixels with enlarge', megapixels(96, true), { w: 12000, h: 8000 }],
    ['an invalid box', box(0, 1000), { w: 6000, h: 4000 }]
  ])('%s', (_name, value, size) => {
    expect(resizedSize(crop, value)).toEqual(size);
  });

  it('fits a portrait crop by its own width and height', () => {
    expect(resizedSize({ w: 4000, h: 6000 }, box(2000, 1000))).toEqual({ w: 667, h: 1000 });
  });

  it('comes out at exactly a linked width and height', () => {
    const odd = { w: 7952, h: 5304 };
    for (let width = 100; width < 4000; width += 7) {
      const height = linkedEdge(width, odd.w, odd.h);
      expect(resizedSize(odd, box(width, height))).toEqual({ w: width, h: height });
      expect(resizedSize(odd, box(null, height)).h).toBe(height);
    }
  });
});

describe('resizeError', () => {
  it.each<[ExportResize | null, boolean]>([
    [null, false],
    [box(2048, null), false],
    [box(null, null), true],
    [box(0, 100), true],
    [box(70000, null), true],
    [box(12.5, null), true],
    [percent(400), false],
    [percent(0.5), true],
    [percent(401), true],
    [percent(Number.NaN), true],
    [megapixels(0.5), false],
    [megapixels(0), true],
    [megapixels(1001), true]
  ])('%o is invalid: %s', (value, invalid) => {
    expect(resizeError(value) !== null).toBe(invalid);
  });
});
