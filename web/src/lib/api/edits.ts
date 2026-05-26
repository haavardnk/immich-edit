import { getJson, sendJson } from './client';
import type { Edits, EditRecord } from '$lib/types/edits';
import { editsToManifest } from '$lib/types/edits';

export interface EditedAssetEntry {
  id: string;
  hash: string;
  updated_at: string;
}

export function listEditedAssets(): Promise<EditedAssetEntry[]> {
  return getJson('/api/edits');
}

export function getEdits(assetId: string): Promise<EditRecord> {
  return getJson(`/api/assets/${assetId}/edits`);
}

export async function putEdits(assetId: string, edits: Edits): Promise<EditRecord> {
  const saved = await sendJson<EditRecord>('PUT', `/api/assets/${assetId}/edits`, editsToManifest(edits));
  if (typeof window !== 'undefined') {
    window.dispatchEvent(
      new CustomEvent('immich-edit:edits-saved', {
        detail: { id: assetId, hash: saved.hash, updated_at: saved.updated_at }
      })
    );
  }
  return saved;
}

export async function deleteEdits(assetId: string): Promise<void> {
  await sendJson<void>('DELETE', `/api/assets/${assetId}/edits`, undefined);
  if (typeof window !== 'undefined') {
    window.dispatchEvent(
      new CustomEvent('immich-edit:edits-deleted', { detail: { id: assetId } })
    );
  }
}

export function autoEdits(assetId: string, context: Edits): Promise<Edits> {
  return sendJson('POST', `/api/assets/${assetId}/edits/auto`, context);
}
