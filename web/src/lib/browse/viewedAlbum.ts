import { canEditAlbum } from '#lib/browse/albumRole.ts';
import { album } from '#lib/stores/album.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { page } from '$app/state';

export function viewedAlbumId(): string | null {
  return page.route.id === '/albums/[id]' ? (page.params.id ?? null) : null;
}

export function editableViewedAlbum(): AlbumSummary | null {
  const current = album.current;
  if (!current || current.id !== viewedAlbumId()) return null;
  return canEditAlbum(current, session.user?.id ?? null) ? current : null;
}
