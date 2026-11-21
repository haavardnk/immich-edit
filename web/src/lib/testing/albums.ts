import type { AlbumSummary } from '#lib/types/album.ts';

export function album(id: string, overrides: Partial<AlbumSummary> = {}): AlbumSummary {
  return {
    id,
    albumName: id,
    description: '',
    assetCount: 0,
    albumThumbnailAssetId: null,
    updatedAt: null,
    ownerId: null,
    shared: false,
    startDate: null,
    endDate: null,
    members: [],
    ...overrides
  };
}
