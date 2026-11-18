import { describe, expect, it } from 'vitest';
import type { AlbumSummary } from '#lib/types/album.ts';
import { ALBUM_SORTS, sortAlbums } from './albumSort';

function album(
  albumName: string,
  assetCount: number,
  updatedAt: string | null,
  endDate: string | null
): AlbumSummary {
  return {
    id: albumName,
    albumName,
    assetCount,
    albumThumbnailAssetId: null,
    updatedAt,
    ownerId: null,
    shared: false,
    endDate
  };
}

const ALBUMS = [
  album('Lofoten', 186, '2026-03-01T00:00:00Z', '2025-02-10T00:00:00Z'),
  album('Family', 1024, null, '2026-07-14T00:00:00Z'),
  album('Norway', 412, '2026-05-01T00:00:00Z', '2025-02-10T00:00:00Z'),
  album('Archive', 186, null, null)
];

function names(list: AlbumSummary[]): string[] {
  return list.map((entry) => entry.albumName);
}

describe('sortAlbums', () => {
  it('puts the most recently updated first and undated albums last by name', () => {
    expect(names(sortAlbums(ALBUMS, 'updated'))).toEqual([
      'Norway',
      'Lofoten',
      'Archive',
      'Family'
    ]);
  });

  it('sorts by name', () => {
    expect(names(sortAlbums(ALBUMS, 'name'))).toEqual(['Archive', 'Family', 'Lofoten', 'Norway']);
  });

  it('puts the album with the newest photo first and empty albums last', () => {
    expect(names(sortAlbums(ALBUMS, 'newest'))).toEqual(['Family', 'Lofoten', 'Norway', 'Archive']);
  });

  it('offers the sorts in the order of the sort sheet', () => {
    expect(ALBUM_SORTS.map((option) => option.label)).toEqual([
      'Recently updated',
      'Name',
      'Newest photo',
      'Photo count'
    ]);
  });

  it('sorts by photo count, largest first, then by name', () => {
    expect(names(sortAlbums(ALBUMS, 'count'))).toEqual(['Family', 'Norway', 'Archive', 'Lofoten']);
  });

  it('leaves the source list alone', () => {
    sortAlbums(ALBUMS, 'name');
    expect(names(ALBUMS)).toEqual(['Lofoten', 'Family', 'Norway', 'Archive']);
  });
});
