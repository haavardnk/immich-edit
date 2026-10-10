import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BrowseFeed } from './browseFeed.svelte';
import type { BrowseFilter, SearchPage } from '$lib/types/search';
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

function result(nextCursor: string | null, ids: string[] = []): SearchPage {
  return { items: ids.map(asset), nextCursor };
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
  it('sends the returned cursor back and the merged filter along', async () => {
    const seen: Array<{ filter: BrowseFilter; cursor: string | undefined }> = [];
    const fetcher = vi.fn(async (filter: BrowseFilter, cursor: string | undefined) => {
      seen.push({ filter, cursor });
      return result(seen.length === 1 ? 'c2' : null, ['a']);
    });
    const feed = new BrowseFeed({
      baseFilter: () => ({ scope: { album: 'a1' } }),
      includeStats: false,
      fetcher
    });

    void feed.fetchPage(true);
    await vi.waitFor(() => expect(feed.nextCursor).toBe('c2'));
    void feed.loadMore();
    await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(2));

    expect(seen.map((call) => call.cursor)).toEqual([undefined, 'c2']);
    expect(seen[0]?.filter).toEqual({
      scope: { album: 'a1' },
      visibility: 'timeline',
      imagesOnly: true
    });
  });

  it('loads every remaining page before completing', async () => {
    const fetcher = vi.fn(async (_filter: BrowseFilter, cursor: string | undefined) => {
      if (cursor === '2') return result('3', ['b']);
      if (cursor === '3') return result(null, ['c']);
      return result('2', ['a']);
    });
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });

    await feed.fetchPage(true);
    expect(await feed.loadAll()).toBe(true);

    expect(fetcher.mock.calls.map(([, cursor]) => cursor)).toEqual([undefined, '2', '3']);
    expect(feed.assets.map((item) => item.id)).toEqual(['a', 'b', 'c']);
    expect(feed.nextCursor).toBeNull();
    expect(feed.loadingMore).toBe(false);
  });

  it('skips empty pages that still carry a cursor', async () => {
    const fetcher = vi.fn(async (_filter: BrowseFilter, cursor: string | undefined) => {
      if (cursor === '2') return result('3');
      if (cursor === '3') return result('4', ['b']);
      return result('2', ['a']);
    });
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });

    await feed.fetchPage(true);
    expect(await feed.loadMore()).toBe(true);

    expect(fetcher.mock.calls.map(([, cursor]) => cursor)).toEqual([undefined, '2', '3']);
    expect(feed.assets.map((item) => item.id)).toEqual(['a', 'b']);
    expect(feed.nextCursor).toBe('4');
  });

  it('does not report completion when a remaining page fails', async () => {
    const fetcher = vi.fn(async (_filter: BrowseFilter, cursor: string | undefined) => {
      if (cursor === '2') throw new Error('page failed');
      return result('2', ['a']);
    });
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });

    await feed.fetchPage(true);

    expect(await feed.loadAll()).toBe(false);
    expect(feed.assets.map((item) => item.id)).toEqual(['a']);
    expect(feed.nextCursor).toBe('2');
    expect(feed.loadingMore).toBe(false);
  });

  it('clears selection when the active filter changes', () => {
    const fetcher = vi.fn(async () => result(null));
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });
    feed.watchFilterChange();
    selection.selectLoaded(['a', 'b']);

    browseControls.filename = 'portrait';
    feed.watchFilterChange();

    expect(selection.active).toBe(false);
    expect(fetcher).toHaveBeenCalledOnce();
  });

  it('drops a stale response that resolves after a newer fetch', async () => {
    const resolvers: Array<(r: SearchPage) => void> = [];
    const fetcher = vi.fn(() => new Promise<SearchPage>((r) => resolvers.push(r)));
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });

    void feed.fetchPage(true);
    void feed.fetchPage(true);
    expect(fetcher).toHaveBeenCalledTimes(2);

    resolvers[1]?.(result(null, ['new']));
    await vi.waitFor(() => expect(feed.assets.map((a) => a.id)).toEqual(['new']));

    resolvers[0]?.(result('2', ['stale']));
    await flush();

    expect(feed.assets.map((a) => a.id)).toEqual(['new']);
    expect(feed.nextCursor).toBeNull();
  });

  it('does not append a page fetched under the previous filter', async () => {
    const resolvers: Array<(r: SearchPage) => void> = [];
    const fetcher = vi.fn(() => new Promise<SearchPage>((r) => resolvers.push(r)));
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });

    void feed.fetchPage(true);
    resolvers[0]?.(result('2', ['a']));
    await vi.waitFor(() => expect(feed.nextCursor).toBe('2'));

    void feed.loadMore();
    expect(feed.loadingMore).toBe(true);
    void feed.fetchPage(true);

    resolvers[2]?.(result(null, ['b']));
    await vi.waitFor(() => expect(feed.assets.map((a) => a.id)).toEqual(['b']));

    resolvers[1]?.(result('3', ['a', 'stale']));
    await flush();

    expect(feed.assets.map((a) => a.id)).toEqual(['b']);
    expect(feed.nextCursor).toBeNull();
    expect(feed.loadingMore).toBe(false);
  });
});

describe('BrowseFeed as the browsing pager', () => {
  function pagedFeed(): { feed: BrowseFeed; fetcher: ReturnType<typeof vi.fn> } {
    const fetcher = vi.fn(async (_filter: BrowseFilter, cursor: string | undefined) =>
      cursor === '2' ? result(null, ['c', 'd']) : result('2', ['a', 'b'])
    );
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });
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
