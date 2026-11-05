import type { AlbumSummary } from '#lib/types/album.ts';

export type AlbumSort = 'updated' | 'name' | 'count';

export const ALBUM_SORTS: readonly { value: AlbumSort; label: string }[] = [
  { value: 'updated', label: 'Recently updated' },
  { value: 'name', label: 'Name' },
  { value: 'count', label: 'Photo count' }
];

function byName(a: AlbumSummary, b: AlbumSummary): number {
  return a.albumName.localeCompare(b.albumName);
}

function updatedTime(album: AlbumSummary): number {
  const time = album.updatedAt ? Date.parse(album.updatedAt) : Number.NaN;
  return Number.isNaN(time) ? Number.NEGATIVE_INFINITY : time;
}

const COMPARE: Readonly<Record<AlbumSort, (a: AlbumSummary, b: AlbumSummary) => number>> = {
  updated: (a, b) => updatedTime(b) - updatedTime(a) || byName(a, b),
  name: byName,
  count: (a, b) => b.assetCount - a.assetCount || byName(a, b)
};

export function sortAlbums(albums: readonly AlbumSummary[], sort: AlbumSort): AlbumSummary[] {
  return [...albums].sort(COMPARE[sort]);
}
