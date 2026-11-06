import { describe, expect, it } from 'vitest';
import type { AssetGridLayout } from './assetGridLayout';
import { marqueeHits, rectBetween, tileIndexAt } from './gridHit';

function grid(perRow: number, rowCount: number): AssetGridLayout {
  const size = 100;
  const gap = 4;
  const boxes = Array.from({ length: perRow * rowCount }, (_, index) => {
    const row = Math.floor(index / perRow);
    const column = index % perRow;
    return {
      top: row * (size + gap),
      left: column * (size + gap),
      width: size,
      height: size,
      row,
      column
    };
  });
  const rows = Array.from({ length: rowCount }, (_, row) => ({
    top: row * (size + gap),
    height: size,
    startIndex: row * perRow,
    endIndex: (row + 1) * perRow
  }));
  return { boxes, rows, height: rowCount * (size + gap) - gap };
}

describe('rectBetween', () => {
  it('normalises a drag in any direction', () => {
    expect(rectBetween({ x: 50, y: 10 }, { x: 5, y: 80 })).toEqual({
      left: 5,
      top: 10,
      right: 50,
      bottom: 80
    });
  });
});

describe('tileIndexAt', () => {
  const layout = grid(3, 3);

  it.each([
    ['inside the second tile of the second row', { x: 150, y: 150 }, 4],
    ['in the gap between two tiles', { x: 102, y: 150 }, -1],
    ['in the gap between rows', { x: 50, y: 102 }, -1],
    ['past the last column', { x: 400, y: 50 }, -1],
    ['below the grid', { x: 50, y: 900 }, -1]
  ])('finds %s', (_name, point, expected) => {
    expect(tileIndexAt(layout, point)).toBe(expected);
  });

  it('finds nothing in an empty layout', () => {
    expect(tileIndexAt({ boxes: [], rows: [], height: 0 }, { x: 0, y: 0 })).toBe(-1);
  });
});

describe('marqueeHits', () => {
  const layout = grid(3, 4);

  it.each([
    ['one tile', { left: 110, top: 110, right: 150, bottom: 150 }, [4]],
    ['a block across two rows', { left: 50, top: 50, right: 150, bottom: 150 }, [0, 1, 3, 4]],
    ['a gap only', { left: 101, top: 101, right: 103, bottom: 103 }, []],
    [
      'everything from a point above the grid',
      { left: -10, top: -50, right: 400, bottom: 900 },
      [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    ],
    ['the last row only', { left: 0, top: 320, right: 10, bottom: 330 }, [9]]
  ])('hits %s', (_name, rect, expected) => {
    expect(marqueeHits(layout, rect)).toEqual(expected);
  });
});
