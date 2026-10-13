import type { ExifInfo } from '$lib/types/asset';

export function fmtAperture(v: number | null): string | null {
  if (v == null) return null;
  return `f/${v.toFixed(v < 10 ? 1 : 0)}`;
}

export function fmtFocal(v: number | null): string | null {
  if (v == null) return null;
  return `${v.toFixed(0)}mm`;
}

export function fmtShutter(v: string | null): string | null {
  if (!v) return null;
  const n = Number(v);
  if (!Number.isFinite(n) || n <= 0) return v;
  if (n >= 1) return `${n.toFixed(1)}s`;
  return `1/${Math.round(1 / n)}s`;
}

export function fmtDim(w: number | null, h: number | null): string | null {
  if (!w || !h) return null;
  return `${w} × ${h}`;
}

export function fmtSize(b: number | null): string | null {
  if (!b) return null;
  const mb = b / (1024 * 1024);
  if (mb >= 1) return `${mb.toFixed(1)} MB`;
  const kb = b / 1024;
  return `${kb.toFixed(0)} KB`;
}

const UTC_OFFSET = /^UTC([+-])(\d{1,2})(?::(\d{2}))?$/;

export interface Dated {
  localDateTime?: string | null;
  fileCreatedAt?: string | null;
  exifInfo: ExifInfo | null;
}

export interface CaptureTime {
  at: Date;
  timeZone?: string;
}

function parseDate(s: string | null | undefined): Date | null {
  if (!s) return null;
  const d = new Date(s);
  return Number.isNaN(d.getTime()) ? null : d;
}

function knownZone(zone: string): boolean {
  try {
    new Intl.DateTimeFormat(undefined, { timeZone: zone });
    return true;
  } catch {
    return false;
  }
}

function inZone(at: Date, zone: string | null | undefined): CaptureTime {
  const offset = zone ? UTC_OFFSET.exec(zone) : null;
  if (offset) {
    const minutes = Number(offset[2]) * 60 + Number(offset[3] ?? 0);
    const sign = offset[1] === '-' ? -1 : 1;
    return { at: new Date(at.getTime() + sign * minutes * 60_000), timeZone: 'UTC' };
  }
  return { at, timeZone: zone && knownZone(zone) ? zone : 'UTC' };
}

export function captureTime(asset: Dated): CaptureTime | null {
  const local = parseDate(asset.localDateTime);
  if (local) return { at: local, timeZone: 'UTC' };
  const original = parseDate(asset.exifInfo?.dateTimeOriginal);
  if (original) return inZone(original, asset.exifInfo?.timeZone);
  const created = parseDate(asset.fileCreatedAt);
  return created ? { at: created } : null;
}

export function fmtDate(t: CaptureTime | null): string | null {
  return t ? t.at.toLocaleString(undefined, { timeZone: t.timeZone }) : null;
}

export function fmtDay(t: CaptureTime | null): string | null {
  return t ? t.at.toLocaleDateString(undefined, { timeZone: t.timeZone }) : null;
}

export function fmtCamera(make: string | null, model: string | null): string | null {
  const v = [make, model].filter(Boolean).join(' ').trim();
  return v.length > 0 ? v : null;
}

export interface ExifRow {
  key: string;
  value: string;
}

export function exifDetailRows(asset: Dated | null): ExifRow[] {
  const exif = asset?.exifInfo;
  if (!exif) return [];
  const items: Array<[string, string | null]> = [
    ['Camera', fmtCamera(exif.make, exif.model)],
    ['Lens', exif.lensModel],
    ['Aperture', fmtAperture(exif.fNumber)],
    ['Focal', fmtFocal(exif.focalLength)],
    ['Shutter', fmtShutter(exif.exposureTime)],
    ['ISO', exif.iso != null ? String(exif.iso) : null],
    ['Dimensions', fmtDim(exif.exifImageWidth, exif.exifImageHeight)],
    ['Size', fmtSize(exif.fileSizeInByte)],
    ['Taken', exif.dateTimeOriginal ? fmtDate(captureTime(asset)) : null]
  ];
  return items
    .filter((r): r is [string, string] => r[1] != null && r[1] !== '')
    .map(([key, value]) => ({ key, value }));
}
