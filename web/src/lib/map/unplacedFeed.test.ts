import { UnplacedFeed, unplacedFilter } from '#lib/map/unplacedFeed.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import { describe, expect, it, vi } from 'vitest';

vi.mock('#lib/stores/browsing.svelte.ts', () => ({
  browsing: { isFeed: () => false, set: vi.fn(), total: undefined }
}));

describe('unplaced feed', () => {
  it('asks Immich for photos without a country inside the map filters', () => {
    expect(unplacedFilter({ favorite: true, after: '2024-01-01' })).toEqual({
      visibility: 'timeline',
      country: [null],
      favorite: true,
      takenAfter: '2024-01-01T00:00:00Z',
      takenBefore: undefined
    });
  });

  it('drops photos that have coordinates and pages on', async () => {
    const located = {
      ...asset('a'),
      exifInfo: { ...asset('a').exifInfo, latitude: 60, longitude: 5 }
    };
    const search = vi
      .fn()
      .mockResolvedValueOnce({ items: [located, asset('b')], nextCursor: '2' })
      .mockResolvedValueOnce({ items: [asset('c')], nextCursor: null });
    const feed = new UnplacedFeed(search as never);
    await feed.show({});
    expect(feed.assets.map((item) => item.id)).toEqual(['b']);
    expect(feed.hasMore).toBe(true);
    await feed.loadMore();
    expect(feed.assets.map((item) => item.id)).toEqual(['b', 'c']);
    expect(feed.hasMore).toBe(false);
    feed.remove(['b']);
    expect(feed.assets.map((item) => item.id)).toEqual(['c']);
  });
});
