import { describe, expect, it } from 'vitest';
import {
  FEED_CACHE_ENTRIES,
  FEED_CACHE_MAX_AGE_MS,
  FEED_CACHE_MAX_ITEMS,
  FeedCache,
  type FeedSnapshot
} from './feedCache';
import type { AssetSummary } from '#lib/types/album.ts';

function snapshot(count = 1): FeedSnapshot {
  return {
    assets: Array.from({ length: count }, (_, index) => ({ id: `a${index}` }) as AssetSummary),
    nextCursor: 'next',
    totalCount: count,
    windowed: false,
    before: false,
    after: false
  };
}

describe('FeedCache', () => {
  it('hands a stashed list back once', () => {
    const cache = new FeedCache();
    const list = snapshot();
    cache.put('photos', list);
    expect(cache.take('photos')).toBe(list);
    expect(cache.take('photos')).toBeNull();
  });

  it('keeps only the most recently stashed lists', () => {
    const cache = new FeedCache();
    for (let index = 0; index <= FEED_CACHE_ENTRIES; index += 1) cache.put(`k${index}`, snapshot());
    expect(cache.take('k0')).toBeNull();
    expect(cache.take(`k${FEED_CACHE_ENTRIES}`)).not.toBeNull();
  });

  it('refreshes the position of a list stashed again', () => {
    const cache = new FeedCache();
    cache.put('k0', snapshot());
    for (let index = 1; index < FEED_CACHE_ENTRIES; index += 1) cache.put(`k${index}`, snapshot());
    cache.put('k0', snapshot());
    cache.put('extra', snapshot());
    expect(cache.take('k0')).not.toBeNull();
    expect(cache.take('k1')).toBeNull();
  });

  it('drops lists that are too old or too long', () => {
    let now = 0;
    const cache = new FeedCache(() => now);
    cache.put('old', snapshot());
    cache.put('long', snapshot(FEED_CACHE_MAX_ITEMS + 1));
    now = FEED_CACHE_MAX_AGE_MS + 1;
    expect(cache.take('old')).toBeNull();
    expect(cache.take('long')).toBeNull();
  });

  it('forgets everything when cleared', () => {
    const cache = new FeedCache();
    cache.put('photos', snapshot());
    cache.clear();
    expect(cache.take('photos')).toBeNull();
  });
});
