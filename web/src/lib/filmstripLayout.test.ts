import { describe, expect, it } from 'vitest';
import { createFilmstripLayout, visibleFilmstripRange } from './filmstripLayout';

function asset(width: number, height: number, orientation: string | null = null) {
  return { exifInfo: { exifImageWidth: width, exifImageHeight: height, orientation } };
}

describe('createFilmstripLayout', () => {
  it('preserves landscape, portrait and rotated aspect ratios', () => {
    const layout = createFilmstripLayout(
      [asset(600, 400), asset(400, 600), asset(600, 400, '6')],
      60,
      4,
      8
    );

    expect(layout.boxes.map(({ width }) => width)).toEqual([90, 40, 40]);
    expect(layout.boxes.map(({ left }) => left)).toEqual([8, 102, 146]);
    expect(layout.width).toBe(194);
  });

  it('returns the visible slice with overscan', () => {
    const layout = createFilmstripLayout(
      [asset(600, 400), asset(400, 600), asset(400, 400), asset(800, 400)],
      60,
      4,
      8
    );

    expect(visibleFilmstripRange(layout.boxes, 110, 190, 1)).toEqual({
      startIndex: 0,
      endIndex: 4
    });
  });
});
