import { cachedFaceData, loadFaceData, type FaceData } from '#lib/stores/zoomTargets.ts';
import type { ViewTransform } from '#lib/utils/canvasCoords.ts';
import { clampView, FIT_VIEW, type View, type ViewportMetrics } from '#lib/utils/imageViewport.ts';
import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
import {
  faceTargets,
  nextTargetIndex,
  sharpestPoint,
  type ZoomTarget
} from '#lib/utils/zoomTarget.ts';

export type ZoomCycleSpec = {
  shown: () => string | null;
  transform: (data: FaceData) => ViewTransform | null;
  image: (id: string) => PreviewSurface | null;
  metrics: (id: string) => ViewportMetrics | null;
  zoomLevel: () => number;
  view: (id: string) => View;
  apply: (id: string, view: View) => void;
};

const CENTRE: ZoomTarget = { u: 0.5, v: 0.5 };

function targetView(target: ZoomTarget, zoom: number): View {
  return { zoom, cx: target.u, cy: target.v };
}

function viewKey(view: View): string {
  return `${view.zoom}:${view.cx}:${view.cy}`;
}

export class ZoomCycle {
  private readonly spec: ZoomCycleSpec;
  private index: number | null = null;
  private stepped: string | null = null;

  constructor(spec: ZoomCycleSpec) {
    this.spec = spec;
  }

  next(): void {
    const id = this.spec.shown();
    if (!id) return;
    if (cachedFaceData(id) === null) {
      void loadFaceData(id).then(() => this.advance(id));
      return;
    }
    this.advance(id);
  }

  reset(): void {
    this.index = null;
    this.stepped = null;
  }

  private targets(id: string): ZoomTarget[] {
    const data = cachedFaceData(id);
    const transform = data ? this.spec.transform(data) : null;
    const faces = data && transform ? faceTargets(data.faces, transform) : [];
    if (faces.length > 0) return faces;
    const image = this.spec.image(id);
    return [(image ? sharpestPoint(image) : null) ?? CENTRE];
  }

  private advance(id: string): void {
    if (this.spec.shown() !== id) return;
    const metrics = this.spec.metrics(id);
    const zoom = this.spec.zoomLevel();
    if (metrics && zoom <= metrics.fitZoom) {
      this.index = null;
      this.show(id, FIT_VIEW);
      return;
    }
    const targets = this.targets(id);
    const from = viewKey(this.spec.view(id)) === this.stepped ? this.index : null;
    this.index = nextTargetIndex(from, targets.length);
    const target = this.index === null ? undefined : targets[this.index];
    const view = target ? targetView(target, zoom) : FIT_VIEW;
    this.show(id, metrics ? clampView(metrics, view) : view);
  }

  private show(id: string, view: View): void {
    this.spec.apply(id, view);
    this.stepped = viewKey(this.spec.view(id));
  }
}
