import { deleteAlbum, getAlbum, leaveAlbum, updateAlbum } from '#lib/api/albums.ts';
import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
import { library } from '#lib/stores/library.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AlbumPatch, AlbumSummary } from '#lib/types/album.ts';
import { errorMessage } from '#lib/utils/errors.ts';

class AlbumStore {
  current = $state<AlbumSummary | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  #requested: string | null = null;

  async load(id: string): Promise<void> {
    if (this.current?.id === id) return;
    this.#requested = id;
    this.current = null;
    this.loading = true;
    this.error = null;
    try {
      const loaded = await getAlbum(id);
      if (this.#requested === id) this.current = loaded;
    } catch (e) {
      if (this.#requested === id) this.error = errorMessage(e);
    } finally {
      if (this.#requested === id) this.loading = false;
    }
  }

  update = async (id: string, patch: AlbumPatch): Promise<AlbumSummary | null> => {
    if (!(await metadataConsent.gate())) return null;
    try {
      const updated = await updateAlbum(id, patch);
      library.putAlbum(updated);
      if (this.current?.id === id) this.current = updated;
      if (patch.albumName !== undefined) assetAlbums.clear();
      return updated;
    } catch (e) {
      toasts.fail('Save album details', e);
      return null;
    }
  };

  remove = (album: AlbumSummary): Promise<boolean> => this.drop(album, deleteAlbum, 'Delete album');

  leave = (album: AlbumSummary): Promise<boolean> => this.drop(album, leaveAlbum, 'Leave album');

  private async drop(
    album: AlbumSummary,
    send: (id: string) => Promise<void>,
    failure: string
  ): Promise<boolean> {
    if (!(await metadataConsent.gate())) return false;
    try {
      await send(album.id);
    } catch (e) {
      toasts.fail(failure, e);
      return false;
    }
    library.dropAlbum(album.id);
    if (this.current?.id === album.id) this.current = null;
    assetAlbums.clear();
    libraryChanged();
    return true;
  }
}

export const album = new AlbumStore();
