import { listEditedAssets } from '#lib/api/edits.ts';
import { folderAssets } from '#lib/api/folders.ts';
import { BROWSE_PAGE_SIZE, searchMetadata, searchSmart } from '#lib/api/search.ts';
import { loadCullTags, stampCullTags } from '#lib/stores/cullTags.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { BrowseFeedOptions } from '#lib/stores/browseFeed.svelte.ts';
import type { SearchMode } from './searchMode';
import type { AssetSummary } from '#lib/types/album.ts';
import type { BrowseFilter, SearchPage } from '#lib/types/search.ts';

export async function runSearch(
  text: string,
  mode: SearchMode,
  filter: BrowseFilter,
  cursor: string | undefined
): Promise<SearchPage> {
  const page = { size: BROWSE_PAGE_SIZE, withExif: true, cursor };
  if (mode !== 'context') return searchMetadata({ ...page, filter: { ...filter, [mode]: text } });
  try {
    return await searchSmart({ ...page, text, filter });
  } catch (e) {
    if (cursor) throw e;
    toasts.push('warn', 'Smart search unavailable, showing filename matches');
    return await searchMetadata({ ...page, filter: { ...filter, filename: text } });
  }
}

export function searchFeed(query: () => string, mode: () => SearchMode): BrowseFeedOptions {
  return {
    baseFilter: () => ({}),
    flatOnly: () => mode() === 'context',
    fetcher: (filter, cursor) => runSearch(query(), mode(), filter, cursor),
    includeStats: false
  };
}

export async function loadEditedAssets(): Promise<AssetSummary[]> {
  const [list] = await Promise.all([listEditedAssets(true), loadCullTags().catch(() => undefined)]);
  return stampCullTags(
    list.items.map<AssetSummary>((entry) =>
      entry.asset
        ? { ...entry.asset, updatedAt: entry.updated_at }
        : {
            id: entry.id,
            originalFileName: `Asset ${entry.id}`,
            type: 'IMAGE',
            fileCreatedAt: null,
            updatedAt: entry.updated_at,
            checksum: null,
            isFavorite: false,
            exifInfo: null,
            tags: []
          }
    )
  );
}

export async function loadFolderAssets(path: string): Promise<AssetSummary[]> {
  const raw = await folderAssets(path);
  return raw.map((a) => ({
    id: a.id,
    originalFileName: a.originalFileName,
    type: a.type,
    localDateTime: a.localDateTime,
    fileCreatedAt: a.fileCreatedAt,
    createdAt: a.createdAt,
    updatedAt: a.updatedAt,
    checksum: a.checksum,
    isFavorite: a.isFavorite ?? false,
    exifInfo: a.exifInfo ?? null,
    tags: []
  }));
}
