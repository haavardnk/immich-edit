import { sendJson } from './client';
import type { AssetSummary } from '$lib/types/album';
import type { SearchQuery, SearchResult } from '$lib/types/search';

export function searchMetadata(body: SearchQuery): Promise<SearchResult> {
  return sendJson('POST', '/api/search/metadata', body);
}

export function searchWindow(
  assetId: string,
  query: SearchQuery,
  radius: number
): Promise<{ items: AssetSummary[] }> {
  return sendJson('POST', '/api/search/window', { assetId, query, radius }, undefined, {
    silent: true
  });
}

export function searchSmart(body: SearchQuery): Promise<SearchResult> {
  return sendJson('POST', '/api/search/smart', body, undefined, { silent: true });
}

export function searchStatistics(body: SearchQuery): Promise<{ total: number }> {
  return sendJson('POST', '/api/search/statistics', body);
}
