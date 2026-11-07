import { describe, expect, it } from 'vitest';
import {
  paneColumns,
  paneGridStyle,
  paneWindow,
  sheetInsetStyle,
  switchMembers
} from './loupeLayout';

const IDS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k'];

describe('paneColumns', () => {
  it.each([
    [1, 2],
    [4, 2],
    [5, 3],
    [9, 3]
  ])('lays %i panes out in %i columns', (count, expected) => {
    expect(paneColumns(count)).toBe(expected);
  });

  it.each([
    [2, 1],
    [3, 2],
    [4, 2]
  ])('stacks %i panes in %i columns on a portrait phone', (count, expected) => {
    expect(paneColumns(count, true)).toBe(expected);
  });
});

describe('paneWindow', () => {
  it('shows every member while they fit the layout', () => {
    expect(paneWindow(6, 5, 9)).toEqual({ start: 0, end: 6 });
    expect(paneWindow(4, 0, 4)).toEqual({ start: 0, end: 4 });
  });

  it('slides a narrower window so it ends at a focused pane beyond it', () => {
    expect(paneWindow(6, 5, 4)).toEqual({ start: 2, end: 6 });
    expect(paneWindow(6, 4, 4)).toEqual({ start: 1, end: 5 });
  });

  it('keeps the window at the start while the focused pane is inside it', () => {
    expect(paneWindow(6, 0, 4)).toEqual({ start: 0, end: 4 });
    expect(paneWindow(6, 3, 4)).toEqual({ start: 0, end: 4 });
  });

  it('is empty without members', () => {
    expect(paneWindow(0, 0, 4)).toEqual({ start: 0, end: 0 });
  });
});

describe('sheetInsetStyle', () => {
  it('lifts the photo above the info sheet less whatever already sits below it', () => {
    expect(sheetInsetStyle(119.6)).toBe('padding-bottom: max(0px, calc(55dvh - 120px));');
    expect(sheetInsetStyle(0)).toBe('padding-bottom: max(0px, calc(55dvh - 0px));');
  });
});

describe('paneGridStyle', () => {
  const grid = (columns: number, rows: number): string =>
    `grid-template-columns: repeat(${columns}, minmax(0, 1fr)); grid-template-rows: repeat(${rows}, minmax(0, 1fr));`;

  it.each([
    [false, 3, false, ''],
    [true, 0, false, ''],
    [true, 1, false, grid(1, 1)],
    [true, 2, false, grid(2, 1)],
    [true, 3, false, grid(2, 2)],
    [true, 6, false, grid(3, 2)],
    [true, 9, false, grid(3, 3)],
    [true, 2, true, grid(1, 2)],
    [true, 4, true, grid(2, 2)]
  ])('multi=%s with %i panes, stacked=%s', (multi, count, stacked, expected) => {
    expect(paneGridStyle(multi, count, stacked)).toBe(expected);
  });
});

describe('switchMembers', () => {
  it('keeps the focused pane first when narrowing to compare', () => {
    expect(switchMembers('compare', IDS, 'd', ['b', 'c', 'd'])).toEqual(['d', 'b']);
  });

  it('rebuilds survey panes from the focused photo onwards', () => {
    expect(switchMembers('survey', IDS, 'b', ['b', 'e'])).toEqual(['b', 'c', 'd', 'e', 'f', 'g']);
  });

  it('returns nothing when fewer than two panes remain', () => {
    expect(switchMembers('compare', IDS, 'd', ['d'])).toEqual([]);
  });
});
