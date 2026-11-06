import type { Point } from '#lib/gestures/point.ts';
import type { Size } from '#lib/utils/geom.ts';
import { fitScale, nativeScale } from '#lib/utils/viewGeometry.ts';
import { clampZoom } from '#lib/utils/zoomLevel.ts';

export type View = { zoom: number | null; cx: number; cy: number };

export type ZoomedView = View & { zoom: number };

export type ViewChange = 'pan' | 'zoom';

export type PanStep = [number, number];

export type ViewportMetrics = {
  box: Size;
  natural: Size;
  fit: number;
  unit: number;
  fitZoom: number;
};

export const FIT_VIEW: View = { zoom: null, cx: 0.5, cy: 0.5 };

const WHEEL_STEP = 1.1;

const PAN_STEP = 0.1;
const PAN_STEPS: Record<string, PanStep> = {
  ArrowLeft: [-PAN_STEP, 0],
  ArrowRight: [PAN_STEP, 0],
  ArrowUp: [0, -PAN_STEP],
  ArrowDown: [0, PAN_STEP]
};

export function panStep(key: string): PanStep | null {
  return PAN_STEPS[key] ?? null;
}

export function viewportMetrics(
  box: Size,
  natural: Size,
  srcLong: number,
  dpr: number
): ViewportMetrics {
  const fit = fitScale(box.w, box.h, natural.w, natural.h);
  const unit = nativeScale(Math.max(natural.w, natural.h), srcLong, dpr) || fit;
  return { box, natural, fit, unit, fitZoom: unit > 0 ? (100 * fit) / unit : 100 };
}

export function scaleOf(metrics: ViewportMetrics, zoom: number | null): number {
  return zoom === null ? metrics.fit : (zoom / 100) * metrics.unit;
}

export function frameSize(metrics: ViewportMetrics, zoom: number | null): Size {
  const scale = scaleOf(metrics, zoom);
  return { w: metrics.natural.w * scale, h: metrics.natural.h * scale };
}

export function zoomRatio(metrics: ViewportMetrics, zoom: number | null): number {
  return metrics.fit > 0 ? scaleOf(metrics, zoom) / metrics.fit : 1;
}

export function isZoomed(metrics: ViewportMetrics, view: View): boolean {
  return view.zoom !== null && view.zoom > metrics.fitZoom;
}

function within(value: number, limit: number): number {
  return Math.min(0.5 + limit, Math.max(0.5 - limit, value));
}

export function clampView(metrics: ViewportMetrics, view: View): View {
  const frame = frameSize(metrics, view.zoom);
  const { box } = metrics;
  if (box.w <= 0 || box.h <= 0 || frame.w <= 0 || frame.h <= 0) return view;
  const limitX = Math.max(0, (frame.w - box.w) / 2) / frame.w;
  const limitY = Math.max(0, (frame.h - box.h) / 2) / frame.h;
  return { zoom: view.zoom, cx: within(view.cx, limitX), cy: within(view.cy, limitY) };
}

export function withZoom(view: View, zoom: number, fitZoom: number): ZoomedView {
  const next = clampZoom(zoom, fitZoom);
  if (next <= fitZoom) return { zoom: next, cx: 0.5, cy: 0.5 };
  return { zoom: next, cx: view.cx, cy: view.cy };
}

export function zoomAround(
  metrics: ViewportMetrics,
  view: View,
  nextZoom: number,
  anchor: Point
): View {
  const from = clampView(metrics, view);
  const target = withZoom(from, nextZoom, metrics.fitZoom);
  if (target.zoom <= metrics.fitZoom) return target;
  const { zoom } = target;
  const before = frameSize(metrics, from.zoom);
  const after = frameSize(metrics, zoom);
  if (before.w <= 0 || before.h <= 0 || after.w <= 0 || after.h <= 0) {
    return { zoom, cx: from.cx, cy: from.cy };
  }
  return clampView(metrics, {
    zoom,
    cx: from.cx + anchor.x / before.w - anchor.x / after.w,
    cy: from.cy + anchor.y / before.h - anchor.y / after.h
  });
}

export function wheelZoom(zoom: number, deltaY: number): number {
  return zoom * (deltaY > 0 ? 1 / WHEEL_STEP : WHEEL_STEP);
}

export function pinchView(
  metrics: ViewportMetrics,
  start: View,
  scale: number,
  anchor: Point,
  pan: Point
): View {
  const zoom = (start.zoom ?? metrics.fitZoom) * scale;
  return panByPixels(metrics, zoomAround(metrics, start, zoom, anchor), pan.x, pan.y);
}

export function panByPixels(metrics: ViewportMetrics, view: View, dx: number, dy: number): View {
  const from = clampView(metrics, view);
  const frame = frameSize(metrics, from.zoom);
  if (frame.w <= 0 || frame.h <= 0) return from;
  return clampView(metrics, {
    zoom: from.zoom,
    cx: from.cx - dx / frame.w,
    cy: from.cy - dy / frame.h
  });
}

export function panByStep(metrics: ViewportMetrics, view: View, step: PanStep): View {
  return panByPixels(metrics, view, -step[0] * metrics.box.w, -step[1] * metrics.box.h);
}

export function frameOffset(metrics: ViewportMetrics, view: View): Point {
  const frame = frameSize(metrics, view.zoom);
  return { x: (0.5 - view.cx) * frame.w, y: (0.5 - view.cy) * frame.h };
}

export function viewportTransform(metrics: ViewportMetrics, view: View): string {
  const ratio = zoomRatio(metrics, view.zoom);
  const offset = frameOffset(metrics, view);
  if (ratio === 1 && offset.x === 0 && offset.y === 0) return '';
  return `transform: scale(${ratio}) translate(${offset.x / ratio}px, ${offset.y / ratio}px); transform-origin: center;`;
}
