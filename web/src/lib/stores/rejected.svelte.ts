import { searchMetadata } from '$lib/api/search';
import { listTags } from '$lib/api/tags';
import { library } from './library.svelte';
import { isRejectTag, toTagRef } from '$lib/reject';
import type { AssetSummary } from '$lib/types/album';
import type { TagRef } from '$lib/types/asset';
import type { SearchQuery } from '$lib/types/search';

class RejectedStore {
  ids = $state(new Set<string>());
  tag = $state<TagRef | null>(null);
  private loadingPromise: Promise<void> | null = null;

  reset(): void {
    this.ids = new Set();
    this.tag = null;
    this.loadingPromise = null;
  }

  load(): Promise<void> {
    if (this.loadingPromise) return this.loadingPromise;
    this.loadingPromise = this.fetch();
    return this.loadingPromise;
  }

  private async fetch(): Promise<void> {
    if (library.tags.length === 0) {
      const tags = await listTags().catch(() => null);
      if (tags) library.tags = tags;
    }
    const tag = library.tags.find((t) => isRejectTag(toTagRef(t)));
    if (!tag) {
      this.ids = new Set();
      this.tag = null;
      return;
    }
    this.tag = toTagRef(tag);
    const ids = new Set<string>();
    let page: string | null = null;
    do {
      const body: SearchQuery = { tagIds: [tag.id], size: 1000 };
      if (page) body.page = Number(page);
      const res = await searchMetadata(body);
      for (const a of res.items) ids.add(a.id);
      page = res.nextPage;
    } while (page);
    this.ids = ids;
  }

  add(id: string, tag: TagRef): void {
    this.tag = tag;
    const next = new Set(this.ids);
    next.add(id);
    this.ids = next;
  }

  remove(id: string): void {
    const next = new Set(this.ids);
    next.delete(id);
    this.ids = next;
  }

  stamp(assets: AssetSummary[]): AssetSummary[] {
    if (this.ids.size === 0 || !this.tag) return assets;
    const tag = this.tag;
    return assets.map((a) =>
      this.ids.has(a.id) && !a.tags.some(isRejectTag) ? { ...a, tags: [...a.tags, tag] } : a
    );
  }
}

export const rejected = new RejectedStore();
