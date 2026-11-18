import type { CropHandle } from '#lib/utils/geom.ts';

export const HANDLE_HIT = 12;

export const HANDLE_DOT = {
  crop: 6,
  corner: 7,
  guideEnd: 6,
  maskEnd: 8,
  maskCentre: 6,
  maskEdge: 6,
  maskGrip: 5,
  maskFeather: 5,
  polygonCorner: 7,
  polygonMid: 5,
  draftStart: 8,
  draftPoint: 5
} as const;

export const RETOUCH_SOURCE_HIT = 8;

export const CROP_HANDLES: readonly CropHandle[] = ['n', 'e', 's', 'w', 'nw', 'ne', 'se', 'sw'];

export function boxHandleHit(w: number, h: number): number {
  return Math.min(HANDLE_HIT, Math.max(HANDLE_DOT.crop, Math.min(w, h) / 4));
}

export type SpacedHandle = { cx: number; cy: number; r: number; hit?: number };

function nearest(handles: readonly SpacedHandle[], index: number): number {
  const self = handles[index];
  if (!self) return Infinity;
  return handles.reduce(
    (best, other, i) =>
      i === index ? best : Math.min(best, Math.hypot(other.cx - self.cx, other.cy - self.cy)),
    Infinity
  );
}

export function spacedHandles<T extends SpacedHandle>(
  handles: readonly T[]
): (T & { hit: number })[] {
  return handles.map((handle, i) => ({
    ...handle,
    hit: handle.hit ?? Math.max(handle.r, Math.min(HANDLE_HIT, nearest(handles, i) / 2))
  }));
}
