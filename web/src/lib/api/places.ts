import { getJson, sendJson, url } from './client';

export interface Spot {
  lat: number;
  lon: number;
  city: string | null;
  state: string | null;
  country: string | null;
}

export interface SavedPlace extends Spot {
  id: string;
  name: string;
  created_at: string;
  used_at: string;
}

export interface RecentPlace extends Spot {
  id: string;
  used_at: string;
}

export interface Places {
  saved: SavedPlace[];
  recent: RecentPlace[];
}

export type SavedPlaceInput = Spot & { name: string };

export function listPlaces(): Promise<Places> {
  return getJson('/api/places');
}

export function createPlace(input: SavedPlaceInput): Promise<SavedPlace> {
  return sendJson('POST', '/api/places', input);
}

export function updatePlace(id: string, input: SavedPlaceInput): Promise<SavedPlace> {
  return sendJson('PUT', url`/api/places/${id}`, input);
}

export function deletePlace(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/places/${id}`, undefined);
}

export function markPlaceUsed(id: string): Promise<void> {
  return sendJson('POST', url`/api/places/${id}/used`, undefined);
}

export function rememberPlace(spot: Spot): Promise<RecentPlace[]> {
  return sendJson('POST', '/api/places/recent', spot);
}

export function clearRecentPlaces(): Promise<void> {
  return sendJson('DELETE', '/api/places/recent', undefined);
}
