import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BrowseFeed } from './browseFeed.svelte';
import type { SearchQuery, SearchResult } from '$lib/types/search';
import { browseControls } from './browseControls.svelte';
import { selection } from './selection.svelte';

function result(nextPage: string | null): SearchResult {
  return { items: [], count: 0, total: 0, nextPage };
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
});
