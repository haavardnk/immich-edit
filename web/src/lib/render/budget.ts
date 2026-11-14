import type { LayoutMode } from '#lib/device/layoutRules.ts';

export type RenderBudget = { liveEdge: number; maxEdge: number; dprCap: number };

export const DESKTOP_BUDGET: RenderBudget = {
  liveEdge: 1600,
  maxEdge: 4096,
  dprCap: Number.POSITIVE_INFINITY
};

export const TOUCH_BUDGET: RenderBudget = { liveEdge: 1024, maxEdge: 2560, dprCap: 2 };

export function budgetFor(mode: LayoutMode): RenderBudget {
  return mode === 'touch' ? TOUCH_BUDGET : DESKTOP_BUDGET;
}

export function cappedDpr(dpr: number, budget: RenderBudget): number {
  return Math.min(dpr || 1, budget.dprCap);
}
