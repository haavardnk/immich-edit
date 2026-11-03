import { browseTarget, type MetaTarget } from '#lib/browse/assetMeta.ts';
import * as tagOps from '#lib/browse/tagOps.ts';
import { isCullTag } from '#lib/stores/cullTags.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { getAsset } from '#lib/api/assets.ts';
import { putBounded } from '#lib/utils/boundedRecord.ts';

const MAX_CACHED = 50;

class LoupeTagStore {
  private cache = $state<Record<string, TagRef[]>>({});
  private order = $state<string[]>([]);

  tagsOf(id: string | null): TagRef[] {
    return id ? (this.cache[id] ?? []) : [];
  }

  set(id: string, tags: TagRef[]): void {
    const next = putBounded(this.cache, this.order, id, tags, MAX_CACHED);
    this.cache = next.record;
    this.order = next.order;
  }

  async ensure(id: string): Promise<void> {
    if (this.cache[id]) return;
    try {
      const asset = await getAsset(id);
      this.set(id, asset.tags);
    } catch (e) {
      toasts.fail('tags', e);
    }
  }

  target(id: string): MetaTarget {
    const summary = browseTarget(id);
    return {
      id,
      read: () => {
        const asset = summary.read();
        const tags = this.cache[id];
        if (!asset || !tags) return asset;
        return {
          ...asset,
          tags: [...tags.filter((t) => !isCullTag(t)), ...asset.tags.filter(isCullTag)]
        };
      },
      patch: (fields) => {
        if (fields.tags && this.cache[id]) this.set(id, fields.tags);
        summary.patch(fields);
      },
      fail: summary.fail
    };
  }

  async add(id: string, tag: TagRef): Promise<void> {
    await tagOps.addTag(this.target(id), tag);
  }

  async remove(id: string, tagId: string): Promise<void> {
    await tagOps.removeTag(this.target(id), tagId);
  }

  createAndAdd(id: string, value: string): Promise<TagRef | null> {
    return tagOps.createAndAddTag(this.target(id), value);
  }
}

export const loupeTags = new LoupeTagStore();
