import type { AlbumSummary } from '#lib/types/album.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  albumsWithAsset: vi.fn<(id: string) => Promise<AlbumSummary[]>>()
}));

vi.mock('#lib/api/albums.ts', () => ({ albumsWithAsset: h.albumsWithAsset }));

const A = '0f8fad5b-d9cb-469f-a165-70867728950e';

function album(id: string, albumName: string): AlbumSummary {
  return { id, albumName, assetCount: 1, albumThumbnailAssetId: null, updatedAt: null };
}

beforeEach(() => {
  vi.resetModules();
  h.albumsWithAsset.mockReset();
});

async function store() {
  return (await import('#lib/stores/assetAlbums.svelte.ts')).assetAlbums;
}

describe('asset albums', () => {
  it('loads once per source photo, sorted by name, and shares it with copies', async () => {
    h.albumsWithAsset.mockResolvedValue([album('2', 'Summer'), album('1', 'Autumn')]);
    const albums = await store();
    expect(albums.albumsOf(A)).toBeNull();
    await Promise.all([albums.ensure(A), albums.ensure(`${A}_1`)]);
    await albums.ensure(A);
    expect(h.albumsWithAsset).toHaveBeenCalledTimes(1);
    expect(h.albumsWithAsset).toHaveBeenCalledWith(A);
    expect(albums.albumsOf(`${A}_1`)?.map((a) => a.albumName)).toEqual(['Autumn', 'Summer']);
  });

  it('reloads after an album change forgets the photo', async () => {
    h.albumsWithAsset.mockResolvedValueOnce([]).mockResolvedValueOnce([album('1', 'Bergen')]);
    const albums = await store();
    await albums.ensure(A);
    albums.forget([`${A}_2`]);
    expect(albums.albumsOf(A)).toBeNull();
    await albums.ensure(A);
    expect(albums.albumsOf(A)?.map((a) => a.id)).toEqual(['1']);
  });

  it('drops a load that a change overtook', async () => {
    let answer: (albums: AlbumSummary[]) => void = () => {};
    h.albumsWithAsset
      .mockImplementationOnce(() => new Promise((done) => (answer = done)))
      .mockResolvedValueOnce([album('2', 'New')]);
    const albums = await store();
    const stale = albums.ensure(A);
    albums.forget([A]);
    const fresh = albums.ensure(A);
    answer([album('1', 'Old')]);
    await Promise.all([stale, fresh]);
    expect(albums.albumsOf(A)?.map((a) => a.id)).toEqual(['2']);
  });

  it('keeps a failure until a retry', async () => {
    h.albumsWithAsset
      .mockRejectedValueOnce(new Error('offline'))
      .mockResolvedValueOnce([album('1', 'Bergen')]);
    const albums = await store();
    await albums.ensure(A);
    expect(albums.errorOf(A)).toBe('offline');
    await albums.ensure(A);
    expect(h.albumsWithAsset).toHaveBeenCalledTimes(1);
    await albums.retry(A);
    expect(albums.errorOf(A)).toBeNull();
    expect(albums.albumsOf(A)).toHaveLength(1);
  });
});
