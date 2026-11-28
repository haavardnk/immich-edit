import type { MapMarker, MarkerFilter, MarkerQuery } from '#lib/api/map.ts';
import { localSpan } from '#lib/browse/dateBounds.ts';

export interface LngLatBox {
  west: number;
  south: number;
  east: number;
  north: number;
}

export interface MapView {
  lat: number;
  lon: number;
  zoom: number;
}

export const WORLD_VIEW: MapView = { lat: 30, lon: 10, zoom: 1.4 };

const VIEW_ZOOM_MAX = 18;

export function inBox(marker: Pick<MapMarker, 'lat' | 'lon'>, box: LngLatBox): boolean {
  if (marker.lat < box.south || marker.lat > box.north) return false;
  if (box.west <= box.east) return marker.lon >= box.west && marker.lon <= box.east;
  return marker.lon >= box.west || marker.lon <= box.east;
}

export function markersIn(markers: readonly MapMarker[], box: LngLatBox): MapMarker[] {
  return markers.filter((marker) => inBox(marker, box));
}

export function boundsOf(markers: readonly Pick<MapMarker, 'lat' | 'lon'>[]): LngLatBox | null {
  const [first, ...rest] = markers;
  if (!first) return null;
  return rest.reduce<LngLatBox>(
    (box, { lat, lon }) => ({
      west: Math.min(box.west, lon),
      east: Math.max(box.east, lon),
      south: Math.min(box.south, lat),
      north: Math.max(box.north, lat)
    }),
    { west: first.lon, east: first.lon, south: first.lat, north: first.lat }
  );
}

export function placeOfMarkers(markers: readonly MapMarker[]): string {
  const counts = new Map<string, number>();
  for (const marker of markers) {
    const name = marker.city ?? marker.state ?? marker.country;
    if (name) counts.set(name, (counts.get(name) ?? 0) + 1);
  }
  const ranked = [...counts.entries()].sort((a, b) => b[1] - a[1]).map(([name]) => name);
  if (ranked.length === 0) return '';
  if (ranked.length === 1) return ranked[0] ?? '';
  return `${ranked[0]} and ${ranked[1]}`;
}

export function parseView(raw: string | null): MapView | null {
  const parts = raw?.split(',').map(Number);
  if (!parts || parts.length !== 3 || parts.some((part) => !Number.isFinite(part))) return null;
  const [lat = 0, lon = 0, zoom = 0] = parts;
  if (Math.abs(lat) > 90 || Math.abs(lon) > 180 || zoom < 0 || zoom > VIEW_ZOOM_MAX) return null;
  return { lat, lon, zoom };
}

export function formatView(view: MapView): string {
  return `${view.lat.toFixed(5)},${view.lon.toFixed(5)},${view.zoom.toFixed(2)}`;
}

const PHOTO_PARAM = 'photo';

export function mapHref(view: MapView, photo?: string): string {
  const pick = photo ? `&${PHOTO_PARAM}=${encodeURIComponent(photo)}` : '';
  return `/map?at=${formatView(view)}${pick}`;
}

export function readMapPhoto(params: Pick<URLSearchParams, 'get'>): string | null {
  return params.get(PHOTO_PARAM) || null;
}

export function dropMapPhoto(params: Pick<URLSearchParams, 'delete'>): void {
  params.delete(PHOTO_PARAM);
}

const DAY = /^\d{4}-\d{2}-\d{2}$/;

export function readMapFilter(params: Pick<URLSearchParams, 'get'>): MarkerFilter {
  const after = params.get('after');
  const before = params.get('before');
  return {
    favorite: params.get('favorite') === '1',
    after: after && DAY.test(after) ? after : undefined,
    before: before && DAY.test(before) ? before : undefined
  };
}

export function writeMapFilter(params: URLSearchParams, filter: MarkerFilter): void {
  if (filter.favorite) params.set('favorite', '1');
  else params.delete('favorite');
  for (const key of ['after', 'before'] as const) {
    const value = filter[key];
    if (value) params.set(key, value);
    else params.delete(key);
  }
}

export function markerQuery(filter: MarkerFilter): MarkerQuery {
  return {
    favorite: filter.favorite,
    taken: localSpan(filter.after ?? '', filter.before ?? '')
  };
}

export function sameView(a: MapView, b: MapView): boolean {
  return (
    Math.abs(a.lat - b.lat) < 1e-4 &&
    Math.abs(a.lon - b.lon) < 1e-4 &&
    Math.abs(a.zoom - b.zoom) < 0.01
  );
}
