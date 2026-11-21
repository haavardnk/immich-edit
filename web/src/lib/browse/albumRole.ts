import type { AlbumMember, AlbumRole, AlbumSummary } from '#lib/types/album.ts';

export function albumRole(album: AlbumSummary, userId: string | null): AlbumRole {
  const own = album.members.find((member) => member.userId === userId);
  if (own) return own.role;
  return album.ownerId === null || album.ownerId === userId ? 'owner' : 'viewer';
}

export function albumOwner(album: AlbumSummary): AlbumMember | null {
  return (
    album.members.find((member) => member.role === 'owner') ??
    album.members.find((member) => member.userId === album.ownerId) ??
    null
  );
}

export function otherMembers(album: AlbumSummary, userId: string | null): number {
  return album.members.filter((member) => member.userId !== userId).length;
}

export function canEditAlbum(album: AlbumSummary, userId: string | null): boolean {
  return albumRole(album, userId) !== 'viewer';
}
