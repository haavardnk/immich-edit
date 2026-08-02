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

export function fmtDate(s: string | null): string | null {
  if (!s) return null;
  try {
    const d = new Date(s);
    if (Number.isNaN(d.getTime())) return s;
    return d.toLocaleString();
  } catch {
    return s;
  }
}

export function fmtCamera(make: string | null, model: string | null): string | null {
  const v = [make, model].filter(Boolean).join(' ').trim();
  return v.length > 0 ? v : null;
}

export interface ExifRow {
  key: string;
  value: string;
}

export function exifDetailRows(exif: ExifInfo | null): ExifRow[] {
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
    ['Taken', fmtDate(exif.dateTimeOriginal)]
  ];
  return items
    .filter((r): r is [string, string] => r[1] != null && r[1] !== '')
    .map(([key, value]) => ({ key, value }));
}

