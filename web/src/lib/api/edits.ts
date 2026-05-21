import { getJson, sendJson } from './client';
import type { Edits, EditRecord } from '$lib/types/edits';
import { editsToManifest } from '$lib/types/edits';

export function listEditedAssetIds(): Promise<string[]> {
  return getJson('/api/edits');
}

export function getEdits(assetId: string): Promise<EditRecord> {
  return getJson(`/api/assets/${assetId}/edits`);
}

export function putEdits(assetId: string, edits: Edits): Promise<EditRecord> {
  return sendJson('PUT', `/api/assets/${assetId}/edits`, editsToManifest(edits));
}

export function deleteEdits(assetId: string): Promise<void> {
  return sendJson('DELETE', `/api/assets/${assetId}/edits`, undefined);
}

export function autoEdits(assetId: string): Promise<Edits> {
  return sendJson('POST', `/api/assets/${assetId}/edits/auto`, undefined);
}
