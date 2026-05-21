import { getJson } from './client';

export interface PersonSummary {
  id: string;
  name: string;
  thumbnailPath: string;
  isHidden: boolean;
  updatedAt: string | null;
}

export function listPeople(): Promise<PersonSummary[]> {
  return getJson('/api/people');
}

export function personThumbUrl(id: string): string {
  return `/api/people/${id}/thumb`;
}
