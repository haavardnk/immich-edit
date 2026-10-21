import { getJson, sendJson, url } from './client';
import type { EditManifest } from '$lib/types/edits';

const PRESET_THUMB_EDGE = 160;

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
  return sendJson('PUT', url`/api/presets/${id}`, input);
}

export function deletePreset(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/presets/${id}`, undefined);
}

export function presetThumbUrl(assetId: string, preset: Preset, editHash: string): string {
  return url`/api/assets/${assetId}/presets/${preset.id}/thumb?max=${PRESET_THUMB_EDGE}&h=${editHash}&p=${preset.updated_at}`;
}
