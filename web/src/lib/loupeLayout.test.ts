import { describe, it, expect } from 'vitest';
import { paneColumns, paneGridStyle, switchMembers } from './loupeLayout';

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
});

describe('paneGridStyle', () => {
  it.each([
    [false, 3, ''],
    [true, 0, ''],
    [true, 1, 'grid-template-columns: repeat(1, minmax(0, 1fr));'],
    [true, 3, 'grid-template-columns: repeat(2, minmax(0, 1fr));'],
    [true, 6, 'grid-template-columns: repeat(3, minmax(0, 1fr));']
  ])('multi=%s with %i panes', (multi, count, expected) => {
    expect(paneGridStyle(multi, count)).toBe(expected);
  });
});

describe('switchMembers', () => {
  it('keeps the focused pane first when narrowing to compare', () => {
    expect(switchMembers('compare', IDS, new Set(), 'd', ['b', 'c', 'd'])).toEqual(['d', 'b']);
  });

  it('rebuilds survey panes from the selection', () => {
    expect(switchMembers('survey', IDS, new Set(['b', 'e', 'g']), 'b', ['b', 'e'])).toEqual([
      'b',
      'e',
      'g'
    ]);
  });

  it('returns nothing when fewer than two panes remain', () => {
    expect(switchMembers('compare', IDS, new Set(), 'd', ['d'])).toEqual([]);
  });
});
