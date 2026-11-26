import { cameraKey } from '#lib/metadata/cameras.ts';
import {
  isUsableZone,
  parsedTime,
  wallToInstant,
  zoneOffsetMinutes
} from '#lib/metadata/photoTime.ts';
import type { AssetSummary } from '#lib/types/album.ts';

const MINUTE = 60_000;

export interface ClockPhoto {
  id: string;
  name: string;
  camera: string;
  wall: number;
  stored: number | null;
  zone: string | null;
}

export interface ClockTarget {
  instant: number;
  zone: string;
}

export interface ZoneChoice {
  takenIn: string | null;
  clock: string | null;
  fallback: string;
}

export type CaptureWrite =
  { id: string; minutes: number; timeZone?: string } | { id: string; at: string; timeZone: string };

export function clockPhoto(asset: AssetSummary): ClockPhoto | null {
  const exif = asset.exifInfo;
  const stored = parsedTime(exif?.dateTimeOriginal);
  const wall = parsedTime(asset.localDateTime) ?? stored ?? parsedTime(asset.fileCreatedAt);
  if (wall === null) return null;
  const recorded = exif?.timeZone?.trim();
  const zone = recorded && isUsableZone(recorded) ? recorded : null;
  return {
    id: asset.id,
    name: asset.originalFileName,
    camera: cameraKey(asset),
    wall,
    stored,
    zone
  };
}

export function wallOf(target: ClockTarget): number {
  return target.instant + zoneOffsetMinutes(target.zone, target.instant) * MINUTE;
}

export function shifted(photo: ClockPhoto, shiftMs: number, zones: ZoneChoice): ClockTarget | null {
  if (shiftMs === 0 && zones.takenIn === null) return null;
  const wall = photo.wall + shiftMs;
  if (zones.takenIn !== null) {
    return { instant: wallToInstant(wall, zones.clock ?? zones.takenIn), zone: zones.takenIn };
  }
  if (photo.zone && photo.stored !== null) {
    return { instant: photo.stored + shiftMs, zone: photo.zone };
  }
  const zone = photo.zone ?? zones.fallback;
  return { instant: wallToInstant(wall, zone), zone };
}

function readZone(photo: ClockPhoto, zones: ZoneChoice): string {
  return zones.takenIn === null ? (photo.zone ?? zones.fallback) : (zones.clock ?? zones.takenIn);
}

export function baseMoment(photo: ClockPhoto, zones: ZoneChoice): number {
  if (zones.takenIn === null && photo.zone && photo.stored !== null) return photo.stored;
  return wallToInstant(photo.wall, readZone(photo, zones));
}

export function correctionTo(photo: ClockPhoto, moment: number, zones: ZoneChoice): number {
  if (zones.takenIn === null && photo.zone && photo.stored !== null) return moment - photo.stored;
  return wallOf({ instant: moment, zone: readZone(photo, zones) }) - photo.wall;
}

export function shownZone(photo: ClockPhoto, zones: ZoneChoice): string {
  return zones.takenIn ?? photo.zone ?? zones.fallback;
}

export function setTo(wall: number, zone: string): ClockTarget {
  return { instant: wallToInstant(wall, zone), zone };
}

export function writeFor(
  id: string,
  stored: number | null,
  zone: string | null,
  target: ClockTarget
): CaptureWrite | null {
  const sameZone = zone === target.zone;
  if (stored !== null) {
    const delta = target.instant - stored;
    if (delta === 0 && sameZone) return null;
    if (delta % MINUTE === 0) {
      const minutes = delta / MINUTE + 0;
      return sameZone ? { id, minutes } : { id, minutes, timeZone: target.zone };
    }
  }
  return { id, at: new Date(target.instant).toISOString(), timeZone: target.zone };
}

export function restoreTarget(photo: ClockPhoto, after: ClockTarget): ClockTarget {
  if (photo.zone && photo.stored !== null) return { instant: photo.stored, zone: photo.zone };
  const zone = photo.zone ?? after.zone;
  return { instant: wallToInstant(photo.wall, zone), zone };
}

export interface ClockChange {
  photo: ClockPhoto;
  target: ClockTarget;
}

export interface CapturePlan {
  write: CaptureWrite[];
  undo: CaptureWrite[];
  restored: Map<string, ClockTarget>;
}

export function planWrites(changes: readonly ClockChange[]): CapturePlan {
  const write: CaptureWrite[] = [];
  const undo: CaptureWrite[] = [];
  const restored = new Map<string, ClockTarget>();
  for (const { photo, target } of changes) {
    const forward = writeFor(photo.id, photo.stored, photo.zone, target);
    if (!forward) continue;
    write.push(forward);
    const before = restoreTarget(photo, target);
    const back = writeFor(photo.id, target.instant, target.zone, before);
    if (!back) continue;
    undo.push(back);
    restored.set(photo.id, before);
  }
  return { write, undo, restored };
}

export interface PreviewRow {
  photo: ClockPhoto;
  before: number;
  after: number;
  zone: string | null;
  changed: boolean;
  reordered: boolean;
}

function byTime(a: { time: number; id: string }, b: { time: number; id: string }): number {
  return a.time - b.time || a.id.localeCompare(b.id);
}

export function previewRows(
  photos: readonly ClockPhoto[],
  targets: ReadonlyMap<string, ClockTarget>
): PreviewRow[] {
  const rows = photos.map((photo) => {
    const target = targets.get(photo.id);
    const changed =
      target !== undefined && writeFor(photo.id, photo.stored, photo.zone, target) !== null;
    return {
      photo,
      before: photo.wall,
      after: changed && target ? wallOf(target) : photo.wall,
      zone: changed && target ? target.zone : photo.zone,
      changed,
      reordered: false
    };
  });
  const rank = new Map(
    rows
      .map((row) => ({ id: row.photo.id, time: row.before }))
      .sort(byTime)
      .map((row, index) => [row.id, index])
  );
  const ordered = rows
    .map((row) => ({ row, id: row.photo.id, time: row.after }))
    .sort(byTime)
    .map(({ row }) => row);
  return ordered.map((row, index) => ({
    ...row,
    reordered: row.changed && rank.get(row.photo.id) !== index
  }));
}
