import { goto } from '$app/navigation';
import { createCopy } from '#lib/api/copies.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { editorHref } from './navigation';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { countOf } from '#lib/utils/plural.ts';
import { countFailures } from '#lib/utils/pool.ts';

const COPY_CONCURRENCY = 4;

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
    { ttlMs: 3000 }
  );
  return created.id;
}

export async function createVirtualCopies(ids: string[]): Promise<void> {
  const failed = await countFailures(ids, COPY_CONCURRENCY, async (id) => {
    await createVirtualCopy(id, { navigate: false });
    return true;
  });
  toasts.tally(
    failed,
    `${failed} of ${ids.length} failed`,
    `Created ${countOf(ids.length, 'virtual copy', 'virtual copies')}`
  );
}
