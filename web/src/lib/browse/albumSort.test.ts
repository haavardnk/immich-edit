import { describe, expect, it } from 'vitest';
import type { AlbumSummary } from '#lib/types/album.ts';
import { sortAlbums } from './albumSort';

function album(albumName: string, assetCount: number, updatedAt: string | null): AlbumSummary {
  return { id: albumName, albumName, assetCount, albumThumbnailAssetId: null, updatedAt };
}

const ALBUMS = [
  album('Lofoten', 186, '2026-03-01T00:00:00Z'),
  album('Family', 1024, null),
  album('Norway', 412, '2026-05-01T00:00:00Z'),
  album('Archive', 186, null)
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

  it('sorts by photo count, largest first, then by name', () => {
    expect(names(sortAlbums(ALBUMS, 'count'))).toEqual(['Family', 'Norway', 'Archive', 'Lofoten']);
  });

  it('leaves the source list alone', () => {
    sortAlbums(ALBUMS, 'name');
    expect(names(ALBUMS)).toEqual(['Lofoten', 'Family', 'Norway', 'Archive']);
  });
});
