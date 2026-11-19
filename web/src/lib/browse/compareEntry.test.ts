import { describe, it, expect } from 'vitest';
import { multiMembers } from './compareEntry';

const IDS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k'];

describe('multiMembers', () => {
  it.each([
    ['compare', new Set(['b', 'd', 'f']), ['b', 'd']],
    ['survey', new Set(['b', 'd', 'f']), ['b', 'd', 'f']],
    ['compare', new Set(IDS), IDS.slice(0, 2)],
    ['survey', new Set(IDS), IDS.slice(0, 9)]
  ] as const)('%s caps a selection of %o', (mode, selected, expected) => {
    expect(multiMembers(mode, IDS, selected, 'a')).toEqual(expected);
  });

  it.each([
    ['compare', ['c', 'd']],
    ['survey', ['c', 'd', 'e', 'f', 'g', 'h']]
  ] as const)('%s falls back to photos after the current one', (mode, expected) => {
    expect(multiMembers(mode, IDS, new Set(['c']), 'c')).toEqual(expected);
  });

  it('returns nothing without a selection or a current photo', () => {
    expect(multiMembers('compare', IDS, new Set(), null)).toEqual([]);
  });

  it('returns the last photo alone when nothing follows it', () => {
    expect(multiMembers('survey', IDS, new Set(), 'k')).toEqual(['k']);
  });
});
