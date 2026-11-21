import { album } from '#lib/testing/albums.ts';
import type { AlbumMember } from '#lib/types/album.ts';
import { describe, expect, it } from 'vitest';
import type { DialogTextPart } from '#lib/browse/dialogText.ts';
import { deleteAlbumText, leaveAlbumText } from './albumDialogText';

const ME = 'user-me';
const ANNE: AlbumMember = { userId: 'user-anne', name: 'Anne Berg', role: 'owner' };
const MINE: AlbumMember = { userId: ME, name: 'Me', role: 'owner' };

function plain(lines: DialogTextPart[][]): string[] {
  return lines.map((line) =>
    line.map((part) => (typeof part === 'string' ? part : `*${part.strong}*`)).join('')
  );
}

describe('deleteAlbumText', () => {
  it('names the album and keeps the photos', () => {
    const text = deleteAlbumText(
      album('a', { albumName: 'Norway', assetCount: 412, ownerId: ME, members: [MINE] }),
      ME
    );
    expect(text.title).toBe('Delete album');
    expect(text.confirm).toBe('Delete album');
    expect(plain(text.lines)).toEqual([
      'Immich deletes the album *Norway*. Its 412 photos stay in your library and in any other albums.',
      'This cannot be undone.'
    ]);
  });

  it('adds the shared consequence and handles one or no photos', () => {
    const shared = album('a', { albumName: 'Family', assetCount: 1, members: [MINE, ANNE] });
    expect(plain(deleteAlbumText(shared, ME).lines)).toEqual([
      'Immich deletes the album *Family*. Its 1 photo stays in your library and in any other albums.',
      'The album is shared with 1 person. It disappears for them too, and its shared links stop working.',
      'This cannot be undone.'
    ]);
    expect(plain(deleteAlbumText(album('a', { albumName: 'Empty' }), ME).lines)[0]).toBe(
      'Immich deletes the album *Empty*. It has no photos.'
    );
  });
});

describe('leaveAlbumText', () => {
  it('names the sharer and never says it cannot be undone', () => {
    const text = leaveAlbumText(
      album('a', { albumName: 'Family', ownerId: ANNE.userId, members: [ANNE] })
    );
    expect(text.title).toBe('Leave album');
    expect(plain(text.lines)).toEqual([
      'You leave *Family*, shared by Anne Berg. It disappears from your albums; Anne and the other members keep it.',
      'Only Anne can invite you back.'
    ]);
  });

  it('falls back to the owner when no name is known', () => {
    expect(plain(leaveAlbumText(album('a', { albumName: 'Trip', ownerId: 'x' })).lines)).toEqual([
      'You leave *Trip*. It disappears from your albums; the owner and the other members keep it.',
      'Only the owner can invite you back.'
    ]);
  });
});
