import { sendJson } from './client';

export interface TrashResult {
  count: number;
}

export function trashAssets(ids: string[]): Promise<TrashResult> {
  return sendJson('POST', '/api/trash', { ids });
}

export function restoreAssets(ids: string[]): Promise<TrashResult> {
  return sendJson('POST', '/api/trash/restore', { ids });
}
