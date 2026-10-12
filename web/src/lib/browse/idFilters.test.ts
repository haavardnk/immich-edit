import { describe, expect, it } from 'vitest';
import { idFilter, idSet, type IdMatch } from './idFilters';

describe('id filters', () => {
  it('forgets the match once no ids are left', () => {
    expect(idFilter([], 'none')).toEqual({ ids: [], match: 'all' });
    expect(idFilter(['p1'], 'none')).toEqual({ ids: ['p1'], match: 'none' });
  });

  it.each([
    ['any', ['p1', 'p2'], true, { any: ['p1', 'p2'] }],
    ['none', ['p1'], true, { none: ['p1'] }],
    ['all', ['p1', 'p2'], false, { all: ['p1', 'p2'] }],
    ['any', ['p1'], false, { all: ['p1'] }],
    ['any', ['p1', 'p2'], false, undefined],
    ['none', ['p1'], false, undefined],
    ['all', [], true, undefined]
  ] as [IdMatch, string[], boolean, unknown][])(
    'sends %s of %j with structured search %s as %j',
    (match, ids, structured, expected) => {
      expect(idSet({ ids, match }, structured)).toEqual(expected);
    }
  );
});
