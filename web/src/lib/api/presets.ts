import { getJson, sendJson } from './client';
import type { EditManifest } from '$lib/types/edits';

export interface Preset {
  id: string;
  name: string;
  group_name: string | null;
  manifest: EditManifest;
  created_at: string;
  updated_at: string;
}

export interface PresetInput {
  name: string;
  group_name: string | null;
  manifest: EditManifest;
}

export function listPresets(): Promise<Preset[]> {
  return getJson('/api/presets');
}

export function createPreset(input: PresetInput): Promise<Preset> {
  return sendJson('POST', '/api/presets', input);
}

export function updatePreset(id: string, input: PresetInput): Promise<Preset> {
  return sendJson('PUT', `/api/presets/${id}`, input);
}

export function deletePreset(id: string): Promise<void> {
  return sendJson('DELETE', `/api/presets/${id}`, undefined);
}
