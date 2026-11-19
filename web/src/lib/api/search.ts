import { sendJson } from './client';
import type {
  BrowseFilter,
  MonthCount,
  PageQuery,
  SearchPage,
  SearchSort,
  SearchWindow,
  SmartQuery,
  SuggestionQuery,
  WindowReach
} from '#lib/types/search.ts';

export const BROWSE_PAGE_SIZE = 500;

export function searchMetadata(query: PageQuery): Promise<SearchPage> {
  return sendJson('POST', '/api/search/metadata', query);
}

export async function searchFirst(filter: BrowseFilter, sort: SearchSort): Promise<string | null> {
  const page = await sendJson<SearchPage>(
    'POST',
    '/api/search/metadata',
    { filter, sort, size: 1 } satisfies PageQuery,
    undefined,
    { silent: true }
  );
  return page.items[0]?.id ?? null;
}

export function searchWindow(
  assetId: string,
  filter: BrowseFilter,
  sort: SearchSort,
  reach: WindowReach
): Promise<SearchWindow> {
  return sendJson('POST', '/api/search/window', { assetId, filter, sort, ...reach }, undefined, {
    silent: true
  });
}

export function searchMonths(filter: BrowseFilter): Promise<{ months: MonthCount[] }> {
  return sendJson('POST', '/api/search/months', { filter }, undefined, { silent: true });
}

export function searchSmart(query: SmartQuery): Promise<SearchPage> {
  return sendJson('POST', '/api/search/smart', query, undefined, { silent: true });
}

export function searchStatistics(filter: BrowseFilter): Promise<{ total: number }> {
  return sendJson('POST', '/api/search/statistics', { filter });
}

export function searchSuggestions(query: SuggestionQuery): Promise<(string | null)[]> {
  return sendJson('POST', '/api/search/suggestions', query);
}
