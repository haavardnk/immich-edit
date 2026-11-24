import { getJson, sendJson } from './client';
import type { AssetSummary } from '#lib/types/album.ts';

export type StyleSource = 'custom' | 'immich' | 'none';

export interface MapConfig {
  light: string | null;
  dark: string | null;
  source: StyleSource;
  search: boolean;
}

export interface MapMarker {
  id: string;
  lat: number;
  lon: number;
  city: string | null;
  state: string | null;
  country: string | null;
}

export interface PlaceName {
  city: string | null;
  state: string | null;
  country: string | null;
}

export interface Place {
  name: string;
  detail: string;
  lat: number;
  lon: number;
}

export interface MarkerFilter {
  favorite?: boolean;
  after?: string;
  before?: string;
}

export interface MapSettings {
  styleLight: string | null;
  styleDark: string | null;
  geocoderUrl: string;
}

export function getMapConfig(): Promise<MapConfig> {
  return getJson('/api/map/config');
}

export function getMapMarkers(filter: MarkerFilter): Promise<MapMarker[]> {
  const params = new URLSearchParams();
  if (filter.favorite) params.set('favorite', 'true');
  if (filter.after) params.set('after', filter.after);
  if (filter.before) params.set('before', filter.before);
  const query = params.toString();
  return getJson(query ? `/api/map/markers?${query}` : '/api/map/markers');
}

export function reverseGeocode(lat: number, lon: number): Promise<PlaceName | null> {
  const params = new URLSearchParams({ lat: String(lat), lon: String(lon) });
  return getJson(`/api/map/reverse-geocode?${params.toString()}`);
}

export function searchPlaces(query: string, signal?: AbortSignal): Promise<Place[]> {
  const params = new URLSearchParams({ q: query });
  return getJson(`/api/map/search?${params.toString()}`, signal ? { signal } : undefined);
}

export function assetSummaries(ids: readonly string[]): Promise<AssetSummary[]> {
  return sendJson('POST', '/api/assets/summaries', { ids });
}

export function setAssetLocation(
  ids: readonly string[],
  latitude: number,
  longitude: number
): Promise<void> {
  return sendJson('PUT', '/api/assets/location', { ids, latitude, longitude });
}

export interface LocatedAsset {
  id: string;
  latitude: number;
  longitude: number;
}

export function setAssetLocations(items: readonly LocatedAsset[]): Promise<{ failed: string[] }> {
  return sendJson('PUT', '/api/assets/locations', { items });
}

export function getMapSettings(): Promise<MapSettings> {
  return getJson('/api/admin/map-settings');
}

export function setMapSettings(settings: MapSettings): Promise<MapSettings> {
  return sendJson('PUT', '/api/admin/map-settings', settings);
}
