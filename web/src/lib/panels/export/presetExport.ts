import { runBulkJob } from '#lib/api/bulkJob.ts';
import type { ExportOptions, ImmichExportOptions, OnConflict } from '#lib/api/export.ts';
import type { ExportPreset } from '#lib/api/exportPresets.ts';
import {
  createImmichPresetsJob,
  createZipPresetsJob,
  type Job,
  type PresetVariant
} from '#lib/api/jobs.ts';
import {
  DEFAULT_FILENAME_TEMPLATE,
  PRESET_FILENAME_TEMPLATE
} from '#lib/editor/filenameTemplate.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { uploadOptions, type ExportScope } from './scope';
import {
  baseOptions,
  pruneLibraryIds,
  restoreExportForm,
  type Destination,
  type ExportFormValues
} from './settings';

export function presetForm(preset: ExportPreset): ExportFormValues {
  const form = restoreExportForm(preset.form);
  pruneLibraryIds(form);
  const template = form.filenameTemplate.trim();
  if (!template || template === DEFAULT_FILENAME_TEMPLATE) {
    form.filenameTemplate = PRESET_FILENAME_TEMPLATE;
  }
  return form;
}

export function presetVariants(
  presets: readonly ExportPreset[],
  scope: ExportScope,
  onConflict: OnConflict = 'keep_both'
): { zip: PresetVariant<ExportOptions>[]; immich: PresetVariant<ImmichExportOptions>[] } {
  return {
    zip: presets
      .filter((preset) => preset.destination === 'download')
      .map((preset) => ({ preset: preset.name, options: baseOptions(presetForm(preset)) })),
    immich: presets
      .filter((preset) => preset.destination === 'immich')
      .map((preset) => ({
        preset: preset.name,
        options: uploadOptions(presetForm(preset), scope, false, onConflict)
      }))
  };
}

export interface StartedExport {
  destination: Destination;
  job: Job;
}

export async function submitPresetExport(
  assetIds: string[],
  presets: readonly ExportPreset[],
  scope: ExportScope,
  onConflict: OnConflict = 'keep_both'
): Promise<StartedExport[]> {
  const { zip, immich } = presetVariants(presets, scope, onConflict);
  const started: StartedExport[] = [];
  if (zip.length > 0) {
    const job = await runBulkJob(
      (ids) => createZipPresetsJob(ids, zip),
      'Failed to queue export',
      assetIds
    );
    if (job) {
      jobs.downloadWhenDone(job.id);
      started.push({ destination: 'download', job });
    }
  }
  if (immich.length > 0) {
    const job = await runBulkJob(
      (ids) => createImmichPresetsJob(ids, immich),
      'Failed to queue upload',
      assetIds
    );
    if (job) started.push({ destination: 'immich', job });
  }
  return started;
}
