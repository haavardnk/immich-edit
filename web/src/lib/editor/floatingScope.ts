import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
import type { Point } from '#lib/gestures/point.ts';
import type { Size } from '#lib/utils/geom.ts';

export type Corner = 'tl' | 'tr' | 'bl' | 'br';

export const CORNERS: readonly Corner[] = ['tl', 'tr', 'bl', 'br'];

export const DRAG_SLOP = 4;

const LIGHT_SETTINGS = new Set(
  TOUCH_GROUPS.filter((group) => group.id === 'light').flatMap((group) =>
    group.views.flatMap((view) => view.params.map((param) => param.id))
  )
);

export function nearestCorner(centre: Point, box: Size): Corner {
  const top = centre.y < box.h / 2;
  const left = centre.x < box.w / 2;
  if (top) return left ? 'tl' : 'tr';
  return left ? 'bl' : 'br';
}

export function yieldsTo(dragging: string | null): boolean {
  return dragging !== null && !LIGHT_SETTINGS.has(dragging);
}
