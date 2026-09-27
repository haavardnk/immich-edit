import { ui } from '$lib/stores/ui.svelte';
import { cachedFaceData, loadFaceData } from '$lib/stores/zoomTargets';
import type { Edits } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';
import { viewTransform } from '$lib/utils/canvasCoords';
import { nudgePan, type PanStep } from '$lib/utils/imageViewport';
import type { PreviewSurface } from '$lib/utils/previewSurface';
import {
  faceTargets,
  nextTargetIndex,
  panForTarget,
  sharpestPoint,
  type ZoomTarget
} from '$lib/utils/zoomTarget';
import type { ViewSnapshot } from './previewEdge';

export interface ViewportCtx {
  assetId: string | null;
  edits: Edits;
  meta: PreviewMeta | null;
}

export class ViewportNav {
  private readonly ctx: ViewportCtx;
  private readonly snapshot: () => ViewSnapshot | null;
  private baseImage: PreviewSurface | null = null;
  private targetIndex: number | null = null;
  private targetView: string | null = null;

  constructor(ctx: ViewportCtx, snapshot: () => ViewSnapshot | null) {
    this.ctx = ctx;
    this.snapshot = snapshot;
  }

  setBaseImage(element: PreviewSurface | null): void {
    this.baseImage = element;
  }

  zoomCycle(): void {
    const id = this.ctx.assetId;
    if (!id) return;
    if (cachedFaceData(id) === null) {
      void loadFaceData(id).then(() => this.stepTarget(id));
      return;
    }
    this.stepTarget(id);
  }

  panBy(step: PanStep): void {
    const snap = this.snapshot();
    if (!ui.zoomed || !snap) return;
    const pan = nudgePan(
      { panX: ui.panX, panY: ui.panY },
      step,
      snap.frame,
      snap.viewW,
      snap.viewH
    );
    ui.panX = pan.panX;
    ui.panY = pan.panY;
  }

  reset(): void {
    this.targetIndex = null;
    this.targetView = null;
  }

  private targets(id: string): ZoomTarget[] {
    const data = cachedFaceData(id);
    const faces = data ? faceTargets(data.faces, viewTransform(this.ctx.edits, this.ctx.meta)) : [];
    if (faces.length > 0) return faces;
    const sharp = this.baseImage ? sharpestPoint(this.baseImage) : null;
    return [sharp ?? { u: 0.5, v: 0.5 }];
  }

  private stepTarget(id: string): void {
    if (this.ctx.assetId !== id) return;
    const targets = this.targets(id);
    const from = this.viewKey() === this.targetView ? this.targetIndex : null;
    const next = nextTargetIndex(from, targets.length);
    this.targetIndex = next;
    const target = next === null ? null : targets[next];
    const snap = this.snapshot();
    if (!target) {
      ui.zoomFit();
    } else if (!snap || snap.frame.width <= 0 || ui.zoom <= 0) {
      ui.setZoom(ui.zoomLevel);
    } else {
      const zoom = ui.zoomLevel;
      const pan = panForTarget(target, snap.frame, snap.viewW, snap.viewH, zoom / ui.zoom);
      ui.setView(zoom, pan.panX, pan.panY);
    }
    this.targetView = this.viewKey();
  }

  private viewKey(): string {
    return `${ui.zoom}:${Math.round(ui.panX)}:${Math.round(ui.panY)}`;
  }
}
