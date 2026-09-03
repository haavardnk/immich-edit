import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BrowseFeed } from './browseFeed.svelte';
import type { SearchQuery, SearchResult } from '$lib/types/search';
import type { AssetSummary } from '$lib/types/album';
import { browseControls } from './browseControls.svelte';
import { selection } from './selection.svelte';

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
});

describe('BrowseFeed pagination', () => {
  it('sends the next page token back as a number', async () => {
    const seen: SearchQuery[] = [];
    const fetcher = vi.fn(async (body: SearchQuery) => {
      seen.push(body);
      return result(seen.length === 1 ? '2' : null);
    });
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });

    feed.fetchPage(true);
    await vi.waitFor(() => expect(feed.nextPage).toBe('2'));
    feed.loadMore();
    await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(2));

    expect(seen[0].page).toBeUndefined();
    expect(seen[1].page).toBe(2);
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

    feed.fetchPage(true);
    feed.fetchPage(true);
    expect(fetcher).toHaveBeenCalledTimes(2);

    resolvers[1](result(null, ['new']));
    await vi.waitFor(() => expect(feed.assets.map((a) => a.id)).toEqual(['new']));

    resolvers[0](result('2', ['stale']));
    await flush();

    expect(feed.assets.map((a) => a.id)).toEqual(['new']);
    expect(feed.nextPage).toBeNull();
  });

  it('does not append a page fetched under the previous filter', async () => {
    const resolvers: Array<(r: SearchResult) => void> = [];
    const fetcher = vi.fn(() => new Promise<SearchResult>((r) => resolvers.push(r)));
    const feed = new BrowseFeed({ baseBody: () => ({}), includeStats: false, fetcher });

    feed.fetchPage(true);
    resolvers[0](result('2', ['a']));
    await vi.waitFor(() => expect(feed.nextPage).toBe('2'));

    feed.loadMore();
    expect(feed.loadingMore).toBe(true);
    feed.fetchPage(true);

    resolvers[2](result(null, ['b']));
    await vi.waitFor(() => expect(feed.assets.map((a) => a.id)).toEqual(['b']));

    resolvers[1](result('3', ['a', 'stale']));
    await flush();

    expect(feed.assets.map((a) => a.id)).toEqual(['b']);
    expect(feed.nextPage).toBeNull();
    expect(feed.loadingMore).toBe(false);
  });
});
