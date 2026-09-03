import { getJson, sendJson, url, ApiError, ConflictError } from './client';
import { editsToManifest } from '$lib/edits/manifest';
import type { AssetDetail } from '$lib/types/asset';
import type { Edits, EditRecord } from '$lib/types/edits';

export interface EditedAssetEntry {
  id: string;
  hash: string;
  updated_at: string;
  asset?: AssetDetail;
}

export function listEditedAssets(withAssets = false): Promise<EditedAssetEntry[]> {
  return getJson(withAssets ? '/api/edits?with_assets=true' : '/api/edits');
}

export function getEdits(assetId: string): Promise<EditRecord> {
  return getJson(url`/api/assets/${assetId}/edits`);
}

export async function putEdits(
  assetId: string,
  edits: Edits,
  baseHash?: string,
  action?: string
): Promise<EditRecord> {
  const headers: Record<string, string> = {};
  if (baseHash) headers['if-match'] = baseHash;
  try {
    const saved = await sendJson<EditRecord>(
      'PUT',
      url`/api/assets/${assetId}/edits`,
      { manifest: editsToManifest(edits), action: action ?? null },
      { headers }
    );
    if (typeof window !== 'undefined') {
      window.dispatchEvent(
        new CustomEvent('immich-edit:edits-saved', {
          detail: { id: assetId, hash: saved.hash, updated_at: saved.updated_at }
        })
      );
    }
    return saved;
  } catch (e) {
    if (e instanceof ApiError && e.status === 409) {
      throw new ConflictError(e.message, e.body as EditRecord | undefined);
    }
    throw e;
  }
}

export async function deleteEdits(
  assetId: string,
  action?: string,
  baseHash?: string
): Promise<void> {
  const headers: Record<string, string> = {};
  if (baseHash) headers['if-match'] = baseHash;
  try {
    await sendJson<void>(
      'DELETE',
      url`/api/assets/${assetId}/edits`,
      { action: action ?? null },
      { headers }
    );
  } catch (e) {
    if (e instanceof ApiError && e.status === 409) {
      throw new ConflictError(e.message, e.body as EditRecord | undefined);
    }
    throw e;
  }
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('immich-edit:edits-deleted', { detail: { id: assetId } }));
  }
}

export function autoEdits(assetId: string, context: Edits): Promise<Edits> {
  return sendJson('POST', url`/api/assets/${assetId}/edits/auto`, context);
}

export interface EditHistoryEntry {
  id: number;
  manifest_hash: string;
  deleted: boolean;
  edits: Edits | null;
  created_at: string;
  action: string | null;
}

export function listEditHistory(assetId: string): Promise<EditHistoryEntry[]> {
  return getJson(url`/api/assets/${assetId}/edits/history`);
}

export async function restoreEdits(assetId: string, entryId: number): Promise<EditRecord | null> {
  const saved = await sendJson<EditRecord | undefined>(
    'POST',
    url`/api/assets/${assetId}/edits/restore`,
    { entry_id: entryId }
  );
  if (typeof window !== 'undefined') {
    if (saved) {
      window.dispatchEvent(
        new CustomEvent('immich-edit:edits-saved', {
          detail: { id: assetId, hash: saved.hash, updated_at: saved.updated_at }
        })
      );
    } else {
      window.dispatchEvent(
        new CustomEvent('immich-edit:edits-deleted', { detail: { id: assetId } })
      );
    }
  }
  return saved ?? null;
}
