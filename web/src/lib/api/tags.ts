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

export interface NewTag {
  name: string;
  parentId?: string;
  color?: string;
}

export interface TagPatch {
  name?: string;
  color?: string | null;
}

export function createTag(tag: NewTag): Promise<TagSummary> {
  return sendJson('POST', '/api/tags', tag);
}

export function updateTag(id: string, patch: TagPatch): Promise<TagSummary> {
  return sendJson('PATCH', url`/api/tags/${id}`, patch);
}

export function deleteTag(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/tags/${id}`, undefined);
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
