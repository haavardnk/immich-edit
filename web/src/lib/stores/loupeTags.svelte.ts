import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
import type { TagRef } from '$lib/types/asset';
import { getAsset } from '$lib/api/assets';
import { putBounded } from '$lib/utils/boundedRecord';

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

  async add(id: string, tag: TagRef): Promise<void> {
    const prev = this.tagsOf(id);
    if (prev.some((t) => t.id === tag.id)) return;
    if (!(await metadataConsent.gate())) return;
    this.set(id, [...prev, tag]);
    try {
      await addTagToAsset(tag.id, id);
    } catch (e) {
      this.set(id, prev);
      toasts.fail('tag', e);
    }
  }

  async remove(id: string, tagId: string): Promise<void> {
    const prev = this.tagsOf(id);
    if (!(await metadataConsent.gate())) return;
    this.set(
      id,
      prev.filter((t) => t.id !== tagId)
    );
    try {
      await removeTagFromAsset(tagId, id);
    } catch (e) {
      this.set(id, prev);
      toasts.fail('tag', e);
    }
  }

  async createAndAdd(id: string, value: string): Promise<TagRef | null> {
    if (!(await metadataConsent.gate())) return null;
    let created: TagRef;
    try {
      const tags = await upsertTags([value]);
      const tag = tags[0];
      if (!tag) return null;
      created = {
        id: tag.id,
        name: tag.name,
        value: tag.value,
        parentId: tag.parentId,
        color: tag.color
      };
    } catch (e) {
      toasts.fail('tag', e);
      return null;
    }
    const prev = this.tagsOf(id);
    if (prev.some((t) => t.id === created.id)) return created;
    this.set(id, [...prev, created]);
    try {
      await addTagToAsset(created.id, id);
    } catch (e) {
      this.set(id, prev);
      toasts.fail('tag', e);
      return null;
    }
    return created;
  }
}

export const loupeTags = new LoupeTagStore();
