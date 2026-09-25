import { searchMetadata } from '$lib/api/search';
import { findManagedTag, libraryTagsReady, tagHasValue } from '$lib/managedTags';
import type { AssetSummary } from '$lib/types/album';
import type { TagRef } from '$lib/types/asset';
import type { SearchQuery } from '$lib/types/search';

export class TagMembers {
  ids = $state(new Set<string>());
  tag = $state<TagRef | null>(null);
  private loadingPromise: Promise<void> | null = null;

  constructor(readonly value: string) {}

  reset(): void {
    this.ids = new Set();
    this.tag = null;
    this.loadingPromise = null;
  }

  load(): Promise<void> {
    this.loadingPromise ??= this.fetch();
    return this.loadingPromise;
  }

  private async fetch(): Promise<void> {
    await libraryTagsReady();
    const tag = findManagedTag(this.value);
    if (!tag) {
      this.ids = new Set();
      this.tag = null;
      return;
    }
    this.tag = tag;
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
    if (!this.ids.has(id)) return;
    const next = new Set(this.ids);
    next.delete(id);
    this.ids = next;
  }

  stamp(assets: AssetSummary[]): AssetSummary[] {
    const tag = this.tag;
    if (this.ids.size === 0 || !tag) return assets;
    return assets.map((a) =>
      this.ids.has(a.id) && !a.tags.some((t) => tagHasValue(t, this.value))
        ? { ...a, tags: [...a.tags, tag] }
        : a
    );
  }
}
