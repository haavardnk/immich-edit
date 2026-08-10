const EDGE_STEPS = [1024, 1536, 2048, 2560, 3072, 4096];
const MAX_DPR = 2;

export function hiresEdge(viewportPx: number, dpr: number, zoom: number): number {
  const scale = Math.min(MAX_DPR, Math.max(1, dpr || 1));
  const needed = Math.ceil(viewportPx * scale * Math.max(1, zoom / 100));
  return EDGE_STEPS.find((step) => step >= needed) ?? EDGE_STEPS[EDGE_STEPS.length - 1];
}
