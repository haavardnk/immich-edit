import { getJson, sendJson } from './client';

export type MaskKind = 'subject' | 'people' | 'sky' | 'depth';

export interface MaskModel {
  id: string;
  name: string;
  kind: MaskKind;
  tier: 'recommended' | 'alternative' | 'low_memory';
  license: string;
  source: string;
  notes: string;
  size_bytes: number;
  input_edge: number;
  gpu_ms: number;
  gpu_mb: number;
  cpu_ms: number;
  cpu_mb: number;
  installed: boolean;
}

export interface MaskModelsResponse {
  runtime: string;
  enabled: boolean;
  models: MaskModel[];
  active: Partial<Record<MaskKind, string>>;
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

export function listMaskModels(): Promise<MaskModelsResponse> {
  return getJson<MaskModelsResponse>('/api/masks/models', undefined, { silent: true });
}

export function generateMask(
  assetId: string,
  kind: MaskKind,
  grow = 0,
  feather = 0
): Promise<GeneratedMask> {
  return sendJson<GeneratedMask>('POST', `/api/assets/${assetId}/masks/generate`, {
    kind,
    grow,
    feather
  });
}

export function rebakeMask(
  assetId: string,
  probRasterId: string,
  grow: number,
  feather: number
): Promise<RebakedMask> {
  return sendJson<RebakedMask>('POST', '/api/masks/rebake', {
    asset_id: assetId,
    prob_raster_id: probRasterId,
    grow,
    feather
  });
}

export function installMaskModel(id: string): Promise<void> {
  return sendJson<void>('POST', `/api/admin/models/${id}`, undefined);
}

export function removeMaskModel(id: string): Promise<void> {
  return sendJson<void>('DELETE', `/api/admin/models/${id}`, undefined);
}

export function selectMaskModel(kind: MaskKind, modelId: string): Promise<void> {
  return sendJson<void>('PUT', '/api/admin/masks/default', { kind, model_id: modelId });
}
