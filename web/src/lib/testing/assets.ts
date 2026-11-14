import type { AssetSummary } from '#lib/types/album.ts';

export function asset(id: string, overrides: Partial<AssetSummary> = {}): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: [],
    ...overrides
  };
}
