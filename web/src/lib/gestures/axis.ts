import type { Point } from '#lib/gestures/point.ts';

export type Axis = 'x' | 'y';

export function dominantAxis(offset: Point): Axis {
  return Math.abs(offset.x) >= Math.abs(offset.y) ? 'x' : 'y';
}

export function along(axis: Axis, point: Point): number {
  return axis === 'x' ? point.x : point.y;
}

export function across(axis: Axis, point: Point): number {
  return axis === 'x' ? point.y : point.x;
}

export function lockedTo(axis: Axis | null, point: Point): Point {
  if (axis === 'x') return { x: point.x, y: 0 };
  if (axis === 'y') return { x: 0, y: point.y };
  return point;
}
