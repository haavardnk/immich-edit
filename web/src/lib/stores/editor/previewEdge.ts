import { cappedDpr, DESKTOP_BUDGET, type RenderBudget } from '#lib/render/budget.ts';
import type { Rect } from '#lib/utils/viewGeometry.ts';

export type ViewSnapshot = { frame: Rect; viewW: number; viewH: number; dpr: number };

function clampEdge(edge: number, budget: RenderBudget): number {
  return Math.max(budget.liveEdge, Math.min(budget.maxEdge, edge));
}

export function baseEdge(snap: ViewSnapshot | null, budget = DESKTOP_BUDGET): number {
  if (!snap) return budget.liveEdge;
  const long = Math.max(snap.frame.width, snap.frame.height) * cappedDpr(snap.dpr, budget);
  return clampEdge(Math.round(long), budget);
}

export function fitEdge(snap: ViewSnapshot | null, budget = DESKTOP_BUDGET): number {
  if (!snap || snap.frame.width <= 0 || snap.frame.height <= 0) return budget.liveEdge;
  const fit = Math.min(1, snap.viewW / snap.frame.width, snap.viewH / snap.frame.height);
  const long = Math.max(snap.frame.width, snap.frame.height) * fit * cappedDpr(snap.dpr, budget);
  return clampEdge(Math.round(long), budget);
}

export function dragEdge(snap: ViewSnapshot | null, budget = DESKTOP_BUDGET): number {
  const long = Math.round(Math.max(snap?.viewW ?? 0, snap?.viewH ?? 0));
  return long > 0 ? Math.min(budget.liveEdge, long) : budget.liveEdge;
}

export function largestEdge(snap: ViewSnapshot | null, budget = DESKTOP_BUDGET): number {
  const long = Math.max(snap?.viewW ?? 0, snap?.viewH ?? 0);
  const view = Math.round(long * cappedDpr(snap?.dpr ?? 1, budget));
  return Math.max(fitEdge(snap, budget), view);
}

export function windowEdge(budget = DESKTOP_BUDGET): number {
  const long = Math.max(globalThis.innerWidth || 0, globalThis.innerHeight || 0);
  return Math.round(long * cappedDpr(globalThis.devicePixelRatio || 1, budget));
}
