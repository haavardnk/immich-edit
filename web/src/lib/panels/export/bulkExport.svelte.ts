import { runBulkJob } from '#lib/api/bulkJob.ts';
import { downloadName, downloadStoredExport } from '#lib/api/export.ts';
import { createImmichExportJob, createZipExportJob, type Job } from '#lib/api/jobs.ts';
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
import { mdiCloudUpload, mdiDownload, mdiFolderZip, mdiLayersTripleOutline } from '@mdi/js';

export async function submitBulkExport({
  form,
  destination,
  scope,
  replaceOriginal
}: {
  form: ExportFormValues;
  destination: Destination;
  scope: ExportScope;
  replaceOriginal: boolean;
}): Promise<Job | null> {
  const job = await runBulkJob(
    (assetIds) =>
      destination === 'immich'
        ? createImmichExportJob(assetIds, uploadOptions(form, scope, replaceOriginal))
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
    toasts.fail('Export failed', e, 6000);
    return false;
  }
}

export class BulkExporter {
  busy = $state(false);
  replaceOriginal = $state(false);

  readonly presets = new PresetSelection();

  private picked = $derived(browsing.assets.filter((asset) => selection.selected.has(asset.id)));

  scope = $derived(
    exportScope({ assets: this.picked, count: selection.count, trashOn: immichCapabilities.trash })
  );

  usingPresets = $derived(this.presets.picked.length > 0);

  run = $derived<ExportRun>({
    photos: selection.count,
    presets: this.presets.picked.map((preset) => preset.destination),
    destination: exportSettings.destination,
    format: formatLabel(exportSettings.form.format),
    albums: albumNames(exportSettings.form.albumIds)
  });

  downloadLabel = $derived(selection.count === 1 ? 'Download' : 'Download ZIP');

  label = $derived(runLabel(this.run));

  summary = $derived(runSummary(this.run));

  note = $derived(runNote(this.run));

  zipNotice = $derived(runZips(this.run) && canShareFiles());

  icon = $derived.by(() => {
    if (this.presets.jobCount > 1) return mdiLayersTripleOutline;
    const destination = this.run.presets[0] ?? this.run.destination;
    if (destination === 'immich') return mdiCloudUpload;
    return runDirect(this.run) ? mdiDownload : mdiFolderZip;
  });

  disabled = $derived(
    this.busy ||
      selection.count === 0 ||
      this.presets.tooMany ||
      (!this.usingPresets && formInvalid(exportSettings.form))
  );

  constructor(private readonly onStarted: () => void) {}

  submit = async (): Promise<void> => {
    if (this.busy) return;
    this.busy = true;
    const started = this.usingPresets ? await this.submitPresets() : await this.submitCurrent();
    this.busy = false;
    if (started === null) return;
    this.onStarted();
    if (started) jobs.showJob(started.id);
  };

  private async submitPresets(): Promise<Job | null> {
    const started = await submitPresetExport(
      [...selection.selected],
      this.presets.picked,
      this.scope
    );
    for (const { destination } of started) this.presets.drop(destination);
    return started[0]?.job ?? null;
  }

  private async submitCurrent(): Promise<Job | false | null> {
    const [picked] = selection.selected;
    if (runDirect(this.run) && picked) {
      return (await downloadPhoto(picked, exportSettings.form)) ? false : null;
    }
    const replaceOriginal = this.replaceOriginal;
    this.replaceOriginal = false;
    return submitBulkExport({
      form: exportSettings.form,
      destination: exportSettings.destination,
      scope: this.scope,
      replaceOriginal
    });
  }
}
