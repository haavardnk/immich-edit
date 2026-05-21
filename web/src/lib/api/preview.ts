import { getJson, postForBlob } from './client';
import type { Edits } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';

export function persistedPreviewUrl(assetId: string, max: number): string {
  return `/api/assets/${assetId}/preview?max=${max}`;
}

export async function livePreview(
  assetId: string,
  edits: Edits,
  maxEdge: number,
  signal?: AbortSignal
): Promise<{ blob: Blob; metaId: string | null }> {
  return postForBlob(`/api/assets/${assetId}/preview`, { max_edge: maxEdge, edits }, signal);
}

export function getPreviewMeta(assetId: string, metaId: string): Promise<PreviewMeta> {
  return getJson(`/api/assets/${assetId}/preview/meta/${metaId}`);
}
