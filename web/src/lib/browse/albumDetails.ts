import type { AlbumPatch, AlbumSummary } from '#lib/types/album.ts';
import { charCount } from '#lib/utils/charCount.ts';
import { displayNameTooLong } from '#lib/utils/displayName.ts';

export const ALBUM_DESCRIPTION_MAX = 2000;

export function validAlbumDetails(name: string, description: string): boolean {
  const trimmed = name.trim();
  return (
    trimmed.length > 0 &&
    !displayNameTooLong(trimmed) &&
    charCount(description.trim()) <= ALBUM_DESCRIPTION_MAX
  );
}

export function albumDetailsPatch(
  album: AlbumSummary,
  name: string,
  description: string
): AlbumPatch | null {
  const patch: AlbumPatch = {};
  const albumName = name.trim();
  const text = description.trim();
  if (albumName !== album.albumName) patch.albumName = albumName;
  if (text !== album.description.trim()) patch.description = text;
  return Object.keys(patch).length > 0 ? patch : null;
}
