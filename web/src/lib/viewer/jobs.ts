import type { ProofOptions } from '#lib/api/preview.ts';
import type { Edits } from '#lib/types/edits.ts';
import { previewColorSpace } from '#lib/utils/colorGamut.ts';
import type { Size } from '#lib/utils/geom.ts';
import { renderRequest, visibleRegion, type Rect, type Roi } from '#lib/utils/viewGeometry.ts';
import type { ViewerRender } from '#lib/viewer/protocol.ts';

export type ViewerSlot = 'base' | 'left' | 'tile';

export type ViewerJob = {
  slot: ViewerSlot;
  key: string;
  assetId: string;
  edits: Edits | null;
  edge: number;
  fullEdge: number;
  proof: ProofOptions;
  roi: Roi | null;
  revision: number;
  target: string;
  patient: boolean;
};

export type LandedJob = ViewerJob & { url: string; w: number; h: number };

export type JobInput = {
  render: ViewerRender;
  revision: number;
  pane: Size;
  dpr: number;
  wide: boolean;
  srcLong: number;
  frame: Rect | null;
  base: LandedJob | null;
  tile: LandedJob | null;
};

const EDGE_STEP = 256;
const MIN_EDGE = 1600;
const MAX_EDGE = 4096;
const DRAG_EDGE = 1600;
const TILE_MAX_EDGE = 65535;
const FULL_ROI: Roi = [0, 0, 1, 1];

export function viewerProof(render: ViewerRender, wide: boolean): ProofOptions {
  return {
    colorSpace: previewColorSpace(render.proofSpace, render.gamutWarn, wide),
    gamutWarn: render.gamutWarn,
    clipWarn: render.clipWarn
  };
}

export function baseEdge(pane: Size, dpr: number, srcLong: number): number {
  const wanted = Math.ceil((Math.max(pane.w, pane.h) * dpr) / EDGE_STEP) * EDGE_STEP;
  const edge = Math.min(MAX_EDGE, Math.max(MIN_EDGE, wanted));
  return Number.isFinite(srcLong) && srcLong > 0 ? Math.min(edge, Math.round(srcLong)) : edge;
}

export function leftTarget(render: ViewerRender, proof: ProofOptions): string | null {
  const left = render.compare?.left;
  if (!left) return null;
  return `${left.assetId}|${left.key}|${JSON.stringify(proof)}`;
}

function baseJob(input: JobInput, proof: ProofOptions, full: number): ViewerJob | null {
  const { render, revision, base } = input;
  const edge = render.dragging ? Math.min(DRAG_EDGE, full) : full;
  if (base && base.revision === revision && base.edge >= edge) return null;
  return {
    slot: 'base',
    key: `base|${revision}|${edge}`,
    assetId: render.assetId,
    edits: render.edits,
    edge,
    fullEdge: edge,
    proof,
    roi: null,
    revision,
    target: render.assetId,
    patient: render.dragging
  };
}

function leftJob(input: JobInput, proof: ProofOptions, edge: number): ViewerJob | null {
  const left = input.render.compare?.left;
  const target = leftTarget(input.render, proof);
  if (!left || !target) return null;
  return {
    slot: 'left',
    key: `left|${target}|${edge}`,
    assetId: left.assetId,
    edits: left.edits,
    edge,
    fullEdge: edge,
    proof,
    roi: null,
    revision: input.revision,
    target,
    patient: false
  };
}

function tileJob(input: JobInput, proof: ProofOptions): ViewerJob | null {
  const { render, revision, frame, base, tile, pane, dpr, srcLong } = input;
  if (!frame || !base || base.assetId !== render.assetId) return null;
  const visible = visibleRegion(frame, pane.w, pane.h);
  if (!visible) return null;
  const request = (haveRoi: Roi, haveFullEdge: number) =>
    renderRequest({
      frame,
      visible,
      dpr,
      srcLong,
      serverMaxEdge: TILE_MAX_EDGE,
      haveRoi,
      haveFullEdge
    });
  if (tile?.roi && tile.revision === revision && !request(tile.roi, tile.fullEdge)) return null;
  const wanted = request(FULL_ROI, Math.max(base.w, base.h));
  if (!wanted) return null;
  return {
    slot: 'tile',
    key: `tile|${revision}|${wanted.roi.join(',')}|${wanted.maxEdge}`,
    assetId: render.assetId,
    edits: render.edits,
    edge: wanted.maxEdge,
    fullEdge: wanted.fullEdge,
    proof,
    roi: wanted.roi,
    revision,
    target: render.assetId,
    patient: false
  };
}

export function viewerJobs(input: JobInput): ViewerJob[] {
  const { render, pane, dpr, wide, srcLong } = input;
  if (pane.w <= 0 || pane.h <= 0) return [];
  const proof = viewerProof(render, wide);
  const edge = baseEdge(pane, dpr, srcLong);
  const second = render.compare
    ? leftJob(input, proof, edge)
    : render.dragging
      ? null
      : tileJob(input, proof);
  return [baseJob(input, proof, edge), second].filter((job) => job !== null);
}

export function landedFullEdge(job: ViewerJob, w: number, h: number): number {
  const delivered = Math.max(w, h);
  return job.roi && job.edge > 0 ? Math.round((delivered * job.fullEdge) / job.edge) : delivered;
}
