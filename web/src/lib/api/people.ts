import { getJson, url } from './client';

export interface PersonSummary {
  id: string;
  name: string;
  thumbnailPath: string;
  isHidden: boolean;
  updatedAt: string | null;
  assetCount?: number | null;
}

export function listPeople(): Promise<PersonSummary[]> {
  return getJson('/api/people');
}

export function personThumbUrl(id: string): string {
  return url`/api/people/${id}/thumb`;
}
