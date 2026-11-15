import type { ImmichExportOptions, OnConflict } from '#lib/api/export.ts';
import type { ExportPreset } from '#lib/api/exportPresets.ts';
import { backToGrid } from '#lib/browse/backToGrid.ts';
import { EarlierExports, type EarlierQuery } from '#lib/panels/export/earlierExports.svelte.ts';
import { exportPresets } from '#lib/panels/export/exportPresets.svelte.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { submitPresetExport } from '#lib/panels/export/presetExport.ts';
import { PresetSelection } from '#lib/panels/export/presetSelection.svelte.ts';
import { exportScope, uploadOptions } from '#lib/panels/export/scope.ts';
import { baseOptions, formatLabel, formInvalid } from '#lib/panels/export/settings.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { croppedOutputSize } from '#lib/utils/geom.ts';
import { sameValue } from '#lib/utils/sameValue.ts';
import { page } from '$app/state';
import { mdiCloudUpload, mdiExport } from '@mdi/js';
import type { PresetExporter } from '#lib/panels/export/presetExporter.ts';

export class EditorExporter implements PresetExporter {
  replaceOriginal = $state(false);

  readonly presets = new PresetSelection();

  readonly earlier = new EarlierExports();

  readonly earlierPresets = new EarlierExports();

  exportingPresets = $state(false);

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

  earlierQuery = $derived<EarlierQuery | null>(
    editor.assetId && !this.download ? { assetIds: [editor.assetId], presets: [] } : null
  );

  presetQuery = $derived<EarlierQuery | null>(
    editor.assetId && this.presets.immichNames.length > 0
      ? { assetIds: [editor.assetId], presets: this.presets.immichNames }
      : null
  );

  busy = $derived(this.download ? editor.exports.rendering : editor.exports.uploading);

  result = $derived(this.download ? editor.exports.lastDownload : editor.exports.lastUpload);

  file = $derived.by(() => {
    const file = this.download ? editor.exports.file : null;
    return file && sameValue(file.options, baseOptions(exportSettings.form)) ? file : null;
  });

  folded = $derived(this.download && (this.file !== null || this.busy));

  uploading = $derived(editor.exports.uploading);

  label = $derived.by(() => {
    if (this.busy) return this.download ? 'Exporting…' : 'Uploading…';
    const format = formatLabel(exportSettings.form.format);
    return this.download ? `Export ${format}` : `Upload ${format} to Immich`;
  });

  icon = $derived(this.download ? mdiExport : mdiCloudUpload);

  disabled = $derived(this.busy || !editor.assetId || formInvalid(exportSettings.form));

  start = (): void => this.run(exportSettings.saveAutomatically, this.earlier.take());

  private run(save: boolean, onConflict: OnConflict, preset?: string): void {
    if (this.download) {
      void editor.exports.render(baseOptions(exportSettings.form), save);
      return;
    }
    const options: ImmichExportOptions = {
      ...uploadOptions(exportSettings.form, this.scope, this.replaceOriginal, onConflict),
      ...(preset === undefined ? {} : { preset })
    };
    this.replaceOriginal = false;
    void this.afterUpload(editor.exports.upload(options));
  }

  retry = (): void => {
    if (this.download) void editor.exports.retryRender();
    else void this.afterUpload(editor.exports.retryUpload());
  };

  save = (): void => editor.exports.save();

  uploadFile = (): void => {
    const file = this.file;
    if (!file) return;
    const onConflict = this.earlier.take();
    const options = {
      ...uploadOptions(exportSettings.form, this.scope, false, onConflict),
      ...file.options
    };
    void this.afterUpload(editor.exports.upload(options), false);
  };

  exportPresets = async (): Promise<boolean> => {
    const assetId = editor.assetId;
    if (!assetId || this.exportingPresets) return false;
    const [only, ...rest] = this.presets.picked;
    if (only && rest.length === 0) return this.exportOne(only);
    this.exportingPresets = true;
    await editor.settled();
    if (editor.saveError) {
      this.exportingPresets = false;
      toasts.push('error', 'Presets export the saved edits; save them first');
      return false;
    }
    const started = await submitPresetExport(
      [assetId],
      this.presets.picked,
      this.scope,
      this.earlierPresets.take()
    );
    this.exportingPresets = false;
    for (const { destination } of started) this.presets.drop(destination);
    const [first] = started;
    if (!first) return false;
    jobs.showJob(first.job.id);
    return this.presets.picked.length === 0;
  };

  private exportOne(preset: ExportPreset): boolean {
    if (editor.exports.rendering || editor.exports.uploading) return false;
    const onConflict = this.earlierPresets.take();
    exportPresets.apply(preset);
    this.presets.clear();
    if (this.disabled) return true;
    if (preset.destination === 'immich') this.run(true, onConflict, preset.name);
    else this.run(true, 'keep_both');
    return true;
  }

  private async afterUpload(upload: Promise<boolean>, leave = true): Promise<void> {
    const assetId = editor.assetId;
    const done = await upload;
    this.earlier.refresh();
    if (done && leave) await backToGrid(assetId, page.url.searchParams.get('from'));
  }
}
