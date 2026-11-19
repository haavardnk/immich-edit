import type { ExportPreset } from '#lib/api/exportPresets.ts';
import { albumNames, restoreExportForm, type ExportFormValues } from './settings';
import { presetSummary } from './summary';

export interface PresetLine {
  form: ExportFormValues;
  immich: boolean;
  summary: string;
}

export function presetLine(preset: ExportPreset): PresetLine {
  const form = restoreExportForm(preset.form);
  const immich = preset.destination === 'immich';
  return {
    form,
    immich,
    summary: presetSummary(form, immich ? { albums: albumNames(form.albumIds) } : null)
  };
}
