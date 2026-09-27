import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BrowseFeed } from './browseFeed.svelte';
import type { SearchQuery, SearchResult } from '$lib/types/search';
import type { AssetSummary } from '$lib/types/album';
import { browseControls } from './browseControls.svelte';
import { selection } from './selection.svelte';
import { browsing } from './browsing.svelte';

function asset(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

function result(nextPage: string | null, ids: string[] = []): SearchResult {
  const items = ids.map(asset);
  return { items, count: items.length, total: items.length, nextPage };
}

function flush(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

beforeEach(() => {
  browseControls.reset();
  selection.clear();
  browsing.clear();
});

describe('BrowseFeed pagination', () => {
  it('sends the next page token back as a number', async () => {
    const seen: SearchQuery[] = [];
    const fetcher = vi.fn(async (body: SearchQuery) => {
      seen.push(body);
      return result(seen.length === 1 ? '2' : null);
    });
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });

    void feed.fetchPage(true);
    await vi.waitFor(() => expect(feed.nextPage).toBe('2'));
    void feed.loadMore();
    await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(2));

    expect(seen[0]?.page).toBeUndefined();
    expect(seen[1]?.page).toBe(2);
  });

  it('loads every remaining page before completing', async () => {
    const fetcher = vi.fn(async (body: SearchQuery) => {
      if (body.page === 2) return result('3', ['b']);
      if (body.page === 3) return result(null, ['c']);
      return result('2', ['a']);
    });
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });

    await feed.fetchPage(true);
    expect(await feed.loadAll()).toBe(true);

    expect(fetcher.mock.calls.map(([body]) => body.page)).toEqual([undefined, 2, 3]);
    expect(feed.assets.map((item) => item.id)).toEqual(['a', 'b', 'c']);
    expect(feed.nextPage).toBeNull();
    expect(feed.loadingMore).toBe(false);
  });

  it('does not report completion when a remaining page fails', async () => {
    const fetcher = vi.fn(async (body: SearchQuery) => {
      if (body.page === 2) throw new Error('page failed');
      return result('2', ['a']);
    });
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });

    await feed.fetchPage(true);

    expect(await feed.loadAll()).toBe(false);
    expect(feed.assets.map((item) => item.id)).toEqual(['a']);
    expect(feed.nextPage).toBe('2');
    expect(feed.loadingMore).toBe(false);
  });

  it('clears selection when the active filter changes', () => {
    const fetcher = vi.fn(async () => result(null));
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });
    feed.watchFilterChange();
    selection.selectLoaded(['a', 'b']);

    browseControls.filename = 'portrait';
    feed.watchFilterChange();

    expect(selection.active).toBe(false);
    expect(fetcher).toHaveBeenCalledOnce();
  });

  it('drops a stale response that resolves after a newer fetch', async () => {
    const resolvers: Array<(r: SearchResult) => void> = [];
    const fetcher = vi.fn(() => new Promise<SearchResult>((r) => resolvers.push(r)));
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });

    void feed.fetchPage(true);
    void feed.fetchPage(true);
    expect(fetcher).toHaveBeenCalledTimes(2);

    resolvers[1]?.(result(null, ['new']));
    await vi.waitFor(() => expect(feed.assets.map((a) => a.id)).toEqual(['new']));

    resolvers[0]?.(result('2', ['stale']));
    await flush();

    expect(feed.assets.map((a) => a.id)).toEqual(['new']);
    expect(feed.nextPage).toBeNull();
  });

  it('does not append a page fetched under the previous filter', async () => {
    const resolvers: Array<(r: SearchResult) => void> = [];
    const fetcher = vi.fn(() => new Promise<SearchResult>((r) => resolvers.push(r)));
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });

    void feed.fetchPage(true);
    resolvers[0]?.(result('2', ['a']));
    await vi.waitFor(() => expect(feed.nextPage).toBe('2'));

    void feed.loadMore();
    expect(feed.loadingMore).toBe(true);
    void feed.fetchPage(true);

    resolvers[2]?.(result(null, ['b']));
    await vi.waitFor(() => expect(feed.assets.map((a) => a.id)).toEqual(['b']));

    resolvers[1]?.(result('3', ['a', 'stale']));
    await flush();

    expect(feed.assets.map((a) => a.id)).toEqual(['b']);
    expect(feed.nextPage).toBeNull();
    expect(feed.loadingMore).toBe(false);
  });
});

describe('BrowseFeed as the browsing pager', () => {
  function pagedFeed(): { feed: BrowseFeed; fetcher: ReturnType<typeof vi.fn> } {
    const fetcher = vi.fn(async (body: SearchQuery) =>
      body.page === 2 ? result(null, ['c', 'd']) : result('2', ['a', 'b'])
    );
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });
    return { feed, fetcher };
  }

  it('loads the next page once for concurrent requests and keeps the feed array', async () => {
    const { feed, fetcher } = pagedFeed();
    await feed.fetchPage(true);

    const first = browsing.requestMore();
    const second = browsing.requestMore();
    expect(await first).toBe(true);
    expect(await second).toBe(true);

    expect(fetcher).toHaveBeenCalledTimes(2);
    expect(browsing.assets).toBe(feed.assets);
    expect(browsing.assets.map((a) => a.id)).toEqual(['a', 'b', 'c', 'd']);
    expect(browsing.hasMore).toBe(false);
    expect(await browsing.requestMore()).toBe(false);
    expect(fetcher).toHaveBeenCalledTimes(2);
  });

  it('prefetches only near the end of the loaded set', async () => {
    const { feed, fetcher } = pagedFeed();
    await feed.fetchPage(true);
    browsing.set(
      Array.from({ length: 30 }, (_, i) => asset(`p${i}`)),
      feed
    );

    browsing.prefetchNear('p5');
    expect(fetcher).toHaveBeenCalledTimes(1);
    browsing.prefetchNear('p20');
    expect(fetcher).toHaveBeenCalledTimes(2);
  });

  it('forgets the pager when a list without paging replaces it', async () => {
    const { feed, fetcher } = pagedFeed();
    await feed.fetchPage(true);

    browsing.set([asset('folder')]);

    expect(browsing.hasMore).toBe(false);
    expect(await browsing.requestMore()).toBe(false);
    expect(fetcher).toHaveBeenCalledTimes(1);
  });
});
