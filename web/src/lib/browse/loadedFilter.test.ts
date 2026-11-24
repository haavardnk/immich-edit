import { describe, expect, it } from 'vitest';
import { keepsLoaded } from './loadedFilter';
import type { AssetSummary } from '#lib/types/album.ts';
import type { ExifInfo } from '#lib/types/asset.ts';
import type { BrowseFilter } from '#lib/types/search.ts';

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
const wallClocked: AssetSummary = { ...photo, localDateTime: '2024-06-15T12:00:00.000Z' };
const eastOfUtc: AssetSummary = { ...photo, localDateTime: '2024-06-16T01:00:00.000Z' };

describe('keepsLoaded', () => {
  it.each([
    [{}, photo, true],
    [{ imagesOnly: true, visibility: 'archive' }, photo, true],
    [{ favorite: true }, photo, false],
    [{ favorite: true }, { ...photo, isFavorite: true }, true],
    [{ filename: ' cafe_00 ' }, photo, true],
    [{ filename: 'CAFÉ' }, photo, true],
    [{ filename: 'img' }, photo, false],
    [{ takenLocal: { from: '2024-06-15T12:00:00' } }, wallClocked, true],
    [{ takenLocal: { from: '2024-06-16T00:00:00' } }, wallClocked, false],
    [{ takenLocal: { until: '2024-06-15T12:00:00' } }, wallClocked, false],
    [{ takenLocal: { until: '2024-06-15T12:01:00' } }, wallClocked, true],
    [{ takenLocal: { until: '2024-06-16T00:00:00' } }, bare, false],
    [
      { takenLocal: { from: '2024-06-15T00:00:00', until: '2024-06-16T00:00:00' } },
      eastOfUtc,
      false
    ],
    [
      { takenLocal: { from: '2024-06-16T00:00:00', until: '2024-06-17T00:00:00' } },
      eastOfUtc,
      true
    ],
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
