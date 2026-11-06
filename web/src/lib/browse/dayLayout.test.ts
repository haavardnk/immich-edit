import { describe, expect, it } from 'vitest';
import { createAssetGridLayout, verticalAssetIndex, visibleAssetRange } from './assetGridLayout';
import { createDayGridLayout, sectionTop, visibleHeaders } from './dayLayout';

const wide = { exifInfo: { exifImageWidth: 6000, exifImageHeight: 4000 } };
const options = { width: 600, rowHeight: 100, gap: 4, headerHeight: 40 };

describe('createDayGridLayout', () => {
  it('matches the flat layout without sections', () => {
    const assets = Array.from({ length: 7 }, () => wide);
    const flat = createAssetGridLayout(assets, 600, 100, 4);
    expect(createDayGridLayout(assets, null, options)).toEqual({ ...flat, headers: [] });
  });

  it('puts each section under its header in one index space', () => {
    const assets = Array.from({ length: 7 }, () => wide);
    const layout = createDayGridLayout(
      assets,
      [
        { start: 0, end: 5 },
        { start: 5, end: 7 }
      ],
      options
    );

    expect(layout.boxes).toHaveLength(7);
    expect(layout.headers.map(({ top, start, end }) => ({ top, start, end }))).toEqual([
      { top: 0, start: 0, end: 5 },
      { top: expect.any(Number), start: 5, end: 7 }
    ]);
    const [first, second] = layout.headers;
    expect(layout.boxes[0]?.top).toBe(40);
    const lastOfFirst = layout.boxes[4];
    const firstOfSecond = layout.boxes[5];
    if (!second || !lastOfFirst || !firstOfSecond || !first) throw new Error('missing geometry');
    expect(second.top).toBeCloseTo(lastOfFirst.top + lastOfFirst.height + 4);
    expect(firstOfSecond.top).toBeCloseTo(second.top + 40);
    expect(firstOfSecond.left).toBe(0);
    expect(layout.rows[0]?.startIndex).toBe(0);
    expect(layout.rows.at(-1)?.endIndex).toBe(7);
    layout.rows.forEach((row, index) => {
      const next = layout.rows[index + 1];
      if (next) expect(next.startIndex).toBe(row.endIndex);
    });
    layout.boxes.forEach((box, index) => {
      const row = layout.rows[box.row];
      expect(row && index >= row.startIndex && index < row.endIndex).toBe(true);
    });
    expect(layout.height).toBeCloseTo(firstOfSecond.top + firstOfSecond.height);
  });

  it('keeps virtualization and vertical moves working across days', () => {
    const assets = Array.from({ length: 12 }, () => wide);
    const layout = createDayGridLayout(
      assets,
      [
        { start: 0, end: 3 },
        { start: 3, end: 12 }
      ],
      options
    );
    const below = verticalAssetIndex(layout, 2, 1);
    expect(below).toBeGreaterThanOrEqual(3);
    expect(verticalAssetIndex(layout, 3, -1)).toBeLessThan(3);
    const second = layout.headers[1];
    if (!second) throw new Error('no header');
    const range = visibleAssetRange(layout, second.top, second.top + 50, 0);
    expect(range.startIndex).toBeLessThanOrEqual(3);
    expect(range.endIndex).toBeGreaterThan(3);
  });

  it('is empty before the grid has a width', () => {
    expect(createDayGridLayout([wide], [{ start: 0, end: 1 }], { ...options, width: 0 })).toEqual({
      boxes: [],
      rows: [],
      height: 0,
      headers: []
    });
  });
});

describe('visibleHeaders', () => {
  const headers = [
    { top: 0, height: 40, section: 0, start: 0, end: 4 },
    { top: 200, height: 40, section: 1, start: 4, end: 9 },
    { top: 500, height: 40, section: 2, start: 9, end: 12 }
  ];

  it('returns the headers whose days overlap the rendered range', () => {
    expect(visibleHeaders(headers, 5, 10).map((header) => header.section)).toEqual([1, 2]);
    expect(visibleHeaders(headers, 0, 4).map((header) => header.section)).toEqual([0]);
    expect(visibleHeaders(headers, 12, 12)).toEqual([]);
  });
});

describe('sectionTop', () => {
  it('scrolls a day start to its header and other tiles to their row', () => {
    const assets = Array.from({ length: 8 }, () => wide);
    const layout = createDayGridLayout(
      assets,
      [
        { start: 0, end: 4 },
        { start: 4, end: 8 }
      ],
      options
    );
    expect(sectionTop(layout, 4)).toBe(layout.headers[1]?.top);
    expect(sectionTop(layout, 5)).toBe(layout.boxes[5]?.top);
    expect(sectionTop(layout, 99)).toBeNull();
  });
});
