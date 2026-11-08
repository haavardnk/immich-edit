import { runBulkJob } from '#lib/api/bulkJob.ts';
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
import { countOf } from '#lib/utils/plural.ts';
import { mdiCloudUpload, mdiFolderZip } from '@mdi/js';

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

export class BulkExporter {
  busy = $state(false);
  replaceOriginal = $state(false);

  private picked = $derived(browsing.assets.filter((asset) => selection.selected.has(asset.id)));

  scope = $derived(
    exportScope({ assets: this.picked, count: selection.count, trashOn: immichCapabilities.trash })
  );

  summary = $derived(`${countOf(selection.count, 'asset')} selected`);

  private immich = $derived(exportSettings.destination === 'immich');

  label = $derived(
    this.immich
      ? `Export ${selection.count} to Immich`
      : `Download ${selection.count} as ${formatLabel(exportSettings.form.format)} ZIP`
  );

  icon = $derived(this.immich ? mdiCloudUpload : mdiFolderZip);

  disabled = $derived(this.busy || selection.count === 0 || formInvalid(exportSettings.form));

  constructor(private readonly onStarted: () => void) {}

  submit = async (): Promise<void> => {
    if (this.busy) return;
    this.busy = true;
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
