import { searchMetadata } from '#lib/api/search.ts';
import { findManagedTag, tagHasValue } from '#lib/browse/managedTags.ts';
import { library } from '#lib/stores/library.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';

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
    await library.ensureTags().catch(() => undefined);
    const tag = findManagedTag(this.value);
    if (!tag) {
      this.ids = new Set();
      this.tag = null;
      return;
    }
    this.tag = tag;
    const ids = new Set<string>();
    let cursor: string | undefined;
    do {
      const res = await searchMetadata({ filter: { scope: { tag: tag.id } }, size: 1000, cursor });
      for (const a of res.items) ids.add(a.id);
      cursor = res.nextCursor ?? undefined;
    } while (cursor);
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
