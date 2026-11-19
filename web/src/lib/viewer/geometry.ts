import type { Size } from '#lib/utils/geom.ts';
import {
  clampView,
  frameOffset,
  scaleOf,
  viewportMetrics,
  type View
} from '#lib/utils/imageViewport.ts';
import { frameBox, type Rect } from '#lib/utils/viewGeometry.ts';

export type ViewerFrame = { frame: Rect; zoom: number };

export function viewerFrame(
  pane: Size,
  natural: Size,
  view: View,
  srcLong: number,
  dpr: number
): ViewerFrame | null {
  if (pane.w <= 0 || pane.h <= 0 || natural.w <= 0 || natural.h <= 0) return null;
  const metrics = viewportMetrics(pane, natural, srcLong, dpr);
  const clamped = clampView(metrics, view);
  const offset = frameOffset(metrics, clamped);
  const scale = scaleOf(metrics, clamped.zoom);
  const frame = frameBox(pane.w, pane.h, natural.w, natural.h, scale, offset.x, offset.y, dpr);
  if (!frame) return null;
  return { frame, zoom: Math.round(clamped.zoom ?? metrics.fitZoom) };
}
