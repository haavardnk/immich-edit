import { beforeEach, describe, expect, it } from 'vitest';
import { selection } from './selection.svelte';

const ids = ['a', 'b', 'c', 'd', 'e'];

beforeEach(() => selection.clear());

describe('toggle', () => {
  it('adds, removes and anchors', () => {
    selection.toggle('b');
    expect([...selection.selected]).toEqual(['b']);
    expect(selection.count).toBe(1);
    selection.toggle('b');
    expect(selection.count).toBe(0);
    expect(selection.anchorId).toBe('b');
  });
});

describe('range', () => {
  it('extends from the anchor and keeps earlier picks', () => {
    selection.toggle('e');
    selection.toggle('b');
    selection.range(ids, 'd');
    expect([...selection.selected].sort()).toEqual(['b', 'c', 'd', 'e']);
  });

  it('re-runs from the same anchor without accumulating', () => {
    selection.toggle('b');
    selection.range(ids, 'd');
    selection.range(ids, 'c');
    expect([...selection.selected].sort()).toEqual(['b', 'c']);
  });

  it.each([
    ['no anchor', null],
    ['anchor outside the list', 'zz']
  ])('falls back to a single toggle with %s', (_name, anchor) => {
    selection.anchorId = anchor;
    selection.range(ids, 'c');
    expect([...selection.selected]).toEqual(['c']);
  });
});

describe('filtered selection', () => {
  it('reports every asset as selected and targets the query', () => {
    selection.selectFiltered({ query: 'cat' }, 42);
    expect(selection.allFiltered).toBe(true);
    expect(selection.has('anything')).toBe(true);
    expect(selection.targetCount).toBe(42);
    expect(selection.buildTarget()).toEqual({ search: { query: 'cat' } });
  });

  it.each([
    ['toggle', () => selection.toggle('a')],
    ['range', () => selection.range(ids, 'a')],
    ['selectLoaded', () => selection.selectLoaded(['a'])]
  ])('drops the filter on %s', (_name, act) => {
    selection.selectFiltered({ query: 'cat' }, 42);
    act();
    expect(selection.allFiltered).toBe(false);
    expect(selection.targetCount).toBe(1);
    expect(selection.buildTarget()).toEqual({ assetIds: ['a'] });
  });
});

describe('active', () => {
  it.each([
    ['empty', () => {}, false],
    ['one selected', () => selection.toggle('a'), true],
    ['filtered', () => selection.selectFiltered({ query: 'cat' }, 3), true]
  ])('is %s', (_name, act, expected) => {
    act();
    expect(selection.active).toBe(expected);
  });
});
