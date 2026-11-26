import { isZone, offsetLabel, zoneOffsetMinutes } from '#lib/metadata/photoTime.ts';
import { DateTime } from 'luxon';

export function shownText(wall: number, withDay: boolean): string {
  const at = DateTime.fromMillis(wall, { zone: 'utc' });
  return at.toFormat(withDay ? 'd LLL yyyy, HH:mm:ss' : 'HH:mm:ss');
}

export function sameDay(a: number, b: number): boolean {
  return Math.floor(a / 86_400_000) === Math.floor(b / 86_400_000);
}

export function zoneName(zone: string): string {
  const place = zone.includes('/') ? zone.slice(zone.lastIndexOf('/') + 1) : zone;
  return place.replaceAll('_', ' ');
}

export interface ShotZone {
  zone: string;
  at: number;
}

export function asShotText(shots: readonly ShotZone[]): string {
  const [first] = shots;
  if (!first) return 'As shot';
  const offsets = new Set(shots.map((shot) => offsetLabel(zoneOffsetMinutes(shot.zone, shot.at))));
  const offset = offsetLabel(zoneOffsetMinutes(first.zone, first.at));
  if (shots.every((shot) => shot.zone === first.zone) && isZone(first.zone)) {
    return `As shot (${zoneName(first.zone)} · ${offset})`;
  }
  return offsets.size > 1 ? `As shot (${offsets.size} zones)` : `As shot (${offset})`;
}

export function clockExample(before: number, after: number, zone: string): string {
  const time = (wall: number): string =>
    DateTime.fromMillis(wall, { zone: 'utc' }).toFormat('HH:mm');
  return `${time(before)} on the camera → ${time(after)} in ${zoneName(zone)}`;
}
