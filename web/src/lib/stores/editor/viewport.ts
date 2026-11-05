import { ui } from '#lib/stores/ui.svelte.ts';
import { ZoomCycle } from '#lib/stores/zoomCycle.ts';
import type { Edits } from '#lib/types/edits.ts';
import type { PreviewMeta } from '#lib/types/preview.ts';
import { viewTransform } from '#lib/utils/canvasCoords.ts';
import { panByStep, type PanStep, type ViewportMetrics } from '#lib/utils/imageViewport.ts';
import type { PreviewSurface } from '#lib/utils/previewSurface.ts';

export interface ViewportCtx {
  assetId: string | null;
  edits: Edits;
  meta: PreviewMeta | null;
}

export class ViewportNav {
  private baseImage: PreviewSurface | null = null;
  private metrics: ViewportMetrics | null = null;
  private readonly cycle: ZoomCycle;

  constructor(ctx: ViewportCtx) {
    this.cycle = new ZoomCycle({
      shown: () => ctx.assetId,
      transform: () => viewTransform(ctx.edits, ctx.meta),
      image: () => this.baseImage,
      metrics: () => this.metrics,
      zoomLevel: () => ui.zoomLevel,
      view: () => ui.view,
      apply: (_id, view) => ui.setView(view)
    });
  }

  setBaseImage(element: PreviewSurface | null): void {
    this.baseImage = element;
  }

  setMetrics(metrics: ViewportMetrics): void {
    this.metrics = metrics;
  }

  cycleZoom(): void {
    this.cycle.next();
  }

  panBy(step: PanStep): void {
    if (!ui.zoomed || !this.metrics) return;
    ui.setView(panByStep(this.metrics, ui.view, step));
  }

  reset(): void {
    this.cycle.reset();
  }
}
