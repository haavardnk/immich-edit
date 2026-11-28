import type { AlbumSummary } from '#lib/types/album.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { TileOverlay } from './tileOverlay';

const h = vi.hoisted(() => ({
  albumMemberships: vi.fn<(ids: readonly string[]) => Promise<Record<string, AlbumSummary[]>>>()
}));

vi.mock('#lib/api/albums.ts', () => ({ albumMemberships: h.albumMemberships }));

const A = '0f8fad5b-d9cb-469f-a165-70867728950e';
const B = '7c9e6679-7425-40de-944b-e07fc1f90ae7';

const BOTH: TileOverlay = { icons: ['albums'], details: ['tags'] };

beforeEach(() => {
  vi.resetModules();
  h.albumMemberships.mockReset();
});

async function load() {
  const membership = await import('./overlayMembership.svelte');
  const { assetAlbums } = await import('#lib/stores/assetAlbums.svelte.ts');
  const { tagCache } = await import('#lib/stores/tagCache.svelte.ts');
  return { ...membership, assetAlbums, tagCache };
}

describe('missing overlay membership', () => {
  it('asks for nothing in hover only mode or when neither kind is shown', async () => {
    const { missingMembership } = await load();
    expect(missingMembership({ info: 'none', overlay: BOTH, ids: [A] })).toEqual({
      albums: [],
      tags: []
    });
    expect(
      missingMembership({ info: 'full', overlay: { icons: ['favorite'], details: [] }, ids: [A] })
    ).toEqual({ albums: [], tags: [] });
  });

  it('lists each unknown source photo once per wanted kind', async () => {
    const { missingMembership, assetAlbums, tagCache } = await load();
    h.albumMemberships.mockResolvedValue({ [A]: [] });
    await assetAlbums.ensureMany([A]);
    tagCache.set(B, []);
    expect(missingMembership({ info: 'icons', overlay: BOTH, ids: [A, `${A}_2`, B] })).toEqual({
      albums: [B],
      tags: [A]
    });
    expect(
      missingMembership({ info: 'icons', overlay: { icons: ['tags'], details: [] }, ids: [B] })
    ).toEqual({ albums: [], tags: [] });
  });

  it('caps the photos it asks for', async () => {
    const { missingMembership, MAX_VISIBLE } = await load();
    const ids = Array.from({ length: MAX_VISIBLE + 20 }, (_, i) => `id-${i}`);
    expect(missingMembership({ info: 'full', overlay: BOTH, ids }).albums).toHaveLength(
      MAX_VISIBLE
    );
  });
});
