import { getAlbum } from '$lib/api/albums';
import type { AlbumDetail } from '$lib/types/album';

class AlbumStore {
  current = $state<AlbumDetail | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async load(id: string): Promise<void> {
    if (this.current?.id === id) return;
    this.current = null;
    this.loading = true;
    this.error = null;
    try {
      this.current = await getAlbum(id);
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.loading = false;
    }
  }
}

export const album = new AlbumStore();
