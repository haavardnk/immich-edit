import { album as albumStore } from '#lib/stores/album.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { album } from '#lib/testing/albums.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { canSetViewedAlbumCover, setViewedAlbumCover } from './albumCover';

const COVER = '00000000-0000-4000-8000-000000000001';
const OTHER = '00000000-0000-4000-8000-000000000002';

const route = vi.hoisted(() => ({
  page: {
    route: { id: '/albums/[id]' as string | null },
    params: { id: 'a1' } as Record<string, string>
  }
}));

vi.mock('$app/state', () => route);
vi.mock('#lib/stores/album.svelte.ts', () => ({
  album: { current: null, update: vi.fn() }
}));
vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts: { push: vi.fn() } }));

beforeEach(() => {
  vi.mocked(albumStore.update).mockReset();
  vi.mocked(toasts.push).mockReset();
  route.page.route.id = '/albums/[id]';
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  albumStore.current = album('a1', {
    albumName: 'Norway',
    ownerId: 'me',
    albumThumbnailAssetId: COVER
  });
});

describe('albumCover', () => {
  it('offers every photo except the current cover, copies included', () => {
    expect(canSetViewedAlbumCover(OTHER)).toBe(true);
    expect(canSetViewedAlbumCover(COVER)).toBe(false);
    expect(canSetViewedAlbumCover(`${COVER}_1`)).toBe(false);
    expect(canSetViewedAlbumCover(null)).toBe(false);
  });

  it('withholds it from viewers and outside the album page', () => {
    route.page.route.id = '/photos';
    expect(canSetViewedAlbumCover(OTHER)).toBe(false);
    route.page.route.id = '/albums/[id]';
    albumStore.current = album('a1', {
      ownerId: 'anne',
      members: [{ userId: 'me', name: 'Me', role: 'viewer' }]
    });
    expect(canSetViewedAlbumCover(OTHER)).toBe(false);
  });

  it('sends the photo and names the album in the toast', async () => {
    vi.mocked(albumStore.update).mockResolvedValue(
      album('a1', { albumName: 'Norway', albumThumbnailAssetId: OTHER })
    );
    await expect(setViewedAlbumCover(OTHER)).resolves.toBe(true);
    expect(albumStore.update).toHaveBeenCalledWith(
      'a1',
      { albumThumbnailAssetId: OTHER },
      'Set album cover'
    );
    expect(toasts.push).toHaveBeenCalledWith('success', 'Set as cover of Norway');
  });

  it('stays quiet when the update is declined or fails', async () => {
    vi.mocked(albumStore.update).mockResolvedValue(null);
    await expect(setViewedAlbumCover(OTHER)).resolves.toBe(false);
    expect(toasts.push).not.toHaveBeenCalled();
  });
});
