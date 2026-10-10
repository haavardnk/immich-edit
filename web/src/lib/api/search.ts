import { sendJson } from './client';
import type { AssetSummary } from '$lib/types/album';
import type {
  BrowseFilter,
  PageQuery,
  SearchPage,
  SearchSort,
  SmartQuery
} from '$lib/types/search';

export const BROWSE_PAGE_SIZE = 500;

export function searchMetadata(query: PageQuery): Promise<SearchPage> {
  return sendJson('POST', '/api/search/metadata', query);
}

export function searchWindow(
  assetId: string,
  filter: BrowseFilter,
  sort: SearchSort,
  radius: number
): Promise<{ items: AssetSummary[] }> {
  return sendJson('POST', '/api/search/window', { assetId, filter, sort, radius }, undefined, {
    silent: true
  });
}

export function searchSmart(query: SmartQuery): Promise<SearchPage> {
  return sendJson('POST', '/api/search/smart', query, undefined, { silent: true });
}

export function searchStatistics(filter: BrowseFilter): Promise<{ total: number }> {
  return sendJson('POST', '/api/search/statistics', { filter });
}
