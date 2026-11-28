import { sourceId } from '#lib/browse/assetKey.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { putAllBounded } from '#lib/utils/boundedRecord.ts';
import { FailureBackoff } from '#lib/utils/failureBackoff.ts';

const MAX_CACHED = 1000;
const BATCH = 100;
export const RETRY_MS = 30_000;

type TagChange = (tags: TagRef[]) => TagRef[];

type BatchFetch = (ids: readonly string[]) => Promise<Record<string, readonly TagRef[]>>;

interface Loading {
  changes: TagChange[];
  done: Promise<void>;
}

class TagCache {
  #cache = $state.raw<Record<string, TagRef[]>>({});
  #order: string[] = [];
  #loading = new Map<string, Loading>();
  #epoch = 0;
  #failed = new FailureBackoff(RETRY_MS);

  get(id: string): TagRef[] | undefined {
    return this.#cache[sourceId(id)];
  }

  set(id: string, tags: TagRef[]): void {
    this.#store([[sourceId(id), tags]]);
  }

  async load(id: string, fetch: () => Promise<TagRef[]>): Promise<void> {
    const key = sourceId(id);
    if (this.#cache[key]) return;
    const loading = this.#loading.get(key);
    if (loading) {
      await loading.done.catch(() => undefined);
      return this.load(id, fetch);
    }
    await this.#run([key], async () => ({ [key]: await fetch() }));
  }

  async loadMany(ids: readonly string[], fetch: BatchFetch): Promise<void> {
    const keys = [...new Set(ids.map(sourceId))].filter(
      (key) => !this.#cache[key] && !this.#loading.has(key) && !this.#failed.waiting(key)
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
    this.#failed.clear();
    this.#loading.clear();
    this.#cache = {};
    this.#order = [];
  }

  update(id: string, change: TagChange): void {
    const key = sourceId(id);
    const tags = this.#cache[key];
    if (tags) this.#store([[key, change(tags)]]);
    else this.#loading.get(key)?.changes.push(change);
  }

  #run(keys: readonly string[], fetch: BatchFetch): Promise<void> {
    const queues = new Map(keys.map((key) => [key, [] as TagChange[]]));
    const done = this.#fetch(this.#epoch, queues, fetch).finally(() => {
      for (const [key, changes] of queues) {
        if (this.#loading.get(key)?.changes === changes) this.#loading.delete(key);
      }
    });
    for (const [key, changes] of queues) this.#loading.set(key, { changes, done });
    return done;
  }

  async #fetch(
    epoch: number,
    queues: ReadonlyMap<string, TagChange[]>,
    fetch: BatchFetch
  ): Promise<void> {
    const keys = [...queues.keys()];
    const fetched = await fetch(keys).catch((error: unknown) => {
      if (epoch === this.#epoch) this.#failed.fail(keys);
      throw error;
    });
    if (epoch !== this.#epoch) return;
    this.#failed.fail(keys.filter((key) => !fetched[key]));
    this.#store(
      [...queues].flatMap(([key, changes]) => {
        const tags = fetched[key];
        return tags ? [[key, changes.reduce((next, change) => change(next), [...tags])]] : [];
      })
    );
  }

  #store(entries: readonly (readonly [string, TagRef[]])[]): void {
    const next = putAllBounded(this.#cache, this.#order, entries, MAX_CACHED);
    this.#cache = next.record;
    this.#order = next.order;
  }
}

export const tagCache = new TagCache();
