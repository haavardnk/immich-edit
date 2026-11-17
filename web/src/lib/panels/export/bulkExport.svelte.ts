import { runBulkJob } from '#lib/api/bulkJob.ts';
import { getEdits } from '#lib/api/edits.ts';
import {
  downloadName,
  downloadStoredExport,
  uploadToImmich,
  type ImmichExportOptions,
  type OnConflict
} from '#lib/api/export.ts';
import { manifestToEdits } from '#lib/edits/manifest.ts';
import { reportUpload } from '#lib/stores/editor/uploadOutcome.ts';
import { errorMessage } from '#lib/utils/errors.ts';
import { createImmichExportJob, createZipExportJob, type Job } from '#lib/api/jobs.ts';
import { EarlierExports, type EarlierQuery } from '#lib/panels/export/earlierExports.svelte.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import {
  runDirect,
  runLabel,
  runNote,
  runSummary,
  runZips,
  type ExportRun
} from '#lib/panels/export/exportRun.ts';
import { submitPresetExport } from '#lib/panels/export/presetExport.ts';
import { PresetSelection } from '#lib/panels/export/presetSelection.svelte.ts';
import type { PresetExporter } from '#lib/panels/export/presetExporter.ts';
import { exportScope, uploadOptions, type ExportScope } from '#lib/panels/export/scope.ts';
import { canShareFiles } from '#lib/panels/export/shareFile.ts';
import {
  albumNames,
  baseOptions,
  formatLabel,
  formInvalid,
  type Destination,
  type ExportFormValues
} from '#lib/panels/export/settings.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { downloadBlob } from '#lib/utils/download.ts';
import { mdiCloudUpload, mdiDownload, mdiFolderZip } from '@mdi/js';

export async function submitBulkExport({
  form,
  destination,
  scope,
  replaceOriginal,
  onConflict = 'keep_both'
}: {
  form: ExportFormValues;
  destination: Destination;
  scope: ExportScope;
  replaceOriginal: boolean;
  onConflict?: OnConflict;
}): Promise<Job | null> {
  const job = await runBulkJob(
    (assetIds) =>
      destination === 'immich'
        ? createImmichExportJob(assetIds, uploadOptions(form, scope, replaceOriginal, onConflict))
        : createZipExportJob(assetIds, baseOptions(form)),
    'Failed to queue export'
  );
  if (job && destination === 'download') jobs.downloadWhenDone(job.id);
  return job;
}

async function downloadPhoto(assetId: string, form: ExportFormValues): Promise<boolean> {
  const opts = baseOptions(form);
  try {
    const download = await downloadStoredExport(assetId, opts);
    const name = downloadName(assetId, download, opts.format);
    downloadBlob(download.blob, name);
    const warned = download.warnings.length > 0;
    toasts.push(warned ? 'warn' : 'success', `Saved ${name}`, {
      description: warned ? download.warnings.join('\n') : undefined
    });
    return true;
  } catch (e) {
    toasts.fail('Export failed', e);
    return false;
  }
}

async function uploadPhoto(assetId: string, options: ImmichExportOptions): Promise<boolean> {
  try {
    const edits = manifestToEdits((await getEdits(assetId)).manifest);
    reportUpload(assetId, await uploadToImmich(assetId, edits, options));
    return true;
  } catch (e) {
    toasts.push('error', `Upload failed: ${errorMessage(e)}`);
    return false;
  }
}

export class BulkExporter implements PresetExporter {
  busy = $state(false);
  exportingPresets = $state(false);
  replaceOriginal = $state(false);

  readonly presets = new PresetSelection();

  readonly earlier = new EarlierExports();

  readonly earlierPresets = new EarlierExports();

  private picked = $derived(browsing.assets.filter((asset) => selection.selected.has(asset.id)));

  scope = $derived(
    exportScope({ assets: this.picked, count: selection.count, trashOn: immichCapabilities.trash })
  );

  earlierQuery = $derived<EarlierQuery | null>(
    selection.count > 0 && exportSettings.destination === 'immich'
      ? { assetIds: [...selection.selected], presets: [] }
      : null
  );

  presetQuery = $derived<EarlierQuery | null>(
    selection.count > 0 && this.presets.immichNames.length > 0
      ? { assetIds: [...selection.selected], presets: this.presets.immichNames }
      : null
  );

  run = $derived<ExportRun>({
    photos: selection.count,
    destination: exportSettings.destination,
    format: formatLabel(exportSettings.form.format),
    albums: albumNames(exportSettings.form.albumIds)
  });

  label = $derived(runLabel(this.run));

  summary = $derived(runSummary(this.run));

  note = $derived(runNote(this.run));

  zipNotice = $derived(runZips(this.run) && canShareFiles());

  icon = $derived.by(() => {
    if (this.run.destination === 'immich') return mdiCloudUpload;
    return runDirect(this.run) ? mdiDownload : mdiFolderZip;
  });

  disabled = $derived(this.busy || selection.count === 0 || formInvalid(exportSettings.form));

  constructor(private readonly onStarted: () => void) {}

  submit = async (): Promise<void> => {
    if (this.busy) return;
    this.busy = true;
    const started = await this.submitCurrent();
    this.busy = false;
    if (started === null) return;
    this.onStarted();
    if (started) jobs.showJob(started.id);
  };

  exportPresets = async (): Promise<boolean> => {
    if (this.exportingPresets || selection.count === 0) return false;
    this.exportingPresets = true;
    const started = await submitPresetExport(
      [...selection.selected],
      this.presets.picked,
      this.scope,
      this.earlierPresets.take()
    );
    this.exportingPresets = false;
    for (const { destination } of started) this.presets.drop(destination);
    const [first] = started;
    if (!first) return false;
    this.onStarted();
    jobs.showJob(first.job.id);
    return this.presets.picked.length === 0;
  };

  private async submitCurrent(): Promise<Job | false | null> {
    const [picked] = selection.selected;
    const replaceOriginal = this.replaceOriginal;
    this.replaceOriginal = false;
    if (runDirect(this.run) && picked) {
      const done =
        exportSettings.destination === 'immich'
          ? await uploadPhoto(
              picked,
              uploadOptions(exportSettings.form, this.scope, replaceOriginal, this.earlier.take())
            )
          : await downloadPhoto(picked, exportSettings.form);
      return done ? false : null;
    }
    return submitBulkExport({
      form: exportSettings.form,
      destination: exportSettings.destination,
      scope: this.scope,
      replaceOriginal,
      onConflict: this.earlier.take()
    });
  }
}
