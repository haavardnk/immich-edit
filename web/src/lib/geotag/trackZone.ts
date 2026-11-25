import tzLookup from '@photostructure/tz-lookup';
import type { TrackPoint } from '#lib/geotag/trackFile.ts';
import { isZone } from '#lib/geotag/photoTime.ts';

export function zoneAt(lat: number, lon: number): string | null {
  try {
    const zone = tzLookup(lat, lon);
    return isZone(zone) ? zone : null;
  } catch {
    return null;
  }
}

export function trackZone(segments: readonly (readonly TrackPoint[])[]): string | null {
  const points = segments.flat();
  const middle = points[Math.floor(points.length / 2)];
  return middle ? zoneAt(middle.lat, middle.lon) : null;
}
