import type { PlaceName } from '#lib/api/map.ts';
import type { GeoLocation } from '#lib/browse/locationOps.ts';
import { isRecord } from '#lib/utils/isRecord.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

const KEY = 'immich-edit:copiedLocation';

function text(value: unknown): string | null {
  return typeof value === 'string' ? value : null;
}

function placeOf(value: unknown): PlaceName | null {
  if (!isRecord(value)) return null;
  return { city: text(value.city), state: text(value.state), country: text(value.country) };
}

export function storedLocation(raw: unknown): GeoLocation | null {
  if (!isRecord(raw)) return null;
  const { lat, lon } = raw;
  if (typeof lat !== 'number' || typeof lon !== 'number') return null;
  if (!Number.isFinite(lat) || !Number.isFinite(lon)) return null;
  if (Math.abs(lat) > 90 || Math.abs(lon) > 180) return null;
  return { lat, lon, place: placeOf(raw.place) };
}

export function coordinateText(location: Pick<GeoLocation, 'lat' | 'lon'>): string {
  return `${location.lat.toFixed(6)}, ${location.lon.toFixed(6)}`;
}

async function writeSystemClipboard(value: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(value);
  } catch {
    return;
  }
}

class LocationClipboard {
  copied = $state.raw<GeoLocation | null>(storedLocation(readStored(KEY, 'session')));

  copy = (location: GeoLocation): void => {
    this.copied = location;
    writeStored(KEY, location, 'session');
    void writeSystemClipboard(coordinateText(location));
  };
}

export const locationClipboard = new LocationClipboard();
