import { MASK_FEATHER, POLYGON_FEATHER } from '#lib/editor/params/maskComponents.ts';
import type { ToolRange } from '#lib/editor/params/types.ts';
import type { MaskComponentKind, Vec2f } from '#lib/types/edits.ts';

export type ShapeWheelAxis = 'size' | 'feather';

const SIZE_PER_NOTCH = 1.1;
const FEATHER_NOTCHES = 20;
const MIN_EXTENT = 0.005;
const MAX_EXTENT = 4;

function clampTo(value: number, range: Pick<ToolRange, 'min' | 'max'>): number {
  return Math.min(range.max, Math.max(range.min, value));
}

function scaleAbout(point: Vec2f, centre: Vec2f, factor: number): Vec2f {
  return {
    x: centre.x + (point.x - centre.x) * factor,
    y: centre.y + (point.y - centre.y) * factor
  };
}

function boundsOf(points: readonly Vec2f[]): { min: Vec2f; max: Vec2f } {
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  return {
    min: { x: Math.min(...xs), y: Math.min(...ys) },
    max: { x: Math.max(...xs), y: Math.max(...ys) }
  };
}

function extentOf(kind: MaskComponentKind, aspect: number): number | null {
  if (kind.kind === 'radial') return Math.max(kind.radius_xy.x * aspect, kind.radius_xy.y);
  if (kind.kind === 'linear') {
    return Math.hypot((kind.p1.x - kind.p0.x) * aspect, kind.p1.y - kind.p0.y);
  }
  if (kind.kind === 'polygon' && kind.points.length > 0) {
    const { min, max } = boundsOf(kind.points);
    return Math.max((max.x - min.x) * aspect, max.y - min.y);
  }
  return null;
}

function boundedFactor(extent: number, notches: number): number {
  const target = clampTo(extent * SIZE_PER_NOTCH ** notches, { min: MIN_EXTENT, max: MAX_EXTENT });
  return extent > 0 ? target / extent : 1;
}

export function resizedKind(
  kind: MaskComponentKind,
  notches: number,
  aspect: number
): MaskComponentKind | null {
  const extent = extentOf(kind, aspect);
  if (extent === null) return null;
  const factor = boundedFactor(extent, notches);
  if (kind.kind === 'radial') {
    return { ...kind, radius_xy: { x: kind.radius_xy.x * factor, y: kind.radius_xy.y * factor } };
  }
  if (kind.kind === 'linear') {
    const mid = { x: (kind.p0.x + kind.p1.x) / 2, y: (kind.p0.y + kind.p1.y) / 2 };
    return { ...kind, p0: scaleAbout(kind.p0, mid, factor), p1: scaleAbout(kind.p1, mid, factor) };
  }
  if (kind.kind === 'polygon') {
    const { min, max } = boundsOf(kind.points);
    const centre = { x: (min.x + max.x) / 2, y: (min.y + max.y) / 2 };
    return { ...kind, points: kind.points.map((p) => scaleAbout(p, centre, factor)) };
  }
  return null;
}

export function featherRange(kind: MaskComponentKind): ToolRange | null {
  if (kind.kind === 'radial' || kind.kind === 'linear') return MASK_FEATHER;
  if (kind.kind === 'polygon') return POLYGON_FEATHER;
  return null;
}

export function featheredKind(kind: MaskComponentKind, notches: number): MaskComponentKind | null {
  const range = featherRange(kind);
  if (!range || !('feather' in kind)) return null;
  const step = (range.max - range.min) / FEATHER_NOTCHES;
  return { ...kind, feather: clampTo(kind.feather + notches * step, range) };
}

export function wheeledKind(
  kind: MaskComponentKind,
  axis: ShapeWheelAxis,
  notches: number,
  aspect: number
): MaskComponentKind | null {
  return axis === 'feather' ? featheredKind(kind, notches) : resizedKind(kind, notches, aspect);
}

export function shapeExtentLabel(
  kind: MaskComponentKind,
  source: { w: number; h: number } | null
): string {
  const aspect = source ? source.w / Math.max(source.h, 1) : 1;
  const extent = extentOf(kind, aspect) ?? 0;
  return source ? `${Math.round(extent * source.h)} px` : extent.toFixed(3);
}

export function shapeFeatherLabel(kind: MaskComponentKind): string {
  const range = featherRange(kind);
  return range && 'feather' in kind ? range.format(kind.feather) : '';
}
