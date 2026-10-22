import { sendJson, url } from './client';

export type MaskKind = 'subject' | 'people' | 'sky' | 'depth' | 'semantic' | 'click';

export interface SemanticClass {
  id: string;
  name: string;
}

export interface GeneratedMask {
  raster_id: string;
  prob_raster_id: string;
  width: number;
  height: number;
  model_id: string;
  backend: string;
  elapsed_ms: number;
}

export interface RebakedMask {
  raster_id: string;
  width: number;
  height: number;
}

export function generateMask(
  assetId: string,
  kind: MaskKind,
  grow = 0,
  feather = 0,
  maskClass?: string
): Promise<GeneratedMask> {
  return sendJson<GeneratedMask>('POST', url`/api/assets/${assetId}/masks/generate`, {
    kind,
    grow,
    feather,
    ...(maskClass ? { class: maskClass } : {})
  });
}

export interface ClickPoint {
  x: number;
  y: number;
  positive: boolean;
}

export interface MaskBox {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

export function clickMask(
  assetId: string,
  points: ClickPoint[],
  grow = 0,
  feather = 0,
  baseRasterId?: string,
  subtract = false,
  bbox?: MaskBox
): Promise<GeneratedMask> {
  return sendJson<GeneratedMask>('POST', url`/api/assets/${assetId}/masks/click`, {
    points,
    grow,
    feather,
    base_raster_id: baseRasterId ?? null,
    subtract,
    bbox: bbox ?? null
  });
}

export interface MaskRange {
  min: number;
  max: number;
  softness: number;
}

export function rebakeMask(
  assetId: string,
  probRasterId: string,
  grow: number,
  feather: number,
  range?: MaskRange
): Promise<RebakedMask> {
  return sendJson<RebakedMask>('POST', '/api/masks/rebake', {
    asset_id: assetId,
    prob_raster_id: probRasterId,
    grow,
    feather,
    range: range ?? null
  });
}
