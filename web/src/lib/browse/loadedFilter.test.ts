import { describe, expect, it } from 'vitest';
import { keepsLoaded } from './loadedFilter';
import type { AssetSummary } from '$lib/types/album';
import type { ExifInfo } from '$lib/types/asset';
import type { BrowseFilter } from '$lib/types/search';

const photo: AssetSummary = {
  id: 'a',
  originalFileName: 'Café_0042.ARW',
  type: 'IMAGE',
  fileCreatedAt: '2024-06-15T12:00:00Z',
  createdAt: '2025-01-10T08:00:00Z',
  updatedAt: null,
  checksum: null,
  isFavorite: false,
  exifInfo: { fileSizeInByte: 3_000_000 } as ExifInfo,
  tags: []
};
const bare: AssetSummary = { ...photo, fileCreatedAt: null, createdAt: null, exifInfo: null };

describe('keepsLoaded', () => {
  it.each([
    [{}, photo, true],
    [{ imagesOnly: true, visibility: 'archive' }, photo, true],
    [{ favorite: true }, photo, false],
    [{ favorite: true }, { ...photo, isFavorite: true }, true],
    [{ filename: ' cafe_00 ' }, photo, true],
    [{ filename: 'CAFÉ' }, photo, true],
    [{ filename: 'img' }, photo, false],
    [{ takenAfter: '2024-06-15T12:00:00Z' }, photo, true],
    [{ takenAfter: '2024-06-16T00:00:00Z' }, photo, false],
    [{ takenBefore: '2024-06-15T11:59:59Z' }, photo, false],
    [{ takenBefore: '2024-06-15T12:00:00Z' }, bare, false],
    [{ addedAfter: '2025-01-10T08:00:00Z' }, photo, true],
    [{ addedBefore: '2025-01-10T07:59:59Z' }, photo, false],
    [{ addedAfter: '2024-01-01T00:00:00Z' }, bare, false],
    [{ minSize: 3_000_000, maxSize: 3_000_000 }, photo, true],
    [{ minSize: 3_000_001 }, photo, false],
    [{ maxSize: 2_999_999 }, photo, false],
    [{ maxSize: 5_000_000 }, bare, false]
  ] as [BrowseFilter, AssetSummary, boolean][])('keeps %j on %j: %s', (filter, asset, kept) => {
    expect(keepsLoaded(filter, asset)).toBe(kept);
  });
});
