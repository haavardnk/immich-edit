import { listEditedAssets } from '$lib/api/edits';
import { folderAssets } from '$lib/api/folders';
import { searchMetadata, searchSmart } from '$lib/api/search';
import { rejected } from '$lib/stores/rejected.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
import type { SearchMode } from '$lib/searchMode';
import type { AssetSummary } from '$lib/types/album';
import type { SearchQuery, SearchResult } from '$lib/types/search';

export async function runSearch(
  query: string,
  mode: SearchMode,
  body: SearchQuery
): Promise<SearchResult> {
  const byFilename: SearchQuery = { ...body, originalFileName: query };
  delete byFilename.query;
  if (mode === 'filename') return searchMetadata(byFilename);
  try {
    return await searchSmart(body);
  } catch {
    toasts.push('warn', 'Smart search unavailable, showing filename matches');
    return await searchMetadata(byFilename);
  }
}

export async function loadEditedAssets(): Promise<AssetSummary[]> {
  const [entries] = await Promise.all([
    listEditedAssets(true),
    rejected.load().catch(() => undefined)
  ]);
  return rejected.stamp(
    entries.map<AssetSummary>((entry) =>
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
    fileCreatedAt: a.fileCreatedAt,
    updatedAt: a.updatedAt,
    checksum: a.checksum,
    isFavorite: a.isFavorite ?? false,
    exifInfo: a.exifInfo ?? null,
    tags: []
  }));
}
