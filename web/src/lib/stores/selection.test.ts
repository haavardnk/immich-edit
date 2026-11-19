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

  it('previews the range without committing it', () => {
    selection.toggle('b');
    expect([...selection.rangeTarget(ids, 'd')]).toEqual(['b', 'c', 'd']);
    expect([...selection.selected]).toEqual(['b']);
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

describe('active', () => {
  it.each([
    ['empty', () => {}, false],
    ['one selected', () => selection.toggle('a'), true]
  ])('is %s', (_name, act, expected) => {
    act();
    expect(selection.active).toBe(expected);
  });
});

describe('forget', () => {
  it('drops removed ids and their anchor', () => {
    selection.replace(['a', 'b', 'c']);
    selection.forget(new Set(['c']));
    expect([...selection.selected]).toEqual(['a', 'b']);
    expect(selection.anchorId).toBeNull();
  });
});

describe('setMany', () => {
  it('adds a block and anchors on its last photo', () => {
    selection.toggle('a');
    selection.setMany(['c', 'd'], true);
    expect([...selection.selected].sort()).toEqual(['a', 'c', 'd']);
    expect(selection.anchorId).toBe('d');
    selection.range(ids, 'e');
    expect([...selection.selected].sort()).toEqual(['a', 'c', 'd', 'e']);
  });

  it('removes a block and keeps the rest', () => {
    selection.replace(['a', 'b', 'c']);
    selection.setMany(['b', 'c'], false);
    expect([...selection.selected]).toEqual(['a']);
  });
});

describe('select mode', () => {
  it('stays on with nothing picked until the selection clears', () => {
    selection.enterMode();
    expect(selection.active).toBe(false);
    expect(selection.selecting).toBe(true);
    selection.toggle('a');
    selection.toggle('a');
    expect(selection.selecting).toBe(true);
    selection.clear();
    expect(selection.mode).toBe(false);
    expect(selection.selecting).toBe(false);
  });

  it('leaves select mode without dropping the picked photos', () => {
    selection.enterMode();
    selection.toggle('a');
    selection.exitMode();
    expect(selection.mode).toBe(false);
    expect([...selection.selected]).toEqual(['a']);
  });
});

describe('anchorAt', () => {
  it('ranges from an already picked photo without dropping it', () => {
    selection.replace(['b']);
    selection.anchorAt('b');
    selection.range(ids, 'd');
    expect([...selection.selected].sort()).toEqual(['b', 'c', 'd']);
  });
});

describe('replace', () => {
  it('swaps the whole set and ranges from the new base', () => {
    selection.replace(['a']);
    selection.replace(['c', 'd'], 'd');
    expect([...selection.selected]).toEqual(['c', 'd']);
    selection.range(ids, 'e');
    expect([...selection.selected].sort()).toEqual(['c', 'd', 'e']);
  });

  it('anchors on the last photo unless told otherwise', () => {
    selection.replace(['b', 'd']);
    expect(selection.anchorId).toBe('d');
    selection.replace(['c'], null);
    expect(selection.anchorId).toBeNull();
  });
});
