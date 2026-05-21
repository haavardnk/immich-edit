import { getJson, sendJson } from './client';
import type { Edits, Sidecar } from '$lib/types/edits';

export function getEdits(assetId: string): Promise<Sidecar> {
  return getJson(`/api/assets/${assetId}/edits`);
}

export function putEdits(assetId: string, edits: Partial<Edits>): Promise<Sidecar> {
  return sendJson('PUT', `/api/assets/${assetId}/edits`, edits);
}

export function deleteEdits(assetId: string): Promise<void> {
  return sendJson('DELETE', `/api/assets/${assetId}/edits`, undefined);
}
