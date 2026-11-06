import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BrowseFeed } from './browseFeed.svelte';
import type { BrowseFilter, SearchPage, SearchWindow, WindowReach } from '#lib/types/search.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import * as searchApi from '#lib/api/search.ts';
import { browseControls } from './browseControls.svelte';
import { selection } from './selection.svelte';
import { browsing } from './browsing.svelte';
import { toasts } from './toasts.svelte';

vi.mock('#lib/api/search.ts', async (importOriginal) => {
  const actual = await importOriginal<typeof import('#lib/api/search.ts')>();
  return {
    ...actual,
    searchFirst: vi.fn(),
    searchMetadata: vi.fn(),
    searchStatistics: vi.fn()
  };
});

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
  toasts.items = [];
  vi.mocked(searchApi.searchStatistics).mockReset().mockResolvedValue({ total: 20 });
  vi.mocked(searchApi.searchMetadata)
    .mockReset()
    .mockResolvedValue(result(null, ['top']));
  vi.mocked(searchApi.searchFirst).mockReset().mockResolvedValue(null);
});

const messages = () => toasts.items.map((toast) => toast.message);

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

  it('stops loading pages once its signal aborts', async () => {
    const abort = new AbortController();
    const fetcher = vi.fn(async (_filter: BrowseFilter, cursor: string | undefined) => {
      if (cursor !== '2') return result('2', ['a']);
      abort.abort();
      return result('3', ['b']);
    });
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });

    await feed.fetchPage(true);

    expect(await feed.loadAll(abort.signal)).toBe(false);
    expect(fetcher.mock.calls.map(([, cursor]) => cursor)).toEqual([undefined, '2']);
    expect(feed.assets.map((item) => item.id)).toEqual(['a', 'b']);
    expect(feed.loadingMore).toBe(false);
  });

  it('clears selection when the active filter changes', () => {
    const fetcher = vi.fn(async () => result(null));
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, fetcher });
    feed.watchFilterChange();
    selection.selectLoaded(['a', 'b']);

    browseControls.filters.filename = 'portrait';
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

describe('BrowseFeed window mode', () => {
  const library = Array.from({ length: 20 }, (_, i) => `p${i}`);

  function serve(assetId: string, _filter: BrowseFilter, reach: WindowReach): SearchWindow {
    const at = library.indexOf(assetId);
    const start = Math.max(0, at - reach.before);
    const end = Math.min(library.length, at + reach.after + 1);
    return {
      items: library.slice(start, end).map(asset),
      hasBefore: start > 0,
      hasAfter: end < library.length
    };
  }

  function windowFeed() {
    const windowFetcher = vi.fn(async (id: string, filter: BrowseFilter, reach: WindowReach) =>
      serve(id, filter, reach)
    );
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, windowFetcher });
    return { feed, windowFetcher };
  }

  const ids = (feed: BrowseFeed) => feed.assets.map((item) => item.id);

  it('starts the list around an anchor with both ends open', async () => {
    const { feed, windowFetcher } = windowFeed();
    expect(await feed.jumpTo('p10', { reach: { before: 2, after: 3 } })).toBe(true);

    expect(ids(feed)).toEqual(['p8', 'p9', 'p10', 'p11', 'p12', 'p13']);
    expect(windowFetcher).toHaveBeenCalledWith('p10', expect.any(Object), { before: 2, after: 3 });
    expect(feed.windowed).toBe(true);
    expect(feed.hasPrevious).toBe(true);
    expect(feed.hasMore).toBe(true);
    expect(browsing.assets).toBe(feed.assets);
    expect(browsing.hasPrevious).toBe(true);
  });

  it('extends each end from its edge master without duplicates', async () => {
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 3 } });

    expect(await feed.loadPrevious()).toBe(true);
    expect(windowFetcher).toHaveBeenLastCalledWith('p8', expect.any(Object), {
      before: 500,
      after: 0
    });
    expect(ids(feed).slice(0, 3)).toEqual(['p0', 'p1', 'p2']);
    expect(feed.hasPrevious).toBe(false);

    expect(await browsing.requestMore()).toBe(true);
    expect(windowFetcher).toHaveBeenLastCalledWith('p13', expect.any(Object), {
      before: 0,
      after: 500
    });
    expect(ids(feed)).toEqual(library);
    expect(feed.hasMore).toBe(false);
    expect(feed.loadingMore).toBe(false);
    expect(feed.loadingPrevious).toBe(false);
  });

  it('shares one request between concurrent loads of the near end', async () => {
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 3 } });
    const first = feed.loadPrevious();
    const second = browsing.requestPrevious();
    expect(feed.loadingPrevious).toBe(true);
    expect(await first).toBe(true);
    expect(await second).toBe(true);
    expect(windowFetcher).toHaveBeenCalledTimes(2);
  });

  it('loads both directions for load all', async () => {
    const { feed } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 1, after: 1 } });
    expect(await feed.loadAll()).toBe(true);
    expect(ids(feed)).toEqual(library);
  });

  it('drops an extension that lands after a new jump', async () => {
    let release = (_value: SearchWindow): void => undefined;
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 2 } });
    windowFetcher.mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
    const stale = feed.loadPrevious();
    await feed.jumpTo('p3', { reach: { before: 1, after: 1 } });
    release(serve('p8', {}, { before: 500, after: 0 }));
    expect(await stale).toBe(false);
    expect(ids(feed)).toEqual(['p2', 'p3', 'p4']);
  });

  it('reports a failed jump and keeps the list', async () => {
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 1, after: 1 } });
    windowFetcher.mockRejectedValueOnce(new Error('gone'));
    expect(await feed.jumpTo('p2', { failure: 'Jump to date' })).toBe(false);
    expect(ids(feed)).toEqual(['p9', 'p10', 'p11']);
    expect(feed.hasPrevious).toBe(true);
    expect(feed.hasMore).toBe(true);
    expect(messages()).toEqual(['Jump to date: gone']);
  });

  it('keeps extending the current list while a jump is pending', async () => {
    let release = (_value: SearchWindow): void => undefined;
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 2 } });
    windowFetcher.mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
    const jump = feed.jumpTo('p2');
    expect(await feed.loadMore()).toBe(true);
    expect(ids(feed).at(-1)).toBe('p19');
    release(serve('p2', {}, { before: 1, after: 1 }));
    expect(await jump).toBe(true);
    expect(ids(feed)).toEqual(['p1', 'p2', 'p3']);
  });

  it('drops an extension when the window restarts from the top', async () => {
    let release = (_value: SearchWindow): void => undefined;
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 2 } });
    windowFetcher.mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
    const stale = feed.loadMore();
    const restart = feed.fetchPage(true);
    expect(feed.windowed).toBe(false);
    expect(feed.hasPrevious).toBe(false);
    await restart;
    release(serve('p12', {}, { before: 0, after: 500 }));
    expect(await stale).toBe(false);
    expect(ids(feed)).toEqual(['top']);
    expect(feed.loadingMore).toBe(false);
  });

  it('drops an extension whose edge photo left the list meanwhile', async () => {
    let release = (_value: SearchWindow): void => undefined;
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 2 } });
    windowFetcher.mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
    const stale = feed.loadMore();
    browsing.remove('p12');
    release(serve('p12', {}, { before: 0, after: 500 }));
    expect(await stale).toBe(false);
    expect(ids(feed)).toEqual(['p8', 'p9', 'p10', 'p11']);
  });

  it('retries an empty extension from the next photo inward', async () => {
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 2 } });
    windowFetcher.mockResolvedValueOnce({ items: [], hasBefore: false, hasAfter: false });
    expect(await feed.loadMore()).toBe(true);
    expect(windowFetcher.mock.calls.slice(-2).map(([id]) => id)).toEqual(['p12', 'p11']);
    expect(ids(feed)).toEqual(library.slice(8));
  });

  it('reports an end that keeps coming back empty', async () => {
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 2, after: 2 } });
    windowFetcher.mockResolvedValue({ items: [], hasBefore: true, hasAfter: true });
    expect(await feed.loadPrevious()).toBe(false);
    expect(windowFetcher.mock.calls.slice(1).map(([id]) => id)).toEqual(['p8', 'p9', 'p10', 'p11']);
    expect(feed.hasPrevious).toBe(false);
    expect(messages()).toEqual(['Could not load earlier photos']);
  });

  it('counts once per filter across jumps', async () => {
    const windowFetcher = vi.fn(async (id: string, filter: BrowseFilter, reach: WindowReach) =>
      serve(id, filter, reach)
    );
    const feed = new BrowseFeed({ baseFilter: () => ({}), windowFetcher });
    await feed.jumpTo('p10', { reach: { before: 1, after: 1 } });
    await vi.waitFor(() => expect(feed.totalCount).toBe(20));
    await feed.jumpTo('p3', { reach: { before: 1, after: 1 } });
    expect(feed.totalCount).toBe(20);
    expect(searchApi.searchStatistics).toHaveBeenCalledOnce();
    browseControls.filters.filename = 'x';
    await feed.jumpTo('p5', { reach: { before: 1, after: 1 } });
    expect(searchApi.searchStatistics).toHaveBeenCalledTimes(2);
  });

  it('falls back to the top of the list for a lone anchor', async () => {
    const windowFetcher = vi.fn(async () => ({
      items: [asset('undated')],
      hasBefore: false,
      hasAfter: false
    }));
    const feed = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, windowFetcher });
    expect(await feed.jumpTo('undated')).toBe(true);
    expect(searchApi.searchMetadata).toHaveBeenCalledOnce();
    expect(ids(feed)).toEqual(['top']);
    expect(feed.windowed).toBe(false);
  });

  it('resumes at a photo and falls back to the top when it is gone', async () => {
    const { feed, windowFetcher } = windowFeed();
    expect(await feed.resume('p10')).toBe(true);
    expect(feed.windowed).toBe(true);
    windowFetcher.mockRejectedValueOnce(new Error('gone'));
    expect(await feed.resume('p4')).toBe(true);
    expect(ids(feed)).toEqual(['top']);
    expect(messages()).toEqual([]);
  });

  it('loads the true ends for Home and End', async () => {
    const { feed, windowFetcher } = windowFeed();
    await feed.jumpTo('p10', { reach: { before: 1, after: 1 } });
    vi.mocked(searchApi.searchFirst).mockResolvedValueOnce('p19');
    expect(await feed.loadEdge('last')).toBe(true);
    expect(searchApi.searchFirst).toHaveBeenCalledWith(expect.any(Object), {
      field: browseControls.sort.field,
      direction: browseControls.sort.direction === 'desc' ? 'asc' : 'desc'
    });
    expect(windowFetcher).toHaveBeenLastCalledWith('p19', expect.any(Object), {
      before: 400,
      after: 0
    });
    expect(ids(feed).at(-1)).toBe('p19');
    expect(feed.hasMore).toBe(false);
    await feed.jumpTo('p10', { reach: { before: 1, after: 1 } });
    expect(await feed.loadEdge('first')).toBe(true);
    expect(ids(feed)).toEqual(['top']);
  });

  it('jumps only on metadata feeds and leaves window mode on reset', async () => {
    const fetcher = vi.fn(async () => result(null, ['top']));
    const windowFetcher = vi.fn(async (id: string, filter: BrowseFilter, reach: WindowReach) =>
      serve(id, filter, reach)
    );
    const feed = new BrowseFeed({
      baseFilter: () => ({}),
      includeStats: false,
      fetcher,
      windowFetcher
    });
    expect(feed.metadataBacked).toBe(false);
    expect(await feed.jumpTo('p10')).toBe(false);
    const metadata = new BrowseFeed({ baseFilter: () => ({}), includeStats: false, windowFetcher });
    await metadata.jumpTo('p10', { reach: { before: 1, after: 1 } });
    expect(metadata.hasPrevious).toBe(true);
    metadata.reset();
    expect(metadata.windowed).toBe(false);
    expect(metadata.hasPrevious).toBe(false);
  });
});
