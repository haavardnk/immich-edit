import { upsertTags, type TagSummary } from '$lib/api/tags';
import { library } from '$lib/stores/library.svelte';
import type { TagRef } from '$lib/types/asset';

export const REJECT_TAG_VALUE = 'immich-edit/reject';
export const MANAGED_TAG_PREFIX = 'immich-edit';

export type RejectableAsset = { tags?: TagRef[] | null };

export function isRejectTag(tag: TagRef): boolean {
  return !!tag.value && tag.value.toLowerCase() === REJECT_TAG_VALUE.toLowerCase();
}

export function isManagedTag(tag: TagRef): boolean {
  const value = tag.value?.toLowerCase();
  if (!value) return false;
  return value === MANAGED_TAG_PREFIX || value.startsWith(`${MANAGED_TAG_PREFIX}/`);
}

export function isRejected(asset: RejectableAsset): boolean {
  return asset.tags?.some(isRejectTag) ?? false;
}

export function toTagRef(tag: TagSummary): TagRef {
  return {
    id: tag.id,
    name: tag.name,
    value: tag.value,
    parentId: tag.parentId,
    color: tag.color
  };
}

export function addRejectTag(tags: TagRef[], rejectTag: TagRef): TagRef[] {
  if (tags.some((t) => t.id === rejectTag.id)) return tags;
  return [...tags, rejectTag];
}

export function removeRejectTag(tags: TagRef[]): TagRef[] {
  return tags.filter((t) => !isRejectTag(t));
}

export function setRejectedTags(tags: TagRef[], rejectTag: TagRef, rejected: boolean): TagRef[] {
  return rejected ? addRejectTag(tags, rejectTag) : removeRejectTag(tags);
}

let cachedRejectTag: TagRef | null = null;

export async function ensureRejectTag(): Promise<TagRef | null> {
  if (cachedRejectTag) return cachedRejectTag;
  const existing = library.tags.find((t) => isRejectTag(toTagRef(t)));
  if (existing) {
    cachedRejectTag = toTagRef(existing);
    return cachedRejectTag;
  }
  const created = await upsertTags([REJECT_TAG_VALUE]);
  const tag = created.find((t) => isRejectTag(toTagRef(t))) ?? created[0];
  if (!tag) return null;
  if (!library.tags.some((t) => t.id === tag.id)) {
    library.tags = [...library.tags, tag];
  }
  cachedRejectTag = toTagRef(tag);
  return cachedRejectTag;
}
