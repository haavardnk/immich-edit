import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { closedQuietly, leaveLoupeFor, returnToLoupe, returnsToLoupe } from './loupeAway.svelte';
import { skipZoomIn } from './loupeZoom';

const state = vi.hoisted(() => ({ url: new URL('http://x/photos?sort=asc') }));

vi.mock('$app/state', () => ({
  page: {
    get url() {
      return state.url;
    }
  }
}));
vi.mock('./loupeZoom', () => ({ skipZoomIn: vi.fn() }));

const PHOTOS = new URL('http://x/photos?sort=asc');
const MAP = new URL('http://x/map?at=1,2');

function photo(id: string): AssetSummary {
  return { id } as AssetSummary;
}

async function leave(info = true): Promise<void> {
  browsing.set([photo('a')]);
  browseView.openLoupe('a');
  browseView.setLoupeInfoOpen(info);
  const complete = Promise.resolve();
  expect(leaveLoupeFor('a', PHOTOS, MAP, { type: 'link', complete })).toEqual({
    direction: 'forward',
    whole: true
  });
  await complete;
  browsing.set([]);
}

beforeEach(() => {
  vi.useFakeTimers();
  browseView.setCompact(true);
  state.url = PHOTOS;
});

afterEach(() => {
  returnToLoupe(MAP, new URL('http://x/library'), { complete: Promise.resolve() });
  browseView.closeLoupe();
  browseView.setCompact(false);
  vi.useRealTimers();
  vi.mocked(skipZoomIn).mockClear();
});

describe('loupeAway', () => {
  it('closes the loupe quietly once the page it links to lands', async () => {
    await leave();
    expect(browseView.loupeId).toBeNull();
    expect(closedQuietly('a')).toBe(true);
    expect(closedQuietly('a')).toBe(false);
  });

  it('reopens the photo with its info on the way back, without a zoom', async () => {
    await leave();
    expect(returnsToLoupe('/photos?sort=asc')).toBe(true);
    browsing.set([photo('a')]);
    const complete = Promise.resolve();
    const turn = returnToLoupe(MAP, PHOTOS, { type: 'popstate', complete });
    expect(turn).toMatchObject({ direction: 'back', whole: true });
    await turn?.ready;
    expect(browseView.loupeId).toBe('a');
    expect(browseView.loupeInfoOpen).toBe(true);
    expect(skipZoomIn).toHaveBeenCalledWith('a');
    expect(returnsToLoupe('/photos?sort=asc')).toBe(false);
  });

  it('holds the page change until the grid lists the photo', async () => {
    await leave(false);
    const complete = Promise.resolve();
    let ready = false;
    void returnToLoupe(MAP, PHOTOS, { type: 'popstate', complete })?.ready?.then(
      () => (ready = true)
    );
    await vi.advanceTimersByTimeAsync(300);
    expect(ready).toBe(false);
    expect(browseView.loupeId).toBeNull();
    browsing.set([photo('a')]);
    await vi.advanceTimersByTimeAsync(0);
    expect(ready).toBe(true);
    expect(browseView.loupeId).toBe('a');
    expect(browseView.loupeInfoOpen).toBe(false);
    expect(skipZoomIn).toHaveBeenCalledWith('a');
  });

  it('lets the page change go and zooms in once the grid lists the photo late', async () => {
    await leave(false);
    const complete = Promise.resolve();
    let ready = false;
    void returnToLoupe(MAP, PHOTOS, { type: 'popstate', complete })?.ready?.then(
      () => (ready = true)
    );
    await vi.advanceTimersByTimeAsync(1000);
    expect(ready).toBe(true);
    expect(browseView.loupeId).toBeNull();
    browsing.set([photo('a')]);
    await vi.advanceTimersByTimeAsync(0);
    expect(browseView.loupeId).toBe('a');
    expect(skipZoomIn).not.toHaveBeenCalled();
  });

  it('keeps the way back through map moves and drops it for another page', async () => {
    await leave();
    const complete = Promise.resolve();
    expect(returnToLoupe(MAP, new URL('http://x/map?at=3,4'), { complete })).toBeNull();
    expect(returnsToLoupe('/photos?sort=asc')).toBe(true);
    expect(returnToLoupe(MAP, new URL('http://x/albums'), { complete })).toBeNull();
    expect(returnsToLoupe('/photos?sort=asc')).toBe(false);
  });
});
