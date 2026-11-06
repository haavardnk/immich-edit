import { getJson, sendJson, url } from './client';

export interface SavedView {
  id: string;
  name: string;
  url: string;
  created_at: string;
  updated_at: string;
}

export interface SavedViewInput {
  name: string;
  url: string;
}

export function listSavedViews(): Promise<SavedView[]> {
  return getJson('/api/saved-views');
}

export function createSavedView(input: SavedViewInput): Promise<SavedView> {
  return sendJson('POST', '/api/saved-views', input);
}

export function updateSavedView(id: string, input: SavedViewInput): Promise<SavedView> {
  return sendJson('PUT', url`/api/saved-views/${id}`, input);
}

export function deleteSavedView(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/saved-views/${id}`, undefined);
}
