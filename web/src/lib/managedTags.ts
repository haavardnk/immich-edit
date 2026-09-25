import { listTags, upsertTags, type TagSummary } from '$lib/api/tags';
import { library } from '$lib/stores/library.svelte';
import type { TagRef } from '$lib/types/asset';

export const MANAGED_TAG_PREFIX = 'immich-edit';

export function isManagedTag(tag: TagRef): boolean {
  const value = tag.value?.toLowerCase();
  if (!value) return false;
  return value === MANAGED_TAG_PREFIX || value.startsWith(`${MANAGED_TAG_PREFIX}/`);
}

export function tagHasValue(tag: TagRef, value: string): boolean {
  return !!tag.value && tag.value.toLowerCase() === value.toLowerCase();
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

let libraryTags: Promise<void> | null = null;

export function libraryTagsReady(): Promise<void> {
  if (library.tags.length > 0) return Promise.resolve();
  libraryTags ??= listTags()
    .then((tags) => {
      library.tags = tags;
    })
    .catch(() => {
      libraryTags = null;
    });
  return libraryTags;
}

export function findManagedTag(value: string): TagRef | null {
  const existing = library.tags.find((t) => tagHasValue(toTagRef(t), value));
  return existing ? toTagRef(existing) : null;
}

const ensured = new Map<string, TagRef>();

export async function ensureManagedTag(value: string): Promise<TagRef | null> {
  const cached = ensured.get(value);
  if (cached) return cached;
  const existing = findManagedTag(value);
  if (existing) {
    ensured.set(value, existing);
    return existing;
  }
  const created = await upsertTags([value]);
  const tag = created.find((t) => tagHasValue(toTagRef(t), value)) ?? created[0];
  if (!tag) return null;
  if (!library.tags.some((t) => t.id === tag.id)) library.tags = [...library.tags, tag];
  const ref = toTagRef(tag);
  ensured.set(value, ref);
  return ref;
}
