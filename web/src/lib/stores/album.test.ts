import { album as albumFixture } from '#lib/testing/albums.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  getAlbum: vi.fn<(id: string) => Promise<AlbumSummary>>(),
  updateAlbum: vi.fn<(id: string, patch: unknown) => Promise<AlbumSummary>>(),
  deleteAlbum: vi.fn<(id: string) => Promise<void>>(),
  leaveAlbum: vi.fn<(id: string) => Promise<void>>(),
  clear: vi.fn()
}));

vi.mock('#lib/api/albums.ts', () => ({
  getAlbum: h.getAlbum,
  updateAlbum: h.updateAlbum,
  deleteAlbum: h.deleteAlbum,
  leaveAlbum: h.leaveAlbum
}));
vi.mock('#lib/stores/assetAlbums.svelte.ts', () => ({ assetAlbums: { clear: h.clear } }));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({
  metadataConsent: { gate: vi.fn(async () => true) }
}));

const NORWAY = albumFixture('norway', { albumName: 'Norway' });
const FAMILY = albumFixture('family', { albumName: 'Family' });

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
});

async function store() {
  return (await import('#lib/stores/album.svelte.ts')).album;
}

describe('album store', () => {
  it('keeps the album last asked for when an earlier load answers late', async () => {
    let answer: (album: AlbumSummary) => void = () => {};
    h.getAlbum
      .mockImplementationOnce(() => new Promise((done) => (answer = done)))
      .mockResolvedValueOnce(FAMILY);
    const album = await store();
    const stale = album.load(NORWAY.id);
    await album.load(FAMILY.id);
    answer(NORWAY);
    await stale;
    expect(album.current?.id).toBe(FAMILY.id);
    expect(album.loading).toBe(false);
  });

  it('refreshes the per-photo albums after a rename, delete or leave', async () => {
    h.updateAlbum.mockResolvedValue(NORWAY);
    const album = await store();
    await album.update(NORWAY.id, { description: 'Coast' });
    expect(h.clear).not.toHaveBeenCalled();
    await album.update(NORWAY.id, { albumName: 'Norway' });
    expect(await album.remove(NORWAY)).toBe(true);
    expect(await album.leave(FAMILY)).toBe(true);
    expect(h.clear).toHaveBeenCalledTimes(3);
  });
});
