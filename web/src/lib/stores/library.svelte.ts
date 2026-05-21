import { listAlbums } from '$lib/api/albums';
import type { AlbumSummary } from '$lib/types/album';

class LibraryStore {
  albums = $state<AlbumSummary[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);

  async load(force = false): Promise<void> {
    if (!force && (this.albums.length > 0 || this.loading)) return;
    this.loading = true;
    this.error = null;
    try {
      this.albums = await listAlbums();
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.loading = false;
    }
  }
}

export const library = new LibraryStore();
