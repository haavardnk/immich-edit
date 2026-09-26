import { getJson, sendJson, url } from './client';

export interface ExportPreset {
  id: string;
  name: string;
  form: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface ExportPresetInput {
  name: string;
  form: Record<string, unknown>;
}

export function listExportPresets(): Promise<ExportPreset[]> {
  return getJson('/api/export-presets');
}

export function createExportPreset(input: ExportPresetInput): Promise<ExportPreset> {
  return sendJson('POST', '/api/export-presets', input);
}

export function updateExportPreset(id: string, input: ExportPresetInput): Promise<ExportPreset> {
  return sendJson('PUT', url`/api/export-presets/${id}`, input);
}

export function deleteExportPreset(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/export-presets/${id}`, undefined);
}
