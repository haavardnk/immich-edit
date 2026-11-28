import { sourceId } from '#lib/browse/assetKey.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { putAllBounded } from '#lib/utils/boundedRecord.ts';

const MAX_CACHED = 1000;
const BATCH = 100;

type TagChange = (tags: TagRef[]) => TagRef[];

type BatchFetch = (ids: readonly string[]) => Promise<Record<string, readonly TagRef[]>>;

class TagCache {
  #cache = $state.raw<Record<string, TagRef[]>>({});
  #order: string[] = [];
  #loading = new Map<string, TagChange[]>();
  #epoch = 0;

  get(id: string): TagRef[] | undefined {
    return this.#cache[sourceId(id)];
  }

  set(id: string, tags: TagRef[]): void {
    this.#store([[sourceId(id), tags]]);
  }

  async load(id: string, fetch: () => Promise<TagRef[]>): Promise<void> {
    const key = sourceId(id);
    if (this.#cache[key] || this.#loading.has(key)) return;
    await this.#run([key], async () => ({ [key]: await fetch() }));
  }

  async loadMany(ids: readonly string[], fetch: BatchFetch): Promise<void> {
    const keys = [...new Set(ids.map(sourceId))].filter(
      (key) => !this.#cache[key] && !this.#loading.has(key)
    );
    const runs: Promise<void>[] = [];
    for (let start = 0; start < keys.length; start += BATCH) {
      const batch = keys.slice(start, start + BATCH);
      runs.push(this.#run(batch, () => fetch(batch)).catch(() => undefined));
    }
    await Promise.all(runs);
  }

  clear(): void {
    this.#epoch += 1;
    this.#loading.clear();
    this.#cache = {};
    this.#order = [];
  }

  update(id: string, change: TagChange): void {
    const key = sourceId(id);
    const tags = this.#cache[key];
    if (tags) this.#store([[key, change(tags)]]);
    else this.#loading.get(key)?.push(change);
  }

  async #run(keys: readonly string[], fetch: BatchFetch): Promise<void> {
    const epoch = this.#epoch;
    const queues = new Map(keys.map((key) => [key, [] as TagChange[]]));
    for (const [key, changes] of queues) this.#loading.set(key, changes);
    try {
      const fetched = await fetch(keys);
      if (epoch !== this.#epoch) return;
      this.#store(
        [...queues].map(([key, changes]) => [
          key,
          changes.reduce((tags, change) => change(tags), [...(fetched[key] ?? [])])
        ])
      );
    } finally {
      for (const [key, changes] of queues) {
        if (this.#loading.get(key) === changes) this.#loading.delete(key);
      }
    }
  }

  #store(entries: readonly (readonly [string, TagRef[]])[]): void {
    const next = putAllBounded(this.#cache, this.#order, entries, MAX_CACHED);
    this.#cache = next.record;
    this.#order = next.order;
  }
}

export const tagCache = new TagCache();
