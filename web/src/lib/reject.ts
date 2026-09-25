import { ensureManagedTag, tagHasValue } from '$lib/managedTags';
import type { TagRef } from '$lib/types/asset';

export const REJECT_TAG_VALUE = 'immich-edit/reject';

export type RejectableAsset = { tags?: TagRef[] | null };

export function isRejectTag(tag: TagRef): boolean {
  return tagHasValue(tag, REJECT_TAG_VALUE);
}

export function isRejected(asset: RejectableAsset): boolean {
  return asset.tags?.some(isRejectTag) ?? false;
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

export function ensureRejectTag(): Promise<TagRef | null> {
  return ensureManagedTag(REJECT_TAG_VALUE);
}
