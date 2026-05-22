import type { AspectLock, CropRect } from '../types/edits';
import { FULL_CROP } from '../types/edits';

export interface Size {
  w: number;
  h: number;
}

export interface Point {
  x: number;
  y: number;
}

export function degToRad(deg: number): number {
  return (deg * Math.PI) / 180;
}

export function rotatedBbox(sw: number, sh: number, angleDeg: number): Size {
  const a = degToRad(angleDeg);
  const c = Math.abs(Math.cos(a));
  const s = Math.abs(Math.sin(a));
  return { w: sw * c + sh * s, h: sw * s + sh * c };
}

export function aspectRatioFor(aspect: AspectLock, sw: number, sh: number): number | null {
  switch (aspect.kind) {
    case 'free':
      return null;
    case 'original':
      return sw / sh;
    case 'ratio':
      if (aspect.num === 0 || aspect.den === 0) return null;
      return aspect.num / aspect.den;
  }
}

export function pointInRotatedSource(
  p: Point,
  sw: number,
  sh: number,
  angleDeg: number
): boolean {
  const bbox = rotatedBbox(sw, sh, angleDeg);
  const a = degToRad(angleDeg);
  const c = Math.cos(a);
  const s = Math.sin(a);
  const cx = bbox.w / 2;
  const cy = bbox.h / 2;
  const dx = p.x - cx;
  const dy = p.y - cy;
  const ux = dx * c + dy * s;
  const uy = -dx * s + dy * c;
  const hw = sw / 2;
  const hh = sh / 2;
  return Math.abs(ux) <= hw + 1e-3 && Math.abs(uy) <= hh + 1e-3;
}

export function cropRectInsideRotatedSource(
  rect: CropRect,
  sw: number,
  sh: number,
  angleDeg: number
): boolean {
  const bbox = rotatedBbox(sw, sh, angleDeg);
  const x0 = rect.x * bbox.w;
  const y0 = rect.y * bbox.h;
  const x1 = (rect.x + rect.w) * bbox.w;
  const y1 = (rect.y + rect.h) * bbox.h;
  const corners: Point[] = [
    { x: x0, y: y0 },
    { x: x1, y: y0 },
    { x: x1, y: y1 },
    { x: x0, y: y1 }
  ];
  return corners.every((p) => pointInRotatedSource(p, sw, sh, angleDeg));
}

export function largestInscribedRect(
  sw: number,
  sh: number,
  angleDeg: number,
  aspect: number
): CropRect {
  const bbox = rotatedBbox(sw, sh, angleDeg);
  const target = Math.max(aspect, 1e-6);
  const bboxAspect = bbox.w / bbox.h;
  const baseW = bboxAspect >= target ? bbox.h * target : bbox.w;
  const baseH = bboxAspect >= target ? bbox.h : bbox.w / target;
  let lo = 0;
  let hi = 1;
  for (let i = 0; i < 40; i++) {
    const mid = (lo + hi) / 2;
    const wPx = baseW * mid;
    const hPx = baseH * mid;
    const nx = (bbox.w - wPx) / 2 / bbox.w;
    const ny = (bbox.h - hPx) / 2 / bbox.h;
    const rect: CropRect = { x: nx, y: ny, w: wPx / bbox.w, h: hPx / bbox.h };
    if (cropRectInsideRotatedSource(rect, sw, sh, angleDeg)) {
      lo = mid;
    } else {
      hi = mid;
    }
  }
  const wPx = baseW * lo;
  const hPx = baseH * lo;
  const nx = (bbox.w - wPx) / 2 / bbox.w;
  const ny = (bbox.h - hPx) / 2 / bbox.h;
  return {
    x: clamp01(nx),
    y: clamp01(ny),
    w: clamp01(wPx / bbox.w),
    h: clamp01(hPx / bbox.h)
  };
}

export function refitCropAtAspect(
  prev: CropRect,
  sw: number,
  sh: number,
  angleDeg: number,
  aspect: number
): CropRect {
  const bbox = rotatedBbox(sw, sh, angleDeg);
  const target = Math.max(aspect, 1e-6);
  const bboxAspect = bbox.w / bbox.h;
  const baseW = bboxAspect >= target ? bbox.h * target : bbox.w;
  const baseH = bboxAspect >= target ? bbox.h : bbox.w / target;
  const cxPx = (prev.x + prev.w / 2) * bbox.w;
  const cyPx = (prev.y + prev.h / 2) * bbox.h;
  let lo = 0;
  let hi = 1;
  let best: CropRect = { x: 0.5, y: 0.5, w: 0, h: 0 };
  for (let i = 0; i < 40; i++) {
    const t = (lo + hi) / 2;
    const wPx = baseW * t;
    const hPx = baseH * t;
    let x0 = cxPx - wPx / 2;
    let y0 = cyPx - hPx / 2;
    if (x0 < 0) x0 = 0;
    if (y0 < 0) y0 = 0;
    if (x0 + wPx > bbox.w) x0 = bbox.w - wPx;
    if (y0 + hPx > bbox.h) y0 = bbox.h - hPx;
    const rect: CropRect = {
      x: x0 / bbox.w,
      y: y0 / bbox.h,
      w: wPx / bbox.w,
      h: hPx / bbox.h
    };
    if (cropRectInsideRotatedSource(rect, sw, sh, angleDeg)) {
      best = rect;
      lo = t;
    } else {
      hi = t;
    }
  }
  return best;
}

export function constrainCropRect(
  candidate: CropRect,
  previous: CropRect | null,
  sw: number,
  sh: number,
  angleDeg: number
): CropRect {
  const clipped: CropRect = {
    x: clamp01(candidate.x),
    y: clamp01(candidate.y),
    w: clamp01(candidate.w),
    h: clamp01(candidate.h)
  };
  if (clipped.x + clipped.w > 1) clipped.w = 1 - clipped.x;
  if (clipped.y + clipped.h > 1) clipped.h = 1 - clipped.y;
  if (cropRectInsideRotatedSource(clipped, sw, sh, angleDeg)) return clipped;
  const base = previous ?? FULL_CROP;
  if (!cropRectInsideRotatedSource(base, sw, sh, angleDeg)) {
    return largestInscribedRect(sw, sh, angleDeg, clipped.w / Math.max(clipped.h, 1e-6));
  }
  let lo = 0;
  let hi = 1;
  let best = base;
  for (let i = 0; i < 24; i++) {
    const t = (lo + hi) / 2;
    const r: CropRect = {
      x: base.x + (clipped.x - base.x) * t,
      y: base.y + (clipped.y - base.y) * t,
      w: base.w + (clipped.w - base.w) * t,
      h: base.h + (clipped.h - base.h) * t
    };
    if (cropRectInsideRotatedSource(r, sw, sh, angleDeg)) {
      best = r;
      lo = t;
    } else {
      hi = t;
    }
  }
  return best;
}

function clamp01(v: number): number {
  if (v < 0) return 0;
  if (v > 1) return 1;
  return v;
}
