import { feedCache } from '#lib/browse/feedCache.ts';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';

export function libraryChanged(): void {
  monthCounts.invalidate();
  feedCache.clear();
}
