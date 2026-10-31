import { warmFrames } from '#lib/api/assets.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';

export function warmNeighbours(id: string): boolean {
  const ids: string[] = [];
  for (const asset of [browsing.nextOf(id), browsing.prevOf(id)]) {
    if (asset?.type === 'IMAGE') ids.push(asset.id);
  }
  if (ids.length === 0) return false;
  void warmFrames(ids).catch(() => undefined);
  return true;
}
