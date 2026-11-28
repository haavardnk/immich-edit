import { album as albumFixture } from '#lib/testing/albums.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  albumsWithAsset: vi.fn<(id: string) => Promise<AlbumSummary[]>>(),
  albumMemberships: vi.fn<(ids: readonly string[]) => Promise<Record<string, AlbumSummary[]>>>()
}));

vi.mock('#lib/api/albums.ts', () => ({
  albumsWithAsset: h.albumsWithAsset,
  albumMemberships: h.albumMemberships
}));

const A = '0f8fad5b-d9cb-469f-a165-70867728950e';
const B = '7c9e6679-7425-40de-944b-e07fc1f90ae7';

function album(id: string, albumName: string): AlbumSummary {
  return albumFixture(id, { albumName, assetCount: 1 });
}

beforeEach(() => {
  vi.resetModules();
  h.albumsWithAsset.mockReset();
  h.albumMemberships.mockReset();
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

  it('forgets every photo and drops loads still running on clear', async () => {
    let answer: (albums: AlbumSummary[]) => void = () => {};
    h.albumsWithAsset
      .mockResolvedValueOnce([album('1', 'Bergen')])
      .mockImplementationOnce(() => new Promise((done) => (answer = done)));
    const albums = await store();
    await albums.ensure(A);
    const running = albums.ensure(B);
    albums.clear();
    answer([album('2', 'Gone')]);
    await running;
    expect(albums.albumsOf(A)).toBeNull();
    expect(albums.albumsOf(B)).toBeNull();
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

  it('loads many photos in batches of 100, skipping known and running ones', async () => {
    h.albumsWithAsset.mockResolvedValue([]);
    h.albumMemberships.mockImplementation((ids) =>
      Promise.resolve(Object.fromEntries(ids.map((id) => [id, [album('1', 'Bergen')]])))
    );
    const albums = await store();
    await albums.ensure(A);
    const ids = Array.from(
      { length: 150 },
      (_, i) => `${i.toString(16).padStart(8, '0')}-0000-4000-8000-000000000000`
    );
    await albums.ensureMany([A, B, `${B}_2`, ...ids]);
    const batches = h.albumMemberships.mock.calls.map(([batch]) => batch.length);
    expect(batches).toEqual([100, 51]);
    expect(h.albumMemberships.mock.calls[0]?.[0]).toContain(B);
    expect(albums.has(A)).toBe(true);
    expect(albums.albumsOf(`${B}_2`)?.map((a) => a.albumName)).toEqual(['Bergen']);
    await albums.ensureMany([B]);
    expect(h.albumMemberships).toHaveBeenCalledTimes(2);
  });

  it('answers photos missing from a batch with no albums', async () => {
    h.albumMemberships.mockResolvedValue({ [A]: [album('1', 'Bergen')] });
    const albums = await store();
    await albums.ensureMany([A, B]);
    expect(albums.albumsOf(B)).toEqual([]);
  });

  it('leaves a failed batch unloaded so the next pass asks again', async () => {
    h.albumMemberships.mockRejectedValueOnce(new Error('offline')).mockResolvedValueOnce({});
    const albums = await store();
    await albums.ensureMany([A]);
    expect(albums.has(A)).toBe(false);
    expect(albums.errorOf(A)).toBeNull();
    await albums.ensureMany([A]);
    expect(albums.albumsOf(A)).toEqual([]);
  });

  it('drops a batch that an album change overtook', async () => {
    let answer: (found: Record<string, AlbumSummary[]>) => void = () => {};
    h.albumMemberships.mockImplementationOnce(() => new Promise((done) => (answer = done)));
    const albums = await store();
    const running = albums.ensureMany([A, B]);
    albums.forget([A]);
    answer({ [A]: [album('1', 'Old')], [B]: [] });
    await running;
    expect(albums.has(A)).toBe(false);
    expect(albums.albumsOf(B)).toEqual([]);
  });
});
