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
