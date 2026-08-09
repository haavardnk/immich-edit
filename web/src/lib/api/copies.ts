import { getJson, sendJson } from './client';

export interface CopyRecord {
  id: string;
  source_asset_id: string;
  name: string | null;
  created_at: string;
}

export function listCopies(assetId: string): Promise<CopyRecord[]> {
  return getJson(`/api/assets/${assetId}/copies`);
}

export function createCopy(
  assetId: string,
  body: { name?: string; from?: 'current' | 'neutral' } = {}
): Promise<CopyRecord> {
  return sendJson('POST', `/api/assets/${assetId}/copies`, body);
}

export function renameCopy(id: string, name: string | null): Promise<CopyRecord> {
  return sendJson('PATCH', `/api/copies/${id}`, { name });
}

export function deleteCopy(id: string): Promise<void> {
  return sendJson('DELETE', `/api/copies/${id}`, undefined);
}
