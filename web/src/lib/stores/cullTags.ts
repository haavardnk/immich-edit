import type { AssetSummary } from '$lib/types/album';
import { labelMembers } from './labels.svelte';
import { rejected } from './rejected.svelte';

const managed = [rejected, ...Object.values(labelMembers)];

export async function loadCullTags(): Promise<void> {
  await Promise.all(managed.map((members) => members.load()));
}

export function stampCullTags(assets: AssetSummary[]): AssetSummary[] {
  return managed.reduce((items, members) => members.stamp(items), assets);
}
