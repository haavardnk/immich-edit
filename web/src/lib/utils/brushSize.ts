import type { PreviewMeta } from '$lib/types/preview';

export function formatBrushSize(size: number, meta: PreviewMeta | null): string {
  const shortEdge = meta ? Math.min(meta.source_w, meta.source_h) : 0;
  return shortEdge > 1 ? `${Math.round(size * shortEdge)} px` : size.toFixed(3);
}
