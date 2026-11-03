import type { AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { isRejectTag } from '#lib/browse/reject.ts';
import { isLabelTag } from './labels';
import { labelMembers } from './labels.svelte';
import { rejected } from './rejected.svelte';

const managed = [rejected, ...Object.values(labelMembers)];

export function isCullTag(tag: TagRef): boolean {
  return isRejectTag(tag) || isLabelTag(tag);
}

export async function loadCullTags(): Promise<void> {
  await Promise.all(managed.map((members) => members.load()));
}

export function stampCullTags(assets: AssetSummary[]): AssetSummary[] {
  return managed.reduce((items, members) => members.stamp(items), assets);
}
