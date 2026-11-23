import type { Spot } from '#lib/api/places.ts';
import type { GeoLocation } from '#lib/browse/locationOps.ts';
import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';

const EARTH_RADIUS_METRES = 6_371_008.8;

export const SAME_SPOT_METRES = 50;

export function distanceMetres(a: GeoPoint, b: GeoPoint): number {
  const latA = (a.lat * Math.PI) / 180;
  const latB = (b.lat * Math.PI) / 180;
  const halfLat = (latB - latA) / 2;
  const halfLon = ((b.lon - a.lon) * Math.PI) / 180 / 2;
  const h = Math.sin(halfLat) ** 2 + Math.cos(latA) * Math.cos(latB) * Math.sin(halfLon) ** 2;
  return 2 * EARTH_RADIUS_METRES * Math.asin(Math.min(1, Math.sqrt(h)));
}

export function spotOf(location: GeoLocation): Spot {
  return {
    lat: location.lat,
    lon: location.lon,
    city: location.place?.city ?? null,
    state: location.place?.state ?? null,
    country: location.place?.country ?? null
  };
}

export function locationOfSpot(spot: Spot): GeoLocation {
  return {
    lat: spot.lat,
    lon: spot.lon,
    place: { city: spot.city, state: spot.state, country: spot.country }
  };
}

export function sameSpot(a: GeoPoint, b: GeoPoint): boolean {
  return distanceMetres(a, b) <= SAME_SPOT_METRES;
}
