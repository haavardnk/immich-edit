import { describe, it, expect, beforeEach } from 'vitest';
import { browsing } from './browsing.svelte';
import { browseControls } from './browseControls.svelte';
import type { AssetSummary } from '$lib/types/album';
import type { ExifInfo } from '$lib/types/asset';

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

  it('removes a copy without replacing the array', () => {
    browsing.insertCopy(`${A}_1`, A, null);
    browsing.remove(`${A}_1`);
    expect(browsing.assets).toBe(feed);
    expect(feed.map((a) => a.id)).toEqual([A, B]);
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
    expect([browsing.positionOf('c'), browsing.count]).toEqual([2, 8]);
    browseControls.reset();
  });
});
