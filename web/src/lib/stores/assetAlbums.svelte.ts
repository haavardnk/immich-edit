import { albumsWithAsset } from '#lib/api/albums.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { putBounded } from '#lib/utils/boundedRecord.ts';
import { errorMessage } from '#lib/utils/errors.ts';

const KEEP = 40;

type Entry = { albums: AlbumSummary[] } | { error: string };

class AssetAlbumsStore {
  #entries = $state.raw<Record<string, Entry>>({});
  #order: string[] = [];
  #pending = new Map<string, Promise<void>>();
  #generations = new Map<string, number>();

  albumsOf(id: string): AlbumSummary[] | null {
    const entry = this.#entries[sourceId(id)];
    return entry && 'albums' in entry ? entry.albums : null;
  }

  errorOf(id: string): string | null {
    const entry = this.#entries[sourceId(id)];
    return entry && 'error' in entry ? entry.error : null;
  }

  ensure(id: string): Promise<void> {
    const key = sourceId(id);
    if (this.#entries[key]) return Promise.resolve();
    const running = this.#pending.get(key);
    if (running) return running;
    const generation = this.#generations.get(key) ?? 0;
    const load = albumsWithAsset(key)
      .then(
        (albums) => this.#settle(key, generation, { albums: byName(albums) }),
        (error: unknown) => this.#settle(key, generation, { error: errorMessage(error) })
      )
      .finally(() => {
        if (this.#pending.get(key) === load) this.#pending.delete(key);
      });
    this.#pending.set(key, load);
    return load;
  }

  retry(id: string): Promise<void> {
    this.forget([id]);
    return this.ensure(id);
  }

  forget(ids: readonly string[]): void {
    const keys = new Set(ids.map(sourceId));
    if (keys.size === 0) return;
    const entries = { ...this.#entries };
    for (const key of keys) {
      this.#generations.set(key, (this.#generations.get(key) ?? 0) + 1);
      this.#pending.delete(key);
      delete entries[key];
    }
    this.#entries = entries;
    this.#order = this.#order.filter((key) => !keys.has(key));
  }

  #settle(key: string, generation: number, entry: Entry): void {
    if ((this.#generations.get(key) ?? 0) !== generation) return;
    const next = putBounded(this.#entries, this.#order, key, entry, KEEP);
    this.#entries = next.record;
    this.#order = next.order;
  }
}

function byName(albums: AlbumSummary[]): AlbumSummary[] {
  return [...albums].sort((a, b) => a.albumName.localeCompare(b.albumName));
}

export const assetAlbums = new AssetAlbumsStore();
