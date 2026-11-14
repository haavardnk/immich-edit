import type { Point } from '#lib/gestures/point.ts';
import { clampView, frameSize, type View, type ViewportMetrics } from '#lib/utils/imageViewport.ts';
import type { Roi } from '#lib/utils/viewGeometry.ts';

function unit(value: number): number {
  return Math.min(1, Math.max(0, value));
}

export function visibleBox(metrics: ViewportMetrics, view: View): Roi {
  const clamped = clampView(metrics, view);
  const frame = frameSize(metrics, clamped.zoom);
  if (frame.w <= 0 || frame.h <= 0) return [0, 0, 1, 1];
  const w = unit(metrics.box.w / frame.w);
  const h = unit(metrics.box.h / frame.h);
  return [unit(clamped.cx - w / 2), unit(clamped.cy - h / 2), w, h];
}

export function grabOffset(box: Roi, point: Point): Point {
  const [x, y, w, h] = box;
  const inside = point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h;
  return inside ? { x: point.x - (x + w / 2), y: point.y - (y + h / 2) } : { x: 0, y: 0 };
}

export function panTo(metrics: ViewportMetrics, view: View, point: Point, grab: Point): View {
  return clampView(metrics, { zoom: view.zoom, cx: point.x - grab.x, cy: point.y - grab.y });
}
