import type { Rect } from '$lib/utils/viewGeometry';

export const LIVE_EDGE = 1600;
export const MAX_EDGE = 4096;

export type ViewSnapshot = { frame: Rect; viewW: number; viewH: number; dpr: number };

export function baseEdge(snap: ViewSnapshot | null): number {
  if (!snap) return LIVE_EDGE;
  const long = Math.round(Math.max(snap.frame.width, snap.frame.height) * snap.dpr);
  return Math.max(LIVE_EDGE, Math.min(MAX_EDGE, long));
}

export function fitEdge(snap: ViewSnapshot | null): number {
  if (!snap || snap.frame.width <= 0 || snap.frame.height <= 0) return LIVE_EDGE;
  const fit = Math.min(1, snap.viewW / snap.frame.width, snap.viewH / snap.frame.height);
  const long = Math.max(snap.frame.width, snap.frame.height) * fit * snap.dpr;
  return Math.max(LIVE_EDGE, Math.min(MAX_EDGE, Math.round(long)));
}

export function dragEdge(snap: ViewSnapshot | null): number {
  const long = Math.round(Math.max(snap?.viewW ?? 0, snap?.viewH ?? 0));
  return long > 0 ? Math.min(LIVE_EDGE, long) : LIVE_EDGE;
}
