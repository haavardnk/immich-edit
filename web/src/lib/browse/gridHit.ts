import type { AssetGridBox, AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import { assetIndexAt } from '#lib/browse/gridZoom.ts';
import type { Point } from '#lib/gestures/point.ts';

export type GridRect = { left: number; top: number; right: number; bottom: number };

export function rectBetween(a: Point, b: Point): GridRect {
  return {
    left: Math.min(a.x, b.x),
    top: Math.min(a.y, b.y),
    right: Math.max(a.x, b.x),
    bottom: Math.max(a.y, b.y)
  };
}

function contains(box: AssetGridBox, point: Point): boolean {
  return (
    point.x >= box.left &&
    point.x <= box.left + box.width &&
    point.y >= box.top &&
    point.y <= box.top + box.height
  );
}

function overlaps(box: AssetGridBox, rect: GridRect): boolean {
  return (
    box.left <= rect.right &&
    box.left + box.width >= rect.left &&
    box.top <= rect.bottom &&
    box.top + box.height >= rect.top
  );
}

export function tileIndexAt(layout: AssetGridLayout, point: Point): number {
  const index = assetIndexAt(layout, point);
  const box = layout.boxes[index];
  return box && contains(box, point) ? index : -1;
}

export function marqueeHits(layout: AssetGridLayout, rect: GridRect): number[] {
  const first = layout.boxes[assetIndexAt(layout, { x: 0, y: rect.top })];
  if (!first) return [];
  const hits: number[] = [];
  for (let rowIndex = first.row; rowIndex < layout.rows.length; rowIndex += 1) {
    const row = layout.rows[rowIndex];
    if (!row || row.top > rect.bottom) break;
    for (let index = row.startIndex; index < row.endIndex; index += 1) {
      const box = layout.boxes[index];
      if (box && overlaps(box, rect)) hits.push(index);
    }
  }
  return hits;
}
