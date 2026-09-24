import { describe, expect, it } from 'vitest';
import type { PreviewMeta } from '$lib/types/preview';
import { formatBrushSize } from './brushSize';

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
