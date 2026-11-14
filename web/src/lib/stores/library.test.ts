import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createAlbum, listAlbums } from '#lib/api/albums.ts';
import { listPeople, type PersonSummary } from '#lib/api/people.ts';
import { folderPaths } from '#lib/api/folders.ts';
import { searchStatistics } from '#lib/api/search.ts';
import type { AlbumSummary } from '#lib/types/album.ts';

vi.mock('#lib/api/people.ts', () => ({ listPeople: vi.fn() }));
vi.mock('#lib/api/albums.ts', () => ({
  listAlbums: vi.fn().mockResolvedValue([]),
  createAlbum: vi.fn()
}));
vi.mock('#lib/api/tags.ts', () => ({ listTags: vi.fn().mockResolvedValue([]) }));
vi.mock('#lib/api/folders.ts', () => ({ folderPaths: vi.fn() }));
vi.mock('#lib/api/search.ts', () => ({ searchStatistics: vi.fn() }));
vi.mock('#lib/stores/editedThumbs.svelte.ts', () => ({
  editedThumbs: { count: 2, loadOnce: vi.fn().mockResolvedValue(undefined) }
}));

const person: PersonSummary = {
  id: 'p1',
  name: 'Ingrid',
  thumbnailPath: '',
  isHidden: false,
  updatedAt: null,
  assetCount: 3
};

function album(albumName: string): AlbumSummary {
  return {
    id: albumName,
    albumName,
    assetCount: 0,
    albumThumbnailAssetId: null,
    updatedAt: null,
    ownerId: 'user-me',
    shared: false,
    endDate: null
  };
}

async function load() {
  return (await import('#lib/stores/library.svelte.ts')).library;
}

beforeEach(() => {
  vi.mocked(listPeople).mockReset();
  vi.mocked(folderPaths).mockResolvedValue(['/a', '/a/b']);
  vi.mocked(searchStatistics).mockResolvedValue({ total: 7 });
});

afterEach(() => {
  vi.resetModules();
});

describe('library loaders', () => {
  it('shares one request per view and remembers the result', async () => {
    vi.mocked(listPeople).mockResolvedValue([person]);
    const library = await load();

    const first = library.ensure('people');
    const second = library.ensure('people');
    expect(second).toBe(first);
    expect(library.isLoaded('people')).toBe(false);
    await first;
    await library.ensure('people');

    expect(listPeople).toHaveBeenCalledOnce();
    expect(library.isLoaded('people')).toBe(true);
    expect(library.people).toEqual([person]);
  });

  it('records a failure per view and retries on the next request', async () => {
    vi.mocked(listPeople).mockRejectedValueOnce(new Error('offline'));
    vi.mocked(listPeople).mockResolvedValueOnce([person]);
    const library = await load();

    await expect(library.ensure('people')).rejects.toThrow('offline');
    expect(library.errorFor('people')).toBe('offline');
    expect(library.errorFor('tags')).toBeNull();
    expect(library.isLoaded('people')).toBe(false);

    library.request('people');
    expect(library.errorFor('people')).toBeNull();
    await vi.waitFor(() => expect(library.isLoaded('people')).toBe(true));
    expect(listPeople).toHaveBeenCalledTimes(2);
  });

  it('keeps collection counts unknown until their views load', async () => {
    vi.mocked(listPeople).mockResolvedValue([person]);
    const library = await load();
    expect(library.foldersCount).toBeNull();
    expect(library.editedCount).toBeNull();
    expect(library.favoritesCount).toBeNull();

    await library.loadCounts();

    expect(library.foldersCount).toBe(2);
    expect(library.editedCount).toBe(2);
    expect(library.favoritesCount).toBe(7);
    expect(library.isLoaded('people')).toBe(true);
  });

  it('retries the counts after a failure', async () => {
    vi.mocked(listPeople).mockResolvedValue([]);
    vi.mocked(searchStatistics).mockRejectedValueOnce(new Error('offline'));
    const library = await load();

    await library.loadCounts();
    expect(library.photosCount).toBeNull();
    await library.loadCounts();
    expect(library.photosCount).toBe(7);
  });
});

describe('creating an album', () => {
  it('adds the new album to the loaded list in name order', async () => {
    vi.mocked(listAlbums).mockResolvedValueOnce([album('Archive'), album('Norway')]);
    vi.mocked(createAlbum).mockResolvedValueOnce(album('Lofoten'));
    const library = await load();
    await library.ensure('albums');

    const created = await library.createAlbum('Lofoten');

    expect(createAlbum).toHaveBeenCalledWith('Lofoten');
    expect(created?.albumName).toBe('Lofoten');
    expect(library.albums.map((entry) => entry.albumName)).toEqual([
      'Archive',
      'Lofoten',
      'Norway'
    ]);
  });

  it('reports a refused create and leaves the list alone', async () => {
    vi.mocked(listAlbums).mockResolvedValueOnce([album('Archive')]);
    vi.mocked(createAlbum).mockRejectedValueOnce(new Error('Forbidden'));
    const library = await load();
    const { toasts } = await import('#lib/stores/toasts.svelte.ts');
    await library.ensure('albums');

    expect(await library.createAlbum('Lofoten')).toBeNull();

    expect(library.albums.map((entry) => entry.albumName)).toEqual(['Archive']);
    expect(toasts.items.map((toast) => toast.message)).toContain(
      'Failed to create album: Forbidden'
    );
  });
});
