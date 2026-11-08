import { runBulkJob } from '#lib/api/bulkJob.ts';
import { downloadName, downloadStoredExport } from '#lib/api/export.ts';
import { createImmichExportJob, createZipExportJob, type Job } from '#lib/api/jobs.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { exportScope, uploadOptions, type ExportScope } from '#lib/panels/export/scope.ts';
import {
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

export async function downloadPhoto(assetId: string, form: ExportFormValues): Promise<boolean> {
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

  private picked = $derived(browsing.assets.filter((asset) => selection.selected.has(asset.id)));

  scope = $derived(
    exportScope({ assets: this.picked, count: selection.count, trashOn: immichCapabilities.trash })
  );

  private immich = $derived(exportSettings.destination === 'immich');

  private single = $derived(selection.count === 1);

  private direct = $derived(this.single && !this.immich);

  downloadLabel = $derived(this.single ? 'Download' : 'Download ZIP');

  label = $derived.by(() => {
    const format = formatLabel(exportSettings.form.format);
    if (this.immich) return `Export ${selection.count} to Immich`;
    if (this.direct) return `Download ${format}`;
    return `Download ${selection.count} as ${format} ZIP`;
  });

  icon = $derived(this.immich ? mdiCloudUpload : this.direct ? mdiDownload : mdiFolderZip);

  disabled = $derived(this.busy || selection.count === 0 || formInvalid(exportSettings.form));

  constructor(private readonly onStarted: () => void) {}

  submit = async (): Promise<void> => {
    if (this.busy) return;
    this.busy = true;
    const [picked] = selection.selected;
    if (this.direct && picked) {
      const saved = await downloadPhoto(picked, exportSettings.form);
      this.busy = false;
      if (saved) this.onStarted();
      return;
    }
    const replaceOriginal = this.replaceOriginal;
    this.replaceOriginal = false;
    const job = await submitBulkExport({
      form: exportSettings.form,
      destination: exportSettings.destination,
      scope: this.scope,
      replaceOriginal
    });
    this.busy = false;
    if (!job) return;
    this.onStarted();
    jobs.showJob(job.id);
  };
}
