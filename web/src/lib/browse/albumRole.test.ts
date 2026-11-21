import { album } from '#lib/testing/albums.ts';
import type { AlbumMember } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';
import { albumOwner, albumRole, canEditAlbum, otherMembers } from './albumRole';

const ME = 'user-me';
const ANNE: AlbumMember = { userId: 'user-anne', name: 'Anne Berg', role: 'owner' };

describe('albumRole', () => {
  it('reads the caller from the member list', () => {
    const shared = album('a', {
      ownerId: ANNE.userId,
      members: [ANNE, { userId: ME, name: 'Me', role: 'editor' }]
    });
    expect(albumRole(shared, ME)).toBe('editor');
    expect(canEditAlbum(shared, ME)).toBe(true);
  });

  it('treats an album the caller owns, or without an owner, as owned', () => {
    expect(albumRole(album('a', { ownerId: ME }), ME)).toBe('owner');
    expect(albumRole(album('a'), ME)).toBe('owner');
  });

  it('falls back to viewer for someone else’s album', () => {
    const shared = album('a', { ownerId: ANNE.userId, members: [ANNE] });
    expect(albumRole(shared, ME)).toBe('viewer');
    expect(canEditAlbum(shared, ME)).toBe(false);
    expect(albumRole(shared, null)).toBe('viewer');
  });
});

describe('albumOwner and otherMembers', () => {
  it('finds the owner by role, then by owner id', () => {
    expect(albumOwner(album('a', { members: [ANNE] }))).toEqual(ANNE);
    const unroled = { ...ANNE, role: 'editor' as const };
    expect(albumOwner(album('a', { ownerId: ANNE.userId, members: [unroled] }))).toEqual(unroled);
    expect(albumOwner(album('a'))).toBeNull();
  });

  it('counts members other than the caller', () => {
    const shared = album('a', {
      members: [ANNE, { userId: ME, name: 'Me', role: 'viewer' }]
    });
    expect(otherMembers(shared, ME)).toBe(1);
    expect(otherMembers(shared, null)).toBe(2);
  });
});
