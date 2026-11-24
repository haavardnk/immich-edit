import { reverseGeocode, type PlaceName } from '#lib/api/map.ts';
import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';

const CELL_DEGREES = 0.01;
const CONCURRENCY = 4;

export function cellOf(point: GeoPoint): string {
  return `${Math.round(point.lat / CELL_DEGREES)},${Math.round(point.lon / CELL_DEGREES)}`;
}

export async function namePlaces(
  points: readonly GeoPoint[]
): Promise<Map<string, PlaceName | null>> {
  const cells = new Map<string, GeoPoint>();
  for (const point of points) if (!cells.has(cellOf(point))) cells.set(cellOf(point), point);
  const names = new Map<string, PlaceName | null>();
  const queue = [...cells];
  async function worker(): Promise<void> {
    for (let next = queue.shift(); next; next = queue.shift()) {
      const [cell, point] = next;
      names.set(cell, await reverseGeocode(point.lat, point.lon).catch(() => null));
    }
  }
  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, queue.length) }, worker));
  return names;
}
