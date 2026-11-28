import { albumMemberships, albumsWithAsset } from '#lib/api/albums.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { putAllBounded } from '#lib/utils/boundedRecord.ts';
import { errorMessage } from '#lib/utils/errors.ts';

const KEEP = 1000;
const BATCH = 100;

type Entry = { albums: AlbumSummary[] } | { error: string };

class AssetAlbumsStore {
  #entries = $state.raw<Record<string, Entry>>({});
  #order: string[] = [];
  #pending = new Map<string, Promise<void>>();
  #generations = new Map<string, number>();

  has(id: string): boolean {
    return this.#entries[sourceId(id)] !== undefined;
  }

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
        (albums) => this.#settle(new Map([[key, generation]]), () => ({ albums: byName(albums) })),
        (error: unknown) =>
          this.#settle(new Map([[key, generation]]), () => ({ error: errorMessage(error) }))
      )
      .finally(() => {
        if (this.#pending.get(key) === load) this.#pending.delete(key);
      });
    this.#pending.set(key, load);
    return load;
  }

  ensureMany(ids: readonly string[]): Promise<void> {
    const keys = [...new Set(ids.map(sourceId))].filter(
      (key) => !this.#entries[key] && !this.#pending.has(key)
    );
    const loads: Promise<void>[] = [];
    for (let start = 0; start < keys.length; start += BATCH) {
      loads.push(this.#loadBatch(keys.slice(start, start + BATCH)));
    }
    return Promise.all(loads).then(() => undefined);
  }

  retry(id: string): Promise<void> {
    this.forget([id]);
    return this.ensure(id);
  }

  clear(): void {
    this.forget([...Object.keys(this.#entries), ...this.#pending.keys()]);
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

  #loadBatch(keys: readonly string[]): Promise<void> {
    const generations = new Map(keys.map((key) => [key, this.#generations.get(key) ?? 0]));
    const load = albumMemberships(keys)
      .then(
        (found) => this.#settle(generations, (key) => ({ albums: byName(found[key] ?? []) })),
        () => undefined
      )
      .finally(() => {
        for (const key of keys) if (this.#pending.get(key) === load) this.#pending.delete(key);
      });
    for (const key of keys) this.#pending.set(key, load);
    return load;
  }

  #settle(generations: ReadonlyMap<string, number>, entryOf: (key: string) => Entry): void {
    const current = [...generations].filter(
      ([key, generation]) => (this.#generations.get(key) ?? 0) === generation
    );
    if (current.length === 0) return;
    const entries = current.map(([key]) => [key, entryOf(key)] as const);
    const next = putAllBounded(this.#entries, this.#order, entries, KEEP);
    this.#entries = next.record;
    this.#order = next.order;
  }
}

function byName(albums: AlbumSummary[]): AlbumSummary[] {
  return [...albums].sort((a, b) => a.albumName.localeCompare(b.albumName));
}

export const assetAlbums = new AssetAlbumsStore();
