import { sendJson } from './client';
import type { SearchQuery, SearchResult } from '$lib/types/search';

export function searchMetadata(body: SearchQuery): Promise<SearchResult> {
  return sendJson('POST', '/api/search/metadata', body);
}

export function searchSmart(body: SearchQuery): Promise<SearchResult> {
  return sendJson('POST', '/api/search/smart', body, undefined, { silent: true });
}

export function searchStatistics(body: SearchQuery): Promise<{ total: number }> {
  return sendJson('POST', '/api/search/statistics', body);
}
