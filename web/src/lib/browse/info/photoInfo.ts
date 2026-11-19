import type { ExifInfo } from '#lib/types/asset.ts';
import { captureTime, exifDetailRows, type Dated, type ExifRow } from '#lib/utils/exif.ts';

export interface ExposureCell {
  label: string;
  value: string;
}

export interface Place {
  name: string;
  region: string | null;
  mapUrl: string | null;
}

export interface InfoSubject extends Dated {
  originalFileName: string;
}

const EXPOSURE = ['Aperture', 'Shutter', 'ISO', 'Focal'] as const;
const EXPOSURE_KEYS = new Set<string>(EXPOSURE);
const MISSING = '—';

export function exposureCells(asset: Dated): ExposureCell[] {
  const rows = new Map(exifDetailRows(asset).map((row) => [row.key, row.value]));
  if (!EXPOSURE.some((key) => rows.has(key))) return [];
  return EXPOSURE.map((label) => ({ label, value: rows.get(label) ?? MISSING }));
}

export function detailRows(asset: InfoSubject): ExifRow[] {
  const type = fileType(asset.originalFileName);
  return exifDetailRows(asset)
    .filter((row) => !EXPOSURE_KEYS.has(row.key))
    .map((row) =>
      row.key === 'Size' && type ? { key: row.key, value: `${row.value} · ${type}` } : row
    );
}

export function fileType(name: string): string | null {
  const dot = name.lastIndexOf('.');
  if (dot <= 0 || dot === name.length - 1) return null;
  return name.slice(dot + 1).toUpperCase();
}

export function placeOf(exif: ExifInfo | null): Place | null {
  if (!exif) return null;
  const names = [exif.city, exif.state, exif.country].filter(
    (value): value is string => typeof value === 'string' && value.trim() !== ''
  );
  const coords = coordinates(exif);
  const name = names[0] ?? (coords ? coordinateLabel(coords.lat, coords.lon) : null);
  if (!name) return null;
  const region = names.slice(1).join(', ');
  return {
    name,
    region: region === '' ? null : region,
    mapUrl: coords ? mapUrl(coords.lat, coords.lon) : null
  };
}

export function mapUrl(lat: number, lon: number): string {
  const y = lat.toFixed(5);
  const x = lon.toFixed(5);
  return `https://www.openstreetmap.org/?mlat=${y}&mlon=${x}#map=14/${y}/${x}`;
}

export function takenLabel(asset: Dated): string | null {
  const taken = captureTime(asset);
  if (!taken) return null;
  const zone = { timeZone: taken.timeZone };
  const day = taken.at.toLocaleDateString(undefined, {
    ...zone,
    weekday: 'short',
    day: 'numeric',
    month: 'short',
    year: 'numeric'
  });
  const time = taken.at.toLocaleTimeString(undefined, {
    ...zone,
    hour: '2-digit',
    minute: '2-digit'
  });
  return `${day} · ${time}`;
}

function coordinates(exif: ExifInfo): { lat: number; lon: number } | null {
  const lat = exif.latitude;
  const lon = exif.longitude;
  if (typeof lat !== 'number' || typeof lon !== 'number') return null;
  if (!Number.isFinite(lat) || !Number.isFinite(lon)) return null;
  if (Math.abs(lat) > 90 || Math.abs(lon) > 180) return null;
  return { lat, lon };
}

function coordinateLabel(lat: number, lon: number): string {
  const ns = lat >= 0 ? 'N' : 'S';
  const ew = lon >= 0 ? 'E' : 'W';
  return `${Math.abs(lat).toFixed(4)}° ${ns}, ${Math.abs(lon).toFixed(4)}° ${ew}`;
}
