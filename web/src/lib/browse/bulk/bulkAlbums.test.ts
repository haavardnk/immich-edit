import { addAssetsToAlbum, getAlbum, removeAssetsFromAlbum } from '#lib/api/albums.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { album } from '#lib/testing/albums.ts';
import { asset } from '#lib/testing/assets.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { library } from '#lib/stores/library.svelte.ts';
import { addToAlbums, changeAlbums, removeFromAlbum, removeFromViewedAlbum } from './bulkAlbums';

vi.mock('#lib/api/albums.ts', () => ({
  addAssetsToAlbum: vi.fn(),
  getAlbum: vi.fn(),
  removeAssetsFromAlbum: vi.fn()
}));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({
  metadataConsent: { gate: vi.fn(async () => true) }
}));

const A = '11111111-1111-1111-1111-111111111111';
const B = '22222222-2222-2222-2222-222222222222';
const C = '33333333-3333-3333-3333-333333333333';
const NORWAY = album('norway', { albumName: 'Norway 2026' });

const remove = vi.mocked(removeAssetsFromAlbum);
const add = vi.mocked(addAssetsToAlbum);
const refetch = vi.mocked(getAlbum);

function ids(): string[] {
  return browsing.assets.map((a) => a.id);
}

beforeEach(() => {
  browsing.set([asset(A), asset(`${A}_1`, { copyOf: A }), asset(B), asset(C)]);
  browsing.total = 4;
  selection.replace([`${A}_1`, B, C]);
  refetch.mockResolvedValue(NORWAY);
});

afterEach(() => {
  vi.clearAllMocks();
  toasts.items = [];
});

describe('removeFromAlbum', () => {
  it('sorts the results into removed, refused and failed', async () => {
    remove.mockResolvedValueOnce([
      { id: A, success: true },
      { id: B, success: false, error: 'no_permission' },
      { id: C, success: false, error: 'not_found' },
      { id: 'd', success: false, error: 'unknown' }
    ]);
    expect(await removeFromAlbum('norway', [A, B, C, 'd'])).toEqual({
      removed: [A],
      refused: 1,
      failed: 1
    });
  });
});

describe('removeFromViewedAlbum', () => {
  it('drops every tile of a removed photo and offers Undo', async () => {
    remove.mockResolvedValueOnce([
      { id: A, success: true },
      { id: B, success: true },
      { id: C, success: false, error: 'no_permission' }
    ]);
    expect(await removeFromViewedAlbum(NORWAY, [`${A}_1`, B, C])).toBe(true);
    expect(remove).toHaveBeenCalledWith('norway', [`${A}_1`, B, C]);
    expect(ids()).toEqual([C]);
    expect(browsing.total).toBe(1);
    expect(selection.count).toBe(0);
    const [toast] = toasts.items;
    expect(toast?.message).toBe('Removed 2 photos from Norway 2026');
    expect(toast?.description).toBe('1 belongs to someone else and stays');
    expect(toast?.action?.label).toBe('Undo');
    expect(refetch).toHaveBeenCalledWith('norway');
  });

  it('adds the photos back and restores their tiles on Undo', async () => {
    remove.mockResolvedValueOnce([
      { id: A, success: true },
      { id: B, success: true }
    ]);
    add.mockResolvedValueOnce([
      { id: A, success: true },
      { id: B, success: true }
    ]);
    await removeFromViewedAlbum(NORWAY, [A, B]);
    toasts.items[0]?.action?.run();
    await vi.waitFor(() => expect(ids()).toEqual([A, `${A}_1`, B, C]));
    expect(add).toHaveBeenCalledWith('norway', [A, B]);
    await vi.waitFor(() => expect(refetch).toHaveBeenCalledTimes(2));
  });

  it('keeps the grid and reports a failed request', async () => {
    remove.mockRejectedValueOnce(new Error('offline'));
    const fail = vi.spyOn(toasts, 'fail');
    expect(await removeFromViewedAlbum(NORWAY, [B])).toBe(false);
    expect(fail).toHaveBeenCalledWith('Remove from album', expect.any(Error));
    expect(ids()).toHaveLength(4);
    fail.mockRestore();
  });

  it('offers no Undo when Immich kept every photo', async () => {
    remove.mockResolvedValueOnce([{ id: B, success: false, error: 'no_permission' }]);
    expect(await removeFromViewedAlbum(NORWAY, [B])).toBe(false);
    expect(toasts.items[0]?.action).toBeUndefined();
    expect(ids()).toHaveLength(4);
    expect(refetch).not.toHaveBeenCalled();
  });
});

describe('addToAlbums', () => {
  it('refreshes the count of every album it added to', async () => {
    const empty = album('test', { albumName: 'Test', assetCount: 0 });
    library.albums = [empty, NORWAY];
    add.mockResolvedValue([{ id: A, success: true }]);
    refetch.mockImplementation(async (id) => ({
      ...(id === 'test' ? empty : NORWAY),
      assetCount: 3
    }));

    expect(await addToAlbums([A], ['test', 'norway'])).toBe(true);
    await vi.waitFor(() =>
      expect(library.albums.map((a) => [a.id, a.assetCount])).toEqual([
        ['norway', 3],
        ['test', 3]
      ])
    );
  });
});

describe('changeAlbums', () => {
  it('refreshes each album it added to or removed from once', async () => {
    add.mockResolvedValueOnce([{ id: B, success: true }]);
    remove.mockResolvedValueOnce([{ id: B, success: true }]);
    expect(await changeAlbums([B], ['test'], ['norway'], null)).toBe(true);
    expect(refetch.mock.calls.map(([id]) => id)).toEqual(['test', 'norway']);
  });

  it('counts refused removals as failed changes', async () => {
    remove.mockResolvedValueOnce([
      { id: B, success: true },
      { id: C, success: false, error: 'no_permission' }
    ]);
    expect(await changeAlbums([B, C], [], ['norway'], 'norway')).toBe(false);
    expect(ids()).toEqual([A, `${A}_1`, C]);
    expect(toasts.items[0]?.message).toBe('1 album changes failed');
  });
});
