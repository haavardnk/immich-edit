import { goto } from '$app/navigation';
import { createCopy } from '$lib/api/copies';
import { sourceId } from '$lib/browse/assetKey';
import { editorHref } from './navigation';
import { browsing } from '$lib/stores/browsing.svelte';
import { toasts } from '$lib/stores/toasts.svelte';

export async function createVirtualCopy(
  assetId: string,
  opts: { from?: 'current' | 'neutral'; navigate?: boolean; returnPath?: string | null } = {}
): Promise<string | null> {
  const source = sourceId(assetId);
  const created = await createCopy(assetId, { from: opts.from ?? 'current' });
  browsing.insertCopy(created.id, source, created.name);
  if (opts.navigate === false) return created.id;
  await goto(editorHref(created.id, opts.returnPath));
  toasts.push(
    'success',
    created.name ? `Created virtual copy "${created.name}"` : 'Created virtual copy',
    3000
  );
  return created.id;
}
