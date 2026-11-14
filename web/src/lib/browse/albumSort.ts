import type { AlbumSummary } from '#lib/types/album.ts';

export type AlbumSort = 'updated' | 'name' | 'newest' | 'count';

export const ALBUM_SORTS: readonly { value: AlbumSort; label: string }[] = [
  { value: 'updated', label: 'Recently updated' },
  { value: 'name', label: 'Name' },
  { value: 'newest', label: 'Newest photo' },
  { value: 'count', label: 'Photo count' }
];

function byName(a: AlbumSummary, b: AlbumSummary): number {
  return a.albumName.localeCompare(b.albumName);
}

function timeOf(value: string | null): number {
  const time = value ? Date.parse(value) : Number.NaN;
  return Number.isNaN(time) ? Number.NEGATIVE_INFINITY : time;
}

const COMPARE: Readonly<Record<AlbumSort, (a: AlbumSummary, b: AlbumSummary) => number>> = {
  updated: (a, b) => timeOf(b.updatedAt) - timeOf(a.updatedAt) || byName(a, b),
  name: byName,
  newest: (a, b) => timeOf(b.endDate) - timeOf(a.endDate) || byName(a, b),
  count: (a, b) => b.assetCount - a.assetCount || byName(a, b)
};

export function sortAlbums(albums: readonly AlbumSummary[], sort: AlbumSort): AlbumSummary[] {
  return [...albums].sort(COMPARE[sort]);
}
