import { getJson, sendJson, url } from './client';

export interface TagSummary {
  id: string;
  name: string;
  value: string;
  parentId?: string | null;
  color?: string | null;
  createdAt: string | null;
  updatedAt?: string | null;
  assetCount?: number | null;
}

export function listTags(): Promise<TagSummary[]> {
  return getJson('/api/tags');
}

export function upsertTags(tags: string[]): Promise<TagSummary[]> {
  return sendJson('PUT', '/api/tags', { tags });
}

export interface BulkIdResponse {
  id: string;
  success: boolean;
  error?: string | null;
}

export function addTagToAsset(tagId: string, assetId: string): Promise<BulkIdResponse[]> {
  return sendJson('PUT', url`/api/tags/${tagId}/assets/${assetId}`, {});
}

export function removeTagFromAsset(tagId: string, assetId: string): Promise<BulkIdResponse[]> {
  return sendJson('DELETE', url`/api/tags/${tagId}/assets/${assetId}`, {});
}
