import { browseTarget, type MetaTarget } from '#lib/browse/assetMeta.ts';
import * as tagOps from '#lib/browse/tagOps.ts';
import { isCullTag } from '#lib/stores/cullTags.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { getAsset } from '#lib/api/assets.ts';
import { tagCache } from '#lib/stores/tagCache.svelte.ts';

class LoupeTagStore {
  tagsOf(id: string | null): TagRef[] {
    return id ? (tagCache.get(id) ?? []) : [];
  }

  async ensure(id: string): Promise<void> {
    try {
      await tagCache.load(id, async () => (await getAsset(id)).tags);
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
        const tags = tagCache.get(id);
        if (!asset || !tags) return asset;
        return {
          ...asset,
          tags: [...tags.filter((t) => !isCullTag(t)), ...asset.tags.filter(isCullTag)]
        };
      },
      patch: (fields) => {
        if (fields.tags && tagCache.get(id)) tagCache.set(id, fields.tags);
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
