import { goto } from '$app/navigation';
import { createCopy } from '$lib/api/copies';
import { sourceId } from '$lib/assetKey';
import { browsing } from '$lib/stores/browsing.svelte';

export async function createVirtualCopy(
  assetId: string,
  opts: { from?: 'current' | 'neutral'; navigate?: boolean } = {}
): Promise<string | null> {
  const source = sourceId(assetId);
  const created = await createCopy(assetId, { from: opts.from ?? 'current' });
  browsing.insertCopy(created.id, source, created.name);
  if (opts.navigate !== false) await goto(`/assets/${created.id}`);
  return created.id;
}
