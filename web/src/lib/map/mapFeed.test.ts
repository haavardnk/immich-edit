import { MapFeed } from '#lib/map/mapFeed.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import { describe, expect, it, vi } from 'vitest';

function ids(count: number): string[] {
  return Array.from({ length: count }, (_, index) => `a${index}`);
}

describe('MapFeed', () => {
  it('pages the photos in view into browsing and jumps to a later one', async () => {
    const load = vi.fn((page: readonly string[]) => Promise.resolve(page.map((id) => asset(id))));
    const feed = new MapFeed(load);
    await feed.show(ids(130));
    expect(feed.assets).toHaveLength(60);
    expect(browsing.assets).toBe(feed.assets);
    expect(browsing.total).toBe(130);
    expect(feed.hasMore).toBe(true);
    expect(await feed.jumpTo('a125')).toBe(true);
    expect(feed.assets).toHaveLength(130);
    expect(load).toHaveBeenCalledTimes(3);
    expect(await feed.jumpTo('missing')).toBe(false);
  });

  it('drops a page that lands after the view moved on', async () => {
    let release: () => void = () => undefined;
    const load = vi.fn(
      (page: readonly string[]) =>
        new Promise<ReturnType<typeof asset>[]>((resolve) => {
          release = () => resolve(page.map((id) => asset(id)));
        })
    );
    const feed = new MapFeed(load);
    const stale = feed.show(['old']);
    const first = release;
    const fresh = feed.show(['new']);
    first();
    release();
    await Promise.all([stale, fresh]);
    expect(feed.assets.map((a) => a.id)).toEqual(['new']);
  });

  it('keeps the loaded set when the same photos stay in view', async () => {
    const load = vi.fn((page: readonly string[]) => Promise.resolve(page.map((id) => asset(id))));
    const feed = new MapFeed(load);
    await feed.show(['a', 'b']);
    await feed.show(['a', 'b']);
    expect(load).toHaveBeenCalledTimes(1);
  });
});
