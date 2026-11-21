import { album } from '#lib/testing/albums.ts';
import type { AlbumMember } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';
import {
  albumCaption,
  albumDateRange,
  albumPhotos,
  ownerName,
  roleLine,
  sharingNote
} from './albumSharing';

const ME = 'user-me';
const ANNE: AlbumMember = { userId: 'user-anne', name: 'Anne Berg', role: 'owner' };
const OLE: AlbumMember = { userId: 'user-ole', name: 'Ole', role: 'viewer' };
const MINE: AlbumMember = { userId: ME, name: 'Me', role: 'owner' };

describe('albumPhotos', () => {
  it('groups thousands and picks the singular', () => {
    expect(albumPhotos(1)).toBe('1 photo');
    expect(albumPhotos(1024)).toBe('1,024 photos');
  });
});

describe('sharing text', () => {
  it('describes a private album', () => {
    const own = album('a', { ownerId: ME, assetCount: 412, members: [MINE] });
    expect(sharingNote(own, ME)).toBe('only you');
    expect(albumCaption(own, ME)).toBe('412 photos · only you');
    expect(roleLine(own, ME)).toBeNull();
  });

  it('counts the people an owner shares with', () => {
    const own = album('a', { ownerId: ME, members: [MINE, OLE] });
    expect(sharingNote(own, ME)).toBe('shared with 1 person');
    expect(roleLine(own, ME)).toBe('Shared with 1 person');
  });

  it('names the sharer and the caller’s role', () => {
    const editor = album('a', {
      ownerId: ANNE.userId,
      members: [ANNE, { userId: ME, name: 'Me', role: 'editor' }]
    });
    expect(ownerName(editor)).toBe('Anne Berg');
    expect(sharingNote(editor, ME)).toBe('shared by Anne Berg');
    expect(roleLine(editor, ME)).toBe('Shared by Anne Berg · you can edit');
    const viewer = album('a', { ownerId: ANNE.userId, members: [ANNE, OLE] });
    expect(roleLine(viewer, OLE.userId)).toBe('Shared by Anne Berg · you can view');
  });

  it('falls back when the owner has no name', () => {
    const viewer = album('a', { ownerId: 'user-x', members: [{ ...ANNE, name: ' ' }] });
    expect(ownerName(viewer)).toBeNull();
    expect(sharingNote(viewer, ME)).toBe('shared by another Immich user');
  });
});

describe('albumDateRange', () => {
  it('formats a range, a single date and nothing', () => {
    const range = albumDateRange(
      album('a', { startDate: '2026-10-12T08:00:00Z', endDate: '2026-11-03T18:00:00Z' })
    );
    expect(range?.replace(/\s/g, ' ')).toBe('12 Oct – 3 Nov 2026');
    expect(albumDateRange(album('a', { endDate: '2026-11-03T18:00:00Z' }))).toBe('3 Nov 2026');
    expect(albumDateRange(album('a'))).toBeNull();
  });

  it('orders a reversed range', () => {
    const range = albumDateRange(
      album('a', { startDate: '2026-11-03T00:00:00Z', endDate: '2026-10-12T00:00:00Z' })
    );
    expect(range?.replace(/\s/g, ' ')).toBe('12 Oct – 3 Nov 2026');
  });
});
