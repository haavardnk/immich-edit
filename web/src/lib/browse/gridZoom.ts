import type { AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import type { Point } from '#lib/gestures/point.ts';

export const ZOOM_STEP_RATIO = 1.25;
export const WHEEL_IDLE_MS = 250;
export const WHEEL_NOTCH_MS = 150;

const STEP = Math.log(ZOOM_STEP_RATIO);
const WHEEL_PX_PER_E = 100;
const LINE_PX = 16;
const PAGE_PX = 800;

export type GridAnchor = { index: number; ratio: number; offset: number };

export type TileAnchor = GridAnchor & { id: string };

export type AnchorRequest = {
  point: Point | null;
  activeIndex: number;
  viewTop: number;
  viewHeight: number;
};

export type WheelStepper = { push: (amount: number, time: number, pinch: boolean) => number };

export type ScaleStepper = { start: () => void; to: (scale: number) => number };

export function wheelZoomAmount(deltaY: number, deltaMode: number): number {
  const unit = deltaMode === 1 ? LINE_PX : deltaMode === 2 ? PAGE_PX : 1;
  return -(deltaY * unit) / WHEEL_PX_PER_E;
}

export function wheelStepper(): WheelStepper {
  let total = 0;
  let last = Number.NEGATIVE_INFINITY;
  let stepped = Number.NEGATIVE_INFINITY;
  return {
    push(amount, time, pinch) {
      if (amount === 0) return 0;
      if (time - last > WHEEL_IDLE_MS || Math.sign(amount) !== Math.sign(total)) total = 0;
      last = time;
      if (!pinch && time - stepped < WHEEL_NOTCH_MS) {
        total = 0;
        return 0;
      }
      total += amount;
      if (Math.abs(total) < STEP) return 0;
      const step = Math.sign(total);
      total = 0;
      stepped = time;
      return step;
    }
  };
}

export function scaleStepper(): ScaleStepper {
  let base = 0;
  return {
    start() {
      base = 0;
    },
    to(scale) {
      if (!(scale > 0)) return 0;
      const steps = Math.trunc((Math.log(scale) - base) / STEP);
      base += steps * STEP;
      return steps;
    }
  };
}

function rowIndexAt(layout: AssetGridLayout, y: number): number {
  let low = 0;
  let high = layout.rows.length - 1;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    const row = layout.rows[middle];
    if (row && row.top + row.height < y) low = middle + 1;
    else high = middle;
  }
  return low;
}

export function assetIndexAt(layout: AssetGridLayout, point: Point): number {
  const row = layout.rows[rowIndexAt(layout, point.y)];
  if (!row) return -1;
  for (let index = row.startIndex; index < row.endIndex; index += 1) {
    const box = layout.boxes[index];
    if (box && point.x <= box.left + box.width) return index;
  }
  return row.endIndex - 1;
}

export function gridAnchor(
  layout: AssetGridLayout,
  index: number,
  y: number,
  viewTop: number
): GridAnchor | null {
  const box = layout.boxes[index];
  if (!box) return null;
  const ratio = box.height > 0 ? Math.min(1, Math.max(0, (y - box.top) / box.height)) : 0;
  return { index, ratio, offset: y - viewTop };
}

function onScreen(layout: AssetGridLayout, index: number, top: number, bottom: number): boolean {
  const box = layout.boxes[index];
  return box !== undefined && box.top + box.height > top && box.top < bottom;
}

export function zoomAnchor(layout: AssetGridLayout, request: AnchorRequest): GridAnchor | null {
  const { point, activeIndex, viewTop, viewHeight } = request;
  if (point) return gridAnchor(layout, assetIndexAt(layout, point), point.y, viewTop);
  const active = layout.boxes[activeIndex];
  if (active && onScreen(layout, activeIndex, viewTop, viewTop + viewHeight)) {
    return gridAnchor(layout, activeIndex, active.top + active.height / 2, viewTop);
  }
  const top = Math.max(0, viewTop);
  return gridAnchor(layout, assetIndexAt(layout, { x: 0, y: top }), top, viewTop);
}

export function anchoredViewTop(layout: AssetGridLayout, anchor: GridAnchor): number | null {
  const box = layout.boxes[anchor.index];
  if (!box) return null;
  return box.top + anchor.ratio * box.height - anchor.offset;
}

export function prependedAnchor(
  ids: readonly string[],
  previousFirst: string | null,
  anchor: TileAnchor | null
): GridAnchor | null {
  if (!anchor || previousFirst === null || ids[0] === previousFirst) return null;
  if (ids.indexOf(previousFirst) <= 0) return null;
  const index = ids.indexOf(anchor.id);
  return index < 0 ? null : { index, ratio: anchor.ratio, offset: anchor.offset };
}
