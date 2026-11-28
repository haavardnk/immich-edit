import { tagMemberships } from '#lib/api/tags.ts';
import { toTagRef } from '#lib/browse/managedTags.ts';
import { tagCache } from '#lib/stores/tagCache.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';

async function fetchTags(ids: readonly string[]): Promise<Record<string, TagRef[]>> {
  const found = await tagMemberships(ids);
  return Object.fromEntries(Object.entries(found).map(([id, tags]) => [id, tags.map(toTagRef)]));
}

export function loadTagMembership(ids: readonly string[]): Promise<void> {
  return tagCache.loadMany(ids, fetchTags);
}
