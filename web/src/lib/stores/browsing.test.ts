import { describe, it, expect, beforeEach, vi } from 'vitest';
import { browsing, type BrowsePager } from './browsing.svelte';
import { browseControls } from './browseControls.svelte';
import type { AssetSummary } from '#lib/types/album.ts';
import type { ExifInfo } from '#lib/types/asset.ts';

const A = '11111111-1111-1111-1111-111111111111';
const B = '22222222-2222-2222-2222-222222222222';

function summary(id: string): AssetSummary {
  return { id, originalFileName: `${id}.arw` } as AssetSummary;
}

describe('browsing store', () => {
  let feed: AssetSummary[];

  beforeEach(() => {
    browsing.clear();
    feed = [summary(A), summary(B)];
    browsing.set(feed);
  });

  it('inserts a copy after its master without replacing the array', () => {
    browsing.insertCopy(`${A}_1`, A, 'Mono');
    expect(browsing.assets).toBe(feed);
    expect(feed.map((a) => a.id)).toEqual([A, `${A}_1`, B]);
    expect(feed[1]?.copyOf).toBe(A);
    expect(feed[1]?.copyLabel).toBe('Mono');
  });

  it('inserts later copies after existing ones', () => {
    browsing.insertCopy(`${A}_1`, A, null);
    browsing.insertCopy(`${A}_2`, A, null);
    expect(feed.map((a) => a.id)).toEqual([A, `${A}_1`, `${A}_2`, B]);
  });

  it('removes many in place and puts them back where they were', () => {
    browsing.total = 2;
    browsing.insertCopy(`${A}_1`, A, null);
    const removed = browsing.removeMany(new Set([A, `${A}_1`]));
    expect(browsing.assets).toBe(feed);
    expect(feed.map((a) => a.id)).toEqual([B]);
    expect(browsing.total).toBe(0);
    browsing.reinsert(removed);
    expect(browsing.assets).toBe(feed);
    expect(feed.map((a) => a.id)).toEqual([A, `${A}_1`, B]);
    expect(browsing.total).toBe(2);
  });

  it('does not put removed photos into a list loaded since', () => {
    const removed = browsing.removeMany(new Set([A]));
    browsing.set([summary(B)]);
    browsing.reinsert(removed);
    expect(browsing.assets.map((a) => a.id)).toEqual([B]);
  });

  it('removes a copy without replacing the array', () => {
    browsing.insertCopy(`${A}_1`, A, null);
    browsing.remove(`${A}_1`);
    expect(browsing.assets).toBe(feed);
    expect(feed.map((a) => a.id)).toEqual([A, B]);
  });

  it('patches the listed photo in place', () => {
    const photo = feed[0];
    browsing.patch(A, { isFavorite: true });
    expect(browsing.assets[0]).toBe(photo);
    expect(photo?.isFavorite).toBe(true);
  });

  it('finds photos by id as the list changes', () => {
    browsing.insertCopy(`${A}_1`, A, null);
    expect(browsing.find(`${A}_1`)?.copyOf).toBe(A);
    browsing.remove(B);
    expect(browsing.find(B)).toBeNull();
  });

  it('steps, places and counts only the photos the filter shows', () => {
    const shot = (id: string, make: string): AssetSummary => ({
      ...summary(id),
      exifInfo: { make } as ExifInfo
    });
    browseControls.reset();
    browseControls.filters.make = ['SONY'];
    browsing.set([shot('a', 'SONY'), shot('b', 'Canon'), shot('c', 'SONY'), shot('d', 'Canon')]);
    browsing.total = 10;

    expect(browsing.visible.map((a) => a.id)).toEqual(['a', 'c']);
    expect([browsing.nextOf('a')?.id, browsing.prevOf('c')?.id]).toEqual(['c', 'a']);
    expect([browsing.nextOf('b')?.id, browsing.nextOf('c'), browsing.prevOf('a')]).toEqual([
      'c',
      null,
      null
    ]);
    expect(browsing.count).toBe(8);
    browseControls.reset();
  });

  it('reveals a photo through the pager without a failure toast', async () => {
    const jumpTo = vi.fn(() => Promise.resolve(false));
    const pager: BrowsePager = {
      hasMore: false,
      hasPrevious: false,
      loadingMore: false,
      loadMore: () => Promise.resolve(false),
      loadPrevious: () => Promise.resolve(false),
      loadEdge: () => Promise.resolve(false),
      jumpTo
    };
    browsing.set(feed, pager);
    expect(await browsing.requestReveal(B)).toBe(false);
    expect(jumpTo).toHaveBeenCalledWith(B, { failure: null });
  });
});
