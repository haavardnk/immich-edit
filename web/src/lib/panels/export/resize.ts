export type ExportResize =
  | { mode: 'dimensions'; width: number | null; height: number | null; enlarge: boolean }
  | { mode: 'megapixels'; megapixels: number; enlarge: boolean }
  | { mode: 'percent'; percent: number };

export type ResizeMode = ExportResize['mode'];

export interface PixelSize {
  w: number;
  h: number;
}

export const EXPORT_MAX_EDGE = 65535;
export const MAX_RESIZE_PERCENT = 400;
export const MAX_RESIZE_MEGAPIXELS = 1000;
export const DEFAULT_RESIZE_BOX = 2048;
export const DEFAULT_RESIZE_MEGAPIXELS = 12;
export const DEFAULT_RESIZE_PERCENT = 50;

function validEdge(edge: number | null): boolean {
  return edge === null || (Number.isInteger(edge) && edge >= 1 && edge <= EXPORT_MAX_EDGE);
}

export function resizeError(resize: ExportResize | null): string | null {
  if (!resize) return null;
  if (resize.mode === 'percent') {
    const valid =
      Number.isFinite(resize.percent) &&
      resize.percent >= 1 &&
      resize.percent <= MAX_RESIZE_PERCENT;
    return valid ? null : `Enter a percentage from 1 to ${MAX_RESIZE_PERCENT}`;
  }
  if (resize.mode === 'megapixels') {
    const valid =
      Number.isFinite(resize.megapixels) &&
      resize.megapixels > 0 &&
      resize.megapixels <= MAX_RESIZE_MEGAPIXELS;
    return valid ? null : `Enter megapixels above 0 and at most ${MAX_RESIZE_MEGAPIXELS}`;
  }
  if (resize.width === null && resize.height === null) return 'Enter a width or a height';
  if (!validEdge(resize.width) || !validEdge(resize.height)) {
    return `Enter whole pixels from 1 to ${EXPORT_MAX_EDGE}`;
  }
  return null;
}

export function linkedEdge(edge: number, from: number, to: number): number {
  return Math.max(1, Math.round((edge * to) / Math.max(from, 1)));
}

function longestWithin(cap: number, long: number, short: number): number {
  let edge = Math.ceil(((cap + 0.5) * long) / short) - 1;
  while (edge > 1 && Math.round(short * (edge / long)) > cap) edge -= 1;
  return edge;
}

function targetEdge(crop: PixelSize, resize: ExportResize): { edge: number; enlarge: boolean } {
  const long = Math.max(crop.w, crop.h, 1);
  const short = Math.max(Math.min(crop.w, crop.h), 1);
  if (resize.mode === 'percent') {
    return { edge: Math.round((long * resize.percent) / 100), enlarge: resize.percent > 100 };
  }
  if (resize.mode === 'megapixels') {
    const edge = Math.round(Math.sqrt((resize.megapixels * 1e6 * long) / short));
    return { edge, enlarge: resize.enlarge };
  }
  const landscape = crop.w >= crop.h;
  const longCap = (landscape ? resize.width : resize.height) ?? EXPORT_MAX_EDGE;
  const shortCap = landscape ? resize.height : resize.width;
  const fromShort = shortCap === null ? EXPORT_MAX_EDGE : longestWithin(shortCap, long, short);
  return { edge: Math.min(longCap, fromShort), enlarge: resize.enlarge };
}

export function resizedSize(crop: PixelSize, resize: ExportResize | null): PixelSize {
  if (!resize || resizeError(resize)) return crop;
  const target = targetEdge(crop, resize);
  const edge = Math.min(EXPORT_MAX_EDGE, Math.max(1, target.edge));
  const long = Math.max(crop.w, crop.h, 1);
  if (edge === long || (edge > long && !target.enlarge)) return crop;
  const scale = edge / long;
  return {
    w: Math.max(1, Math.round(crop.w * scale)),
    h: Math.max(1, Math.round(crop.h * scale))
  };
}
