import { getJson, sendJson, url } from './client';
import type { MaskKind, SemanticClass } from './masks';

export type ModelKind = MaskKind | 'denoise';

export interface Model {
  id: string;
  name: string;
  kind: ModelKind;
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
  installing: boolean;
  progress_bytes: number;
  install_error: string | null;
}

export interface ModelsResponse {
  runtime: string;
  enabled: boolean;
  models: Model[];
  active: Partial<Record<ModelKind, string>>;
  semantic_classes: SemanticClass[];
}

export function listModels(): Promise<ModelsResponse> {
  return getJson<ModelsResponse>('/api/masks/models', undefined, { silent: true });
}

export function installModel(id: string): Promise<void> {
  return sendJson<void>('POST', url`/api/admin/models/${id}`, undefined, undefined, {
    silent: true
  });
}

export function cancelModelInstall(id: string): Promise<void> {
  return sendJson<void>('DELETE', url`/api/admin/models/${id}/install`, undefined);
}

export function removeModel(id: string): Promise<void> {
  return sendJson<void>('DELETE', url`/api/admin/models/${id}`, undefined);
}

export function selectDefaultModel(kind: ModelKind, modelId: string): Promise<void> {
  return sendJson<void>('PUT', '/api/admin/masks/default', { kind, model_id: modelId });
}
