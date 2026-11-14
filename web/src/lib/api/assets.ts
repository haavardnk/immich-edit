import { getJson, request, sendJson, url } from './client';
import type { AssetDetail } from '#lib/types/asset.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { fnv1a } from '#lib/utils/hash.ts';
import { thumbhashDataUrl } from '#lib/utils/thumbhash.ts';

export function getAsset(id: string): Promise<AssetDetail> {
  return getJson(url`/api/assets/${id}`);
}

export interface AssetUpdate {
  isFavorite?: boolean;
  rating?: number | null;
  description?: string;
}

export function updateAsset(id: string, body: AssetUpdate): Promise<AssetDetail> {
  return sendJson('PUT', url`/api/assets/${id}`, body);
}

export async function warmFrames(ids: string[]): Promise<void> {
  await request(
    '/api/frames/warm',
    {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ ids })
    },
    { silent: true }
  );
}

export type ThumbSource = { id: string; thumbhash?: string | null; checksum?: string | null };

function thumbVersion({ thumbhash, checksum }: ThumbSource): string | null {
  return thumbhash && checksum ? fnv1a(thumbhash) + fnv1a(checksum) : null;
}

export function thumbUrl(
  id: string,
  size: 'thumbnail' | 'preview' = 'thumbnail',
  version?: string | null
): string {
  return version
    ? url`/api/assets/${id}/thumb?size=${size}&c=${version}`
    : url`/api/assets/${id}/thumb?size=${size}`;
}

export function immichPreviewUrl(asset: ThumbSource): string {
  return thumbUrl(sourceId(asset.id), 'preview', thumbVersion(asset));
}

export function editedThumbUrl(id: string, hash: string, revision: string, size = 400): string {
  return url`/api/assets/${id}/edited-thumb?h=${hash}&r=${revision}&size=${size}`;
}

export function assetThumbUrl(
  asset: ThumbSource,
  immichSize: 'thumbnail' | 'preview' = 'thumbnail',
  editedSize = 400
): string {
  const hash = editedThumbs.getHash(asset.id);
  return hash
    ? editedThumbUrl(asset.id, hash, editedThumbs.getRevision(asset.id), editedSize)
    : thumbUrl(sourceId(asset.id), immichSize, thumbVersion(asset));
}

export function assetPlaceholder(asset: ThumbSource): string | undefined {
  if (!asset.thumbhash || editedThumbs.getHash(asset.id)) return undefined;
  const dataUrl = thumbhashDataUrl(asset.thumbhash);
  return dataUrl ? `url("${dataUrl}")` : undefined;
}
