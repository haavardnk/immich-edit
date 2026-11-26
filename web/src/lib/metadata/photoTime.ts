import { DateTime, IANAZone } from 'luxon';
import type { AssetSummary } from '#lib/types/album.ts';

export function isZone(zone: string): boolean {
  return IANAZone.isValidZone(zone);
}

export function isUsableZone(zone: string): boolean {
  return DateTime.now().setZone(zone).isValid;
}

export function zoneOffsetMinutes(zone: string, instant: number): number {
  return DateTime.fromMillis(instant, { zone }).offset;
}

export function wallToInstant(wall: number, zone: string): number {
  return DateTime.fromMillis(wall, { zone: 'utc' })
    .setZone(zone, { keepLocalTime: true })
    .toMillis();
}

export interface CameraTime {
  instant: number;
  recorded: boolean;
}

function parsed(value: string | null | undefined): number | null {
  if (!value) return null;
  const time = Date.parse(value);
  return Number.isFinite(time) ? time : null;
}

export function cameraTime(asset: AssetSummary, zone: string): CameraTime | null {
  const exif = asset.exifInfo;
  const original = parsed(exif?.dateTimeOriginal);
  if (exif?.timeZone && original !== null) return { instant: original, recorded: true };
  const wall = parsed(asset.localDateTime) ?? original ?? parsed(asset.fileCreatedAt);
  return wall === null ? null : { instant: wallToInstant(wall, zone), recorded: false };
}

export function offsetLabel(minutes: number): string {
  const sign = minutes < 0 ? '−' : '+';
  const total = Math.abs(minutes);
  const hours = Math.floor(total / 60);
  const rest = total % 60;
  return `UTC${sign}${hours}${rest ? `:${String(rest).padStart(2, '0')}` : ''}`;
}

export function zoneLabel(zone: string, at: number): string {
  return `${zone.replaceAll('_', ' ')} · ${offsetLabel(zoneOffsetMinutes(zone, at))}`;
}

export function deviceZone(): string {
  return DateTime.local().zoneName ?? 'UTC';
}
