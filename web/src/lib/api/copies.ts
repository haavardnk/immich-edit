import { getJson, sendJson, url } from './client';

export interface CopyRecord {
  id: string;
  source_asset_id: string;
  name: string | null;
  created_at: string;
}

export function listCopies(assetId: string): Promise<CopyRecord[]> {
  return getJson(url`/api/assets/${assetId}/copies`);
}

export function createCopy(
  assetId: string,
  body: { name?: string; from?: 'current' | 'neutral' } = {}
): Promise<CopyRecord> {
  return sendJson('POST', url`/api/assets/${assetId}/copies`, body);
}

export function renameCopy(id: string, name: string | null): Promise<CopyRecord> {
  return sendJson('PATCH', url`/api/copies/${id}`, { name });
}

export function deleteCopy(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/copies/${id}`, undefined);
}
