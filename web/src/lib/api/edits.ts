import { getJson, sendJson, url, ApiError, ConflictError } from './client';
import { editsToManifest, manifestToEdits } from '#lib/edits/manifest.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import type {
  AsShotWhite,
  EditManifest,
  Edits,
  EditRecord,
  WhiteBalance
} from '#lib/types/edits.ts';

export interface EditedAssetEntry {
  id: string;
  hash: string;
  updated_at: string;
  revision?: string;
  asset?: AssetDetail;
}

export interface EditedAssetList {
  render_revision: string;
  items: EditedAssetEntry[];
}

export function listEditedAssets(withAssets = false): Promise<EditedAssetList> {
  return getJson(withAssets ? '/api/edits?with_assets=true' : '/api/edits');
}

export function getEdits(assetId: string): Promise<EditRecord> {
  return getJson(url`/api/assets/${assetId}/edits`);
}

type WriteMethod = 'POST' | 'PUT' | 'DELETE';

async function conditionalWrite<T>(
  method: WriteMethod,
  path: string,
  body: unknown,
  baseHash?: string
): Promise<T> {
  const headers: Record<string, string> = {};
  if (baseHash) headers['if-match'] = baseHash;
  try {
    return await sendJson<T>(method, path, body, { headers });
  } catch (e) {
    if (e instanceof ApiError && e.status === 409) {
      throw new ConflictError(e.message, e.body as EditRecord | undefined);
    }
    throw e;
  }
}

function announce(assetId: string, saved: EditRecord | null): void {
  if (typeof window === 'undefined') return;
  window.dispatchEvent(
    saved
      ? new CustomEvent('immich-edit:edits-saved', {
          detail: { id: assetId, hash: saved.hash, updated_at: saved.updated_at }
        })
      : new CustomEvent('immich-edit:edits-deleted', { detail: { id: assetId } })
  );
}

export async function writeEdits(
  assetId: string,
  method: WriteMethod,
  path: string,
  body: unknown,
  baseHash?: string
): Promise<EditRecord> {
  const saved = await conditionalWrite<EditRecord>(method, path, body, baseHash);
  announce(assetId, saved);
  return saved;
}

export function putEdits(
  assetId: string,
  edits: Edits,
  baseHash?: string,
  action?: string
): Promise<EditRecord> {
  return writeEdits(
    assetId,
    'PUT',
    url`/api/assets/${assetId}/edits`,
    { manifest: editsToManifest(edits), action: action ?? null },
    baseHash
  );
}

export async function deleteEdits(
  assetId: string,
  action?: string,
  baseHash?: string
): Promise<void> {
  await conditionalWrite<void>(
    'DELETE',
    url`/api/assets/${assetId}/edits`,
    { action: action ?? null },
    baseHash
  );
  announce(assetId, null);
}

export function autoEdits(assetId: string, context: Edits): Promise<Edits> {
  return sendJson('POST', url`/api/assets/${assetId}/edits/auto`, context);
}

export interface WhiteBalanceSample {
  u: number;
  v: number;
  w: number;
  h: number;
}

export function sampleWhiteBalance(
  assetId: string,
  sample: WhiteBalanceSample,
  context: Edits,
  opts?: { silent?: boolean }
): Promise<WhiteBalance> {
  return sendJson(
    'POST',
    url`/api/assets/${assetId}/edits/white-balance`,
    { ...sample, edits: context },
    undefined,
    opts
  );
}

export function autoWhiteBalance(assetId: string, context: Edits): Promise<WhiteBalance> {
  return sendJson('POST', url`/api/assets/${assetId}/edits/white-balance/auto`, context);
}

export function getAsShotWhite(assetId: string, context: Edits): Promise<AsShotWhite> {
  return sendJson('POST', url`/api/assets/${assetId}/edits/white-balance/as-shot`, context);
}

export interface EditHistoryEntry {
  id: number;
  manifest_hash: string;
  deleted: boolean;
  edits: Edits | null;
  created_at: string;
  action: string | null;
}

export interface EditHistoryItem extends Omit<EditHistoryEntry, 'edits'> {
  manifest: EditManifest | null;
}

export async function listEditHistory(assetId: string): Promise<EditHistoryEntry[]> {
  const items = await getJson<EditHistoryItem[]>(url`/api/assets/${assetId}/edits/history`);
  return items.map(({ manifest, ...item }) => ({
    ...item,
    edits: manifest ? manifestToEdits(manifest) : null
  }));
}

export async function restoreEdits(assetId: string, entryId: number): Promise<EditRecord | null> {
  const saved = await sendJson<EditRecord | undefined>(
    'POST',
    url`/api/assets/${assetId}/edits/restore`,
    { entry_id: entryId }
  );
  announce(assetId, saved ?? null);
  return saved ?? null;
}
