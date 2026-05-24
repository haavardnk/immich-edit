import { getJson, sendJson } from './client';
import type { AssetDetail } from '$lib/types/asset';
import { editedThumbs } from '$lib/stores/editedThumbs.svelte';

export function getAsset(id: string): Promise<AssetDetail> {
  return getJson(`/api/assets/${id}`);
}

export function updateAsset(id: string, body: Record<string, unknown>): Promise<AssetDetail> {
  return sendJson('PUT', `/api/assets/${id}`, body);
}

export function thumbUrl(id: string, size: 'thumbnail' | 'preview' = 'thumbnail'): string {
  return `/api/assets/${id}/thumb?size=${size}`;
}

export function editedThumbUrl(id: string, hash: string, size = 400): string {
  return `/api/assets/${id}/edited-thumb?h=${encodeURIComponent(hash)}&size=${size}`;
}

export function assetThumbUrl(
  id: string,
  immichSize: 'thumbnail' | 'preview' = 'thumbnail',
  editedSize = 400
): string {
  const hash = editedThumbs.getHash(id);
  return hash ? editedThumbUrl(id, hash, editedSize) : thumbUrl(id, immichSize);
}
