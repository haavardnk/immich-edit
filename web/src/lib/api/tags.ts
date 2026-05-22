import { getJson, sendJson } from './client';

export interface TagSummary {
  id: string;
  name: string;
  value: string;
  parentId?: string | null;
  color?: string | null;
  createdAt: string | null;
  updatedAt?: string | null;
}

export function listTags(): Promise<TagSummary[]> {
  return getJson('/api/tags');
}

export function upsertTags(tags: string[]): Promise<TagSummary[]> {
  return sendJson('PUT', '/api/tags', { tags });
}

export function addTagToAsset(tagId: string, assetId: string): Promise<unknown> {
  return sendJson('PUT', `/api/tags/${tagId}/assets/${assetId}`, {});
}

export function removeTagFromAsset(tagId: string, assetId: string): Promise<unknown> {
  return sendJson('DELETE', `/api/tags/${tagId}/assets/${assetId}`, {});
}
