import type { AlbumSummary } from '#lib/types/album.ts';

export type AlbumScope = 'all' | 'mine' | 'shared';

export const ALBUM_SCOPES: readonly { value: AlbumScope; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'mine', label: 'Mine' },
  { value: 'shared', label: 'Shared with me' }
];

function sharedWithMe(album: AlbumSummary, userId: string | null): boolean {
  return album.ownerId !== null && album.ownerId !== userId;
}

export function albumsInScope(
  albums: readonly AlbumSummary[],
  scope: AlbumScope,
  userId: string | null
): readonly AlbumSummary[] {
  if (scope === 'all') return albums;
  if (userId === null) return [];
  const wanted = scope === 'shared';
  return albums.filter((album) => sharedWithMe(album, userId) === wanted);
}
