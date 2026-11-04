import { backToGrid } from '#lib/browse/backToGrid.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { exportScope, uploadOptions } from '#lib/panels/export/scope.ts';
import { baseOptions, formatLabel, formInvalid } from '#lib/panels/export/settings.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { croppedOutputSize } from '#lib/utils/geom.ts';
import { page } from '$app/state';
import { mdiCloudUpload, mdiExport } from '@mdi/js';

export class EditorExporter {
  replaceOriginal = $state(false);

  scope = $derived(
    exportScope({
      assets: editor.asset ? [editor.asset] : [],
      count: 1,
      trashOn: immichCapabilities.trash
    })
  );

  crop = $derived(
    editor.meta
      ? croppedOutputSize(editor.edits.geometry, editor.meta.source_w, editor.meta.source_h)
      : null
  );

  private download = $derived(exportSettings.destination === 'download');

  busy = $derived(this.download ? editor.exporting : editor.exportingToImmich);

  result = $derived(this.download ? editor.lastDownload : editor.lastUpload);

  label = $derived.by(() => {
    if (this.busy) return this.download ? 'Exporting…' : 'Uploading…';
    const format = formatLabel(exportSettings.form.format);
    return this.download ? `Export ${format}` : `Upload ${format} to Immich`;
  });

  icon = $derived(this.download ? mdiExport : mdiCloudUpload);

  disabled = $derived(this.busy || !editor.assetId || formInvalid(exportSettings.form));

  start = (): void => {
    if (this.download) {
      void editor.onExport(baseOptions(exportSettings.form));
      return;
    }
    const options = uploadOptions(exportSettings.form, this.scope, this.replaceOriginal);
    this.replaceOriginal = false;
    void this.afterUpload(editor.onUploadToImmich(options));
  };

  retry = (): void => {
    if (this.download) void editor.retryExport();
    else void this.afterUpload(editor.retryUpload());
  };

  private async afterUpload(upload: Promise<boolean>): Promise<void> {
    const assetId = editor.assetId;
    if (await upload) await backToGrid(assetId, page.url.searchParams.get('from'));
  }
}
