import { describe, expect, it } from 'vitest';
import type { AlbumSummary } from '#lib/types/album.ts';
import { ALBUM_SCOPES, albumsInScope } from './albumScope';

const ME = 'user-me';

function album(albumName: string, ownerId: string | null, shared: boolean): AlbumSummary {
  return {
    id: albumName,
    albumName,
    assetCount: 1,
    albumThumbnailAssetId: null,
    updatedAt: null,
    ownerId,
    shared,
    endDate: null
  };
}

const ALBUMS = [
  album('Norway', ME, false),
  album('Family', 'user-anne', true),
  album('Portfolio', ME, true),
  album('Unknown owner', null, false)
];

function names(list: readonly AlbumSummary[]): string[] {
  return list.map((entry) => entry.albumName);
}

describe('albumsInScope', () => {
  it('keeps every album for All', () => {
    expect(albumsInScope(ALBUMS, 'all', ME)).toBe(ALBUMS);
  });

  it('keeps my albums for Mine, shared or not, and an album without a known owner', () => {
    expect(names(albumsInScope(ALBUMS, 'mine', ME))).toEqual([
      'Norway',
      'Portfolio',
      'Unknown owner'
    ]);
  });

  it('keeps albums another user owns for Shared with me', () => {
    expect(names(albumsInScope(ALBUMS, 'shared', ME))).toEqual(['Family']);
  });

  it('claims no album as mine or shared while the signed-in user is unknown', () => {
    expect(albumsInScope(ALBUMS, 'all', null)).toBe(ALBUMS);
    expect(albumsInScope(ALBUMS, 'mine', null)).toEqual([]);
    expect(albumsInScope(ALBUMS, 'shared', null)).toEqual([]);
  });

  it('labels the chips like the albums tab', () => {
    expect(ALBUM_SCOPES.map((scope) => scope.label)).toEqual(['All', 'Mine', 'Shared with me']);
  });
});
