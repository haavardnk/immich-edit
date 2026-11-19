import type { TagRef } from '#lib/types/asset.ts';
import { putBounded } from '#lib/utils/boundedRecord.ts';

const MAX_CACHED = 50;

type TagChange = (tags: TagRef[]) => TagRef[];

class TagCache {
  #cache = $state<Record<string, TagRef[]>>({});
  #order = $state<string[]>([]);
  #loading = new Map<string, TagChange[]>();

  get(id: string): TagRef[] | undefined {
    return this.#cache[id];
  }

  set(id: string, tags: TagRef[]): void {
    const next = putBounded(this.#cache, this.#order, id, tags, MAX_CACHED);
    this.#cache = next.record;
    this.#order = next.order;
  }

  async load(id: string, fetch: () => Promise<TagRef[]>): Promise<void> {
    if (this.#cache[id] || this.#loading.has(id)) return;
    const changes: TagChange[] = [];
    this.#loading.set(id, changes);
    try {
      const fetched = await fetch();
      this.set(
        id,
        changes.reduce((tags, change) => change(tags), fetched)
      );
    } finally {
      this.#loading.delete(id);
    }
  }

  update(id: string, change: TagChange): void {
    const tags = this.#cache[id];
    if (tags) this.set(id, change(tags));
    else this.#loading.get(id)?.push(change);
  }
}

export const tagCache = new TagCache();
