import { getJson, sendJson } from './client';
import type { AssetDetail } from '$lib/types/asset';

export function getAsset(id: string): Promise<AssetDetail> {
  return getJson(`/api/assets/${id}`);
}

export function updateAsset(id: string, body: Record<string, unknown>): Promise<AssetDetail> {
  return sendJson('PUT', `/api/assets/${id}`, body);
}

export function thumbUrl(id: string, size: 'thumbnail' | 'preview' = 'thumbnail'): string {
  return `/api/assets/${id}/thumb?size=${size}`;
}
