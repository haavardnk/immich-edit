import type { CropRect } from '#lib/types/edits.ts';

export const CROP_FILL = 0.92;
const MAX_VIEW_ZOOM = 4;

export type CropView = { zoom: number; cx: number; cy: number };
type Size = { w: number; h: number };

export const IDENTITY_VIEW: CropView = { zoom: 1, cx: 0.5, cy: 0.5 };

export function cropFit(box: Size, bbox: Size): number {
  return Math.min(
    (box.w * CROP_FILL) / Math.max(bbox.w, 1),
    (box.h * CROP_FILL) / Math.max(bbox.h, 1)
  );
}

export function previewZoom(crop: CropRect, bbox: Size, fit: number, box: Size): number {
  const w = Math.max(crop.w * bbox.w * fit, 1e-6);
  const h = Math.max(crop.h * bbox.h * fit, 1e-6);
  return Math.min(box.w / w, box.h / h);
}

function centred(crop: CropRect, zoom: number): CropView {
  return { zoom, cx: crop.x + crop.w / 2, cy: crop.y + crop.h / 2 };
}

export function framedView(crop: CropRect, bbox: Size, fit: number, box: Size): CropView {
  const zoom = previewZoom(crop, bbox, fit, box) * CROP_FILL;
  return centred(crop, Math.min(MAX_VIEW_ZOOM, Math.max(1, zoom)));
}

export function previewView(crop: CropRect, bbox: Size, fit: number, box: Size): CropView {
  return centred(crop, previewZoom(crop, bbox, fit, box));
}
