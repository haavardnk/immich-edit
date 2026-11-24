import { searchMetadata } from '#lib/api/search.ts';
import type { AssetSummary } from '#lib/types/album.ts';

const HOUR = 3_600_000;
const SEARCH_MARGIN = 14 * HOUR;
const PAGE = 500;
const MAX_PAGES = 10;

export function stillsOnly(assets: readonly AssetSummary[]): AssetSummary[] {
  return assets.filter((asset) => asset.type !== 'VIDEO');
}

export async function photosAround(start: number, end: number): Promise<AssetSummary[]> {
  const filter = {
    takenAfter: new Date(start - SEARCH_MARGIN).toISOString(),
    takenBefore: new Date(end + SEARCH_MARGIN).toISOString(),
    imagesOnly: true
  };
  const found: AssetSummary[] = [];
  let cursor: string | undefined;
  for (let page = 0; page < MAX_PAGES; page += 1) {
    const result = await searchMetadata({
      filter,
      sort: { field: 'taken', direction: 'asc' },
      size: PAGE,
      withExif: true,
      ...(cursor ? { cursor } : {})
    });
    found.push(...result.items);
    if (!result.nextCursor) break;
    cursor = result.nextCursor;
  }
  return found;
}
