import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { paneWindow } from '#lib/browse/loupeLayout.ts';
import { FIT_VIEW } from '#lib/utils/imageViewport.ts';
import { COMPACT_PANES, compare, COMPARE_PANES, MAX_PANES } from './compare.svelte';

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
    compare.applyView('c', { zoom: 200, cx: 0.2, cy: 0.4 });
    compare.drop(2);
    expect(compare.members).toEqual(['a', 'b']);
    expect(compare.focusIndex).toBe(1);
    expect(compare.viewOf('c')).toEqual(FIT_VIEW);
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
    ['swapped', (): void => compare.setMember(1, 'd'), false],
    [
      'grown then dropped back',
      (): void => {
        compare.addMember('d');
        compare.drop(3);
      },
      true
    ]
  ])('a %s survey reports pruned %s', (_name, act, expected) => {
    compare.enter('survey', ['a', 'b', 'c']);
    act();
    expect(compare.pruned).toBe(expected);
  });

  it('adds a pane, focuses it and inherits the current view', () => {
    compare.enter('compare', ['a', 'b'], 1);
    compare.applyView('b', { zoom: 200, cx: 0.3, cy: 0.6 });
    compare.addMember('c');
    expect(compare.members).toEqual(['a', 'b', 'c']);
    expect(compare.focusIndex).toBe(2);
    expect(compare.mode).toBe('survey');
    expect(compare.viewOf('c')).toEqual({ zoom: 200, cx: 0.3, cy: 0.6 });
  });

  it.each([
    ['a photo already on screen', ['a', 'b', 'c'], 'c'],
    ['a tenth pane', ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'], 'j']
  ])('refuses to add %s', (_name, members, added) => {
    compare.enter('survey', members);
    compare.addMember(added);
    expect(compare.members).toEqual(members);
  });

  describe('under a layout pane limit', () => {
    let release = (): void => undefined;

    beforeEach(() => (release = compare.limitBy(() => 4)));
    afterEach(() => release());

    it('caps survey members on entry but never compare', () => {
      compare.enter('survey', ['a', 'b', 'c', 'd', 'e', 'f'], 5);
      expect(compare.members).toEqual(['a', 'b', 'c', 'd']);
      expect(compare.focusIndex).toBe(3);
      expect(compare.startCount).toBe(4);
      compare.enter('compare', ['a', 'b']);
      expect(compare.members).toEqual(['a', 'b']);
    });

    it('reports full and refuses a fifth pane', () => {
      compare.enter('survey', ['a', 'b', 'c']);
      expect(compare.full).toBe(false);
      compare.addMember('d');
      expect(compare.full).toBe(true);
      compare.addMember('e');
      expect(compare.members).toEqual(['a', 'b', 'c', 'd']);
    });

    it('never lifts the limit past nine panes and restores it on release', () => {
      const lift = compare.limitBy(() => 20);
      expect(compare.paneLimit).toBe(MAX_PANES);
      lift();
      expect(compare.paneLimit).toBe(MAX_PANES);
      release = compare.limitBy(() => 4);
      expect(compare.paneLimit).toBe(4);
    });

    it('keeps every member when the layout narrows mid-survey and shows the focused window', () => {
      let limit = MAX_PANES;
      release();
      release = compare.limitBy(() => limit);
      compare.enter('survey', ['a', 'b', 'c', 'd', 'e', 'f'], 5);
      expect(compare.full).toBe(false);

      limit = 4;
      expect(compare.members).toEqual(['a', 'b', 'c', 'd', 'e', 'f']);
      expect(compare.full).toBe(true);
      compare.addMember('g');
      expect(compare.members).toHaveLength(6);
      const shown = paneWindow(compare.members.length, compare.focusIndex, compare.paneLimit);
      expect(compare.members.slice(shown.start, shown.end)).toEqual(['c', 'd', 'e', 'f']);

      limit = MAX_PANES;
      expect(compare.full).toBe(false);
      compare.addMember('g');
      expect(compare.members).toHaveLength(7);
    });

    it.each([
      [COMPARE_PANES, false],
      [COMPACT_PANES, true],
      [MAX_PANES, true]
    ])('a %i-pane layout offers survey: %s', (limit, available) => {
      release();
      release = compare.limitBy(() => limit);
      expect(compare.surveyAvailable).toBe(available);
    });

    it('holds a phone compare at two panes and keeps a narrowed survey whole', () => {
      let limit = COMPARE_PANES;
      release();
      release = compare.limitBy(() => limit);
      compare.enter('compare', ['a', 'b']);
      expect(compare.full).toBe(true);
      compare.addMember('c');
      expect(compare.members).toEqual(['a', 'b']);
      expect(compare.mode).toBe('compare');

      limit = MAX_PANES;
      compare.enter('survey', ['a', 'b', 'c', 'd'], 3);
      limit = COMPARE_PANES;
      expect(compare.mode).toBe('survey');
      expect(compare.members).toEqual(['a', 'b', 'c', 'd']);
      const shown = paneWindow(compare.members.length, compare.focusIndex, compare.paneLimit);
      expect(compare.members.slice(shown.start, shown.end)).toEqual(['c', 'd']);
    });
  });

  it('a replacement member inherits the view of the photo it replaced', () => {
    compare.enter('compare', ['a', 'b']);
    compare.applyView('a', { zoom: 200, cx: 0.1, cy: 0.9 });
    compare.setMember(0, 'c');
    expect(compare.members).toEqual(['c', 'b']);
    expect(compare.viewOf('c')).toEqual({ zoom: 200, cx: 0.1, cy: 0.9 });
    expect(compare.viewOf('a')).toEqual(FIT_VIEW);
  });

  it.each([
    [true, { zoom: 200, cx: 0.25, cy: 0.75 }],
    [false, FIT_VIEW]
  ])('sync %s fans the view out to every pane', (sync, expected) => {
    compare.enter('compare', ['a', 'b']);
    compare.syncView = sync;
    compare.applyView('a', { zoom: 200, cx: 0.25, cy: 0.75 });
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
