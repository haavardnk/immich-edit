import { getJson } from './client';

export interface TagSummary {
  id: string;
  name: string;
  value: string;
  createdAt: string | null;
}

export function listTags(): Promise<TagSummary[]> {
  return getJson('/api/tags');
}
