import { album as albumStore } from '#lib/stores/album.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { album } from '#lib/testing/albums.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { editableViewedAlbum, viewedAlbumId } from './viewedAlbum';

const route = vi.hoisted(() => ({
  page: {
    route: { id: '/albums/[id]' as string | null },
    params: { id: 'a1' } as Record<string, string>
  }
}));

vi.mock('$app/state', () => route);

beforeEach(() => {
  route.page.route.id = '/albums/[id]';
  route.page.params = { id: 'a1' };
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  albumStore.current = album('a1', { ownerId: 'me' });
});

describe('viewedAlbum', () => {
  it('reads the album id only on an album page', () => {
    expect(viewedAlbumId()).toBe('a1');
    route.page.route.id = '/tags/[id]';
    expect(viewedAlbumId()).toBeNull();
  });

  it('offers the open album to its owner and editors', () => {
    expect(editableViewedAlbum()?.id).toBe('a1');
    albumStore.current = album('a1', {
      ownerId: 'anne',
      members: [
        { userId: 'anne', name: 'Anne', role: 'owner' },
        { userId: 'me', name: 'Me', role: 'editor' }
      ]
    });
    expect(editableViewedAlbum()?.id).toBe('a1');
  });

  it('withholds it from viewers, other pages and a stale album', () => {
    albumStore.current = album('a1', {
      ownerId: 'anne',
      members: [{ userId: 'me', name: 'Me', role: 'viewer' }]
    });
    expect(editableViewedAlbum()).toBeNull();
    albumStore.current = album('a2', { ownerId: 'me' });
    expect(editableViewedAlbum()).toBeNull();
    albumStore.current = album('a1', { ownerId: 'me' });
    route.page.route.id = '/albums';
    expect(editableViewedAlbum()).toBeNull();
  });
});
