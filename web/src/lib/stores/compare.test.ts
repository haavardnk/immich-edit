import { describe, it, expect, beforeEach } from 'vitest';
import { compare, CENTERED } from './compare.svelte';

describe('compare store', () => {
  beforeEach(() => compare.exit());

  it.each([
    [0, 1, 1],
    [2, 1, 0],
    [0, -1, 2],
    [1, 3, 1]
  ])('focus %i moved by %i wraps to %i', (start, delta, expected) => {
    compare.enter('survey', ['a', 'b', 'c'], start);
    compare.focusDelta(delta);
    expect(compare.focusIndex).toBe(expected);
  });

  it('drops a pane and clamps focus', () => {
    compare.enter('survey', ['a', 'b', 'c'], 2);
    compare.applyView('c', { zoomed: true, cx: 0.2, cy: 0.4 });
    compare.drop(2);
    expect(compare.members).toEqual(['a', 'b']);
    expect(compare.focusIndex).toBe(1);
    expect(compare.viewOf('c')).toEqual(CENTERED);
  });

  it('promotes a pane to the first slot', () => {
    compare.enter('compare', ['a', 'b'], 1);
    compare.promote(1);
    expect(compare.members).toEqual(['b', 'a']);
    expect(compare.focusIndex).toBe(0);
  });

  it('keeps only the chosen pane', () => {
    compare.enter('survey', ['a', 'b', 'c'], 1);
    compare.keepOnly(1);
    expect(compare.members).toEqual(['b']);
    expect(compare.focusIndex).toBe(0);
  });

  it.each([
    ['untouched', (): void => {}, false],
    ['dropped', (): void => compare.drop(1), true],
    ['kept one', (): void => compare.keepOnly(0), true],
    ['swapped', (): void => compare.setMember(1, 'd'), false]
  ])('a %s survey reports pruned %s', (_name, act, expected) => {
    compare.enter('survey', ['a', 'b', 'c']);
    act();
    expect(compare.pruned).toBe(expected);
  });

  it('a replacement member inherits the view of the photo it replaced', () => {
    compare.enter('compare', ['a', 'b']);
    compare.applyView('a', { zoomed: true, cx: 0.1, cy: 0.9 });
    compare.setMember(0, 'c');
    expect(compare.members).toEqual(['c', 'b']);
    expect(compare.viewOf('c')).toEqual({ zoomed: true, cx: 0.1, cy: 0.9 });
    expect(compare.viewOf('a')).toEqual(CENTERED);
  });

  it.each([
    [true, { zoomed: true, cx: 0.25, cy: 0.75 }],
    [false, CENTERED]
  ])('sync %s fans the view out to every pane', (sync, expected) => {
    compare.enter('compare', ['a', 'b']);
    compare.syncView = sync;
    compare.applyView('a', { zoomed: true, cx: 0.25, cy: 0.75 });
    expect(compare.viewOf('b')).toEqual(expected);
    compare.syncView = true;
  });

  it('exits back to single mode', () => {
    compare.enter('survey', ['a', 'b'], 1);
    compare.exit();
    expect(compare.mode).toBe('single');
    expect(compare.members).toEqual([]);
    expect(compare.focusedId).toBeNull();
  });
});
