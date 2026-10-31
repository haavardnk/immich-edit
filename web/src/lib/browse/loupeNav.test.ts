import { beforeEach, describe, expect, it } from 'vitest';
import type { AssetSummary } from '#lib/types/album.ts';
import { browsing, type BrowsePager } from '#lib/stores/browsing.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import {
  advanceFocused,
  enterMulti,
  leaveMulti,
  openLastLoupe,
  pickFromStrip,
  selectViewMode,
  stepLoupe
} from './loupeNav';

function summary(id: string): AssetSummary {
  return { id, originalFileName: `${id}.arw` } as AssetSummary;
}

function load(ids: string[], pages: string[][] = []): void {
  const pending = [...pages];
  const pager: BrowsePager = {
    get hasMore() {
      return pending.length > 0;
    },
    loadingMore: false,
    loadMore: () => {
      const page = pending.shift();
      if (!page) return Promise.resolve(false);
      browsing.assets = [...browsing.assets, ...page.map(summary)];
      return Promise.resolve(true);
    }
  };
  browsing.set(ids.map(summary), pager);
}

function deferPage(ids: string[], more: string[]): () => Promise<void> {
  let resolvePage = (_loaded: boolean): void => undefined;
  const page = new Promise<boolean>((resolve) => (resolvePage = resolve));
  browsing.set(ids.map(summary), { hasMore: true, loadingMore: false, loadMore: () => page });
  return async () => {
    browsing.assets = [...browsing.assets, ...more.map(summary)];
    resolvePage(true);
    await page;
    await Promise.resolve();
  };
}

describe('loupe navigation', () => {
  beforeEach(() => {
    browseView.closeLoupe();
    selection.clear();
    toasts.items = [];
    load(['a', 'b', 'c', 'd']);
  });

  it.each([
    ['b', 1, 'c'],
    ['b', -1, 'a'],
    ['a', -1, 'a']
  ])('steps from %s by %i to %s', (from, delta, expected) => {
    browseView.openLoupe(from);
    stepLoupe(delta);
    expect(browseView.loupeId).toBe(expected);
  });

  it('loads the next page when stepping past the end', async () => {
    load(['a', 'b'], [['c']]);
    browseView.openLoupe('b');
    stepLoupe(1);
    await Promise.resolve();
    await Promise.resolve();
    expect(browseView.loupeId).toBe('c');
  });

  it('keeps every step pressed while the next page loads', async () => {
    const arrive = deferPage(['a', 'b'], ['c', 'd', 'e']);
    browseView.openLoupe('b');
    stepLoupe(1);
    stepLoupe(1);
    await arrive();
    expect(browseView.loupeId).toBe('d');
  });

  it('forgets steps past the end once the user steps back', async () => {
    const arrive = deferPage(['a', 'b'], ['c', 'd', 'e']);
    browseView.openLoupe('b');
    stepLoupe(1);
    stepLoupe(-1);
    stepLoupe(1);
    stepLoupe(1);
    await arrive();
    expect(browseView.loupeId).toBe('c');
  });

  it('opens the last asset after loading every page', async () => {
    load(['a'], [['b'], ['c']]);
    browseView.openLoupe('a');
    await openLastLoupe();
    expect(browseView.loupeId).toBe('c');
  });

  it('swaps the focused pane for the next non-member', () => {
    compare.enter('compare', ['a', 'b'], 0);
    advanceFocused(1);
    expect(compare.members).toEqual(['c', 'b']);
  });

  it.each([
    ['compare', ['b', 'c']],
    ['survey', ['b', 'c', 'd']]
  ] as const)('enters %s from the open asset', (mode, members) => {
    browseView.openLoupe('b');
    enterMulti(mode);
    expect(compare.mode).toBe(mode);
    expect(compare.members).toEqual(members);
  });

  it('refuses to compare a lone photo', () => {
    load(['a']);
    browseView.openLoupe('a');
    enterMulti('compare');
    expect(compare.mode).toBe('single');
    expect(toasts.items.map((t) => t.message)).toEqual(['compare needs two photos']);
  });

  it('selects the survivors of a pruned survey on leave', () => {
    browseView.openLoupe('a');
    enterMulti('survey');
    compare.drop(0);
    leaveMulti();
    expect(compare.mode).toBe('single');
    expect(browseView.loupeId).toBe('b');
    expect([...selection.selected]).toEqual(['b', 'c', 'd']);
  });

  it('switches between single and compare views', () => {
    browseView.openLoupe('a');
    selectViewMode('compare');
    expect(compare.members).toEqual(['a', 'b']);
    compare.focusIndex = 1;
    selectViewMode('single');
    expect(compare.mode).toBe('single');
    expect(browseView.loupeId).toBe('b');
  });

  it.each([
    [false, 'single', ['c']],
    [true, 'compare', ['a', 'c']]
  ] as const)('picks from the strip additive=%s', (additive, mode, members) => {
    browseView.openLoupe('a');
    pickFromStrip('c', additive);
    expect(compare.mode).toBe(mode);
    expect(mode === 'single' ? [browseView.loupeId] : compare.members).toEqual(members);
  });

  it('removes a compare member from the strip back to single', () => {
    browseView.openLoupe('a');
    enterMulti('compare');
    pickFromStrip('a', true);
    expect(compare.mode).toBe('single');
    expect(browseView.loupeId).toBe('b');
  });
});
