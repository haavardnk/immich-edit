import { albumOwner, albumRole, otherMembers } from '#lib/browse/albumRole.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

const DATES = new Intl.DateTimeFormat('en-GB', {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
  timeZone: 'UTC'
});

export function ownerName(album: AlbumSummary): string | null {
  return albumOwner(album)?.name.trim() || null;
}

function sharer(album: AlbumSummary): string {
  return ownerName(album) ?? 'another Immich user';
}

export function albumPhotos(count: number): string {
  return `${count.toLocaleString('en-GB')} ${count === 1 ? 'photo' : 'photos'}`;
}

export function sharingNote(album: AlbumSummary, userId: string | null): string {
  if (albumRole(album, userId) !== 'owner') return `shared by ${sharer(album)}`;
  const others = otherMembers(album, userId);
  return others === 0 ? 'only you' : `shared with ${countOf(others, 'person', 'people')}`;
}

export function albumCaption(album: AlbumSummary, userId: string | null): string {
  return `${albumPhotos(album.assetCount)} · ${sharingNote(album, userId)}`;
}

export function roleLine(album: AlbumSummary, userId: string | null): string | null {
  const role = albumRole(album, userId);
  if (role !== 'owner') {
    return `Shared by ${sharer(album)} · you can ${role === 'editor' ? 'edit' : 'view'}`;
  }
  const others = otherMembers(album, userId);
  return others === 0 ? null : `Shared with ${countOf(others, 'person', 'people')}`;
}

function dateOf(value: string | null): Date | null {
  if (!value) return null;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}

export function albumDateRange(album: AlbumSummary): string | null {
  const start = dateOf(album.startDate);
  const end = dateOf(album.endDate);
  if (!start || !end) {
    const only = start ?? end;
    return only ? DATES.format(only) : null;
  }
  return start <= end ? DATES.formatRange(start, end) : DATES.formatRange(end, start);
}
