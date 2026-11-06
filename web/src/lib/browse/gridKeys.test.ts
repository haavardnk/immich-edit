import { describe, expect, it } from 'vitest';
import type { AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import { moveIndex, pageRows, type MoveContext } from './gridKeys';

function grid(rows: number, columns: number): AssetGridLayout {
  const boxes = Array.from({ length: rows * columns }, (_, index) => {
    const row = Math.floor(index / columns);
    const column = index % columns;
    return { top: row * 100, left: column * 100, width: 96, height: 96, row, column };
  });
  return {
    boxes,
    rows: Array.from({ length: rows }, (_, row) => ({
      top: row * 100,
      height: 96,
      startIndex: row * columns,
      endIndex: (row + 1) * columns
    })),
    height: rows * 100
  };
}

function context(cursor: number, extra: Partial<MoveContext> = {}): MoveContext {
  return {
    layout: grid(3, 3),
    count: 9,
    cursor,
    selecting: true,
    pageRows: 2,
    hasPrevious: false,
    hasMore: false,
    ...extra
  };
}

describe('moveIndex', () => {
  it('steps sideways, vertically and by pages inside the loaded list', () => {
    expect(moveIndex('ArrowRight', context(4))).toEqual({ kind: 'index', index: 5 });
    expect(moveIndex('ArrowLeft', context(4))).toEqual({ kind: 'index', index: 3 });
    expect(moveIndex('ArrowDown', context(4))).toEqual({ kind: 'index', index: 7 });
    expect(moveIndex('ArrowUp', context(4))).toEqual({ kind: 'index', index: 1 });
    expect(moveIndex('PageDown', context(1))).toEqual({ kind: 'index', index: 7 });
  });

  it('first selects the remembered photo before moving', () => {
    expect(moveIndex('ArrowRight', context(4, { selecting: false }))).toEqual({
      kind: 'index',
      index: 4
    });
    expect(moveIndex('ArrowDown', context(-1))).toEqual({ kind: 'index', index: 0 });
  });

  it('stops at the ends of a complete list', () => {
    expect(moveIndex('ArrowLeft', context(0))).toEqual({ kind: 'index', index: 0 });
    expect(moveIndex('ArrowRight', context(8))).toEqual({ kind: 'index', index: 8 });
    expect(moveIndex('ArrowUp', context(1))).toEqual({ kind: 'index', index: 1 });
    expect(moveIndex('Home', context(5))).toEqual({ kind: 'index', index: 0 });
    expect(moveIndex('End', context(5))).toEqual({ kind: 'index', index: 8 });
  });

  it('asks for more photos when a step leaves a loaded end that continues', () => {
    const open = { hasPrevious: true, hasMore: true };
    expect(moveIndex('ArrowLeft', context(0, open))).toEqual({ kind: 'beyond', end: 'first' });
    expect(moveIndex('ArrowUp', context(2, open))).toEqual({ kind: 'beyond', end: 'first' });
    expect(moveIndex('ArrowRight', context(8, open))).toEqual({ kind: 'beyond', end: 'last' });
    expect(moveIndex('PageDown', context(4, open))).toEqual({ kind: 'beyond', end: 'last' });
  });

  it('sends Home and End to the true ends of an open list', () => {
    const open = { hasPrevious: true, hasMore: true };
    expect(moveIndex('Home', context(4, open))).toEqual({ kind: 'edge', end: 'first' });
    expect(moveIndex('End', context(4, open))).toEqual({ kind: 'edge', end: 'last' });
  });
});

describe('pageRows', () => {
  it('fits whole rows into the view and never less than one', () => {
    expect(pageRows(500, 150)).toBe(3);
    expect(pageRows(100, 150)).toBe(1);
    expect(pageRows(500, 0)).toBe(500);
  });
});
