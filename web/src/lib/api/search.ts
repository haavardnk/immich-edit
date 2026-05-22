import { getJson, sendJson } from './client';
import type { AssetSummary } from '$lib/types/album';

export interface SearchResult {
  items: AssetSummary[];
  count: number;
  total: number;
  nextPage: string | null;
}

export interface AssetStatistics {
  images: number;
  videos: number;
  total: number;
}

export function searchMetadata(body: Record<string, unknown>): Promise<SearchResult> {
  return sendJson('POST', '/api/search/metadata', body);
}

export function searchStatistics(body: Record<string, unknown>): Promise<{ total: number }> {
  return sendJson('POST', '/api/search/statistics', body);
}

export function assetStatistics(params?: Record<string, string>): Promise<AssetStatistics> {
  const qs = params ? '?' + new URLSearchParams(params).toString() : '';
  return getJson(`/api/assets/statistics${qs}`);
}
