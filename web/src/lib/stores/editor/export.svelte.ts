import { getAsset } from '#lib/api/assets.ts';
import {
  downloadExport,
  downloadName,
  uploadToImmich,
  type ExportOptions,
  type ImmichExportOptions
} from '#lib/api/export.ts';
import { resizedSize, type PixelSize } from '#lib/panels/export/resize.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { reportUpload } from '#lib/stores/editor/uploadOutcome.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import type { Edits } from '#lib/types/edits.ts';
import type { PreviewMeta } from '#lib/types/preview.ts';
import { downloadBlob } from '#lib/utils/download.ts';
import { errorMessage } from '#lib/utils/errors.ts';
import { croppedOutputSize } from '#lib/utils/geom.ts';
import { sameValue } from '#lib/utils/sameValue.ts';

export interface ExportResult {
  kind: 'success' | 'warning' | 'error';
  message: string;
  warnings: string[];
}

export interface ExportedFile {
  blob: Blob;
  name: string;
  options: ExportOptions;
  edits: Edits;
  size: PixelSize | null;
  renderedAt: Date;
  renderMs: number;
  saved: boolean;
  warnings: string[];
}

export interface ExportCtx {
  readonly assetId: string | null;
  asset: AssetDetail | null;
  readonly edits: Edits;
  readonly meta: PreviewMeta | null;
}

export class EditorExports {
  private ctx: ExportCtx;
  private lastRender: { options: ExportOptions; save: boolean } | null = null;
  private lastUploadOptions: ImmichExportOptions | null = null;
  private session = 0;

  rendering = $state(false);
  uploading = $state(false);
  lastDownload = $state<ExportResult | null>(null);
  lastUpload = $state<ExportResult | null>(null);
  private rendered = $state.raw<ExportedFile | null>(null);

  readonly file = $derived.by<ExportedFile | null>(() => {
    const file = this.rendered;
    return file && sameValue(this.ctx.edits, file.edits) ? file : null;
  });

  constructor(ctx: ExportCtx) {
    this.ctx = ctx;
  }

  render = async (options: ExportOptions, save: boolean): Promise<void> => {
    const assetId = this.ctx.assetId;
    if (!assetId) return;
    const session = this.session;
    this.lastRender = { options, save };
    this.rendering = true;
    this.lastDownload = null;
    this.rendered = null;
    const edits = $state.snapshot(this.ctx.edits) as Edits;
    const size = this.outputSize(edits, options);
    const started = performance.now();
    try {
      const download = await downloadExport(assetId, edits, options);
      if (session !== this.session) return;
      const name = downloadName(assetId, download, options.format);
      if (save) downloadBlob(download.blob, name);
      this.rendered = {
        blob: download.blob,
        name,
        options,
        edits,
        size,
        renderedAt: new Date(),
        renderMs: performance.now() - started,
        saved: save,
        warnings: download.warnings
      };
    } catch (e) {
      if (session !== this.session) return;
      this.lastDownload = {
        kind: 'error',
        message: `Export failed: ${errorMessage(e)}`,
        warnings: []
      };
    } finally {
      if (session === this.session) this.rendering = false;
    }
  };

  retryRender = async (): Promise<void> => {
    if (this.lastRender) await this.render(this.lastRender.options, this.lastRender.save);
  };

  save = (): void => {
    const file = this.file;
    if (!file) return;
    downloadBlob(file.blob, file.name);
    this.rendered = { ...file, saved: true };
  };

  upload = async (options: ImmichExportOptions): Promise<boolean> => {
    const assetId = this.ctx.assetId;
    if (!assetId) return false;
    const session = this.session;
    this.lastUploadOptions = options;
    this.uploading = true;
    this.lastUpload = null;
    try {
      const result = await uploadToImmich(assetId, $state.snapshot(this.ctx.edits), options);
      const { kind, message } = reportUpload(assetId, result);
      if (session !== this.session) return false;
      this.lastUpload = { kind, message, warnings: result.warnings };
      if (result.original_trashed) return true;
      if (options.stackWithOriginal || options.favorite) {
        const asset = await getAsset(assetId).catch(() => null);
        if (asset && session === this.session) this.ctx.asset = asset;
      }
      return false;
    } catch (e) {
      const message = errorMessage(e);
      toasts.push('error', `Upload failed: ${message}`);
      if (session !== this.session) return false;
      this.lastUpload = { kind: 'error', message: `Upload failed: ${message}`, warnings: [] };
      return false;
    } finally {
      if (session === this.session) this.uploading = false;
    }
  };

  retryUpload = async (): Promise<boolean> =>
    this.lastUploadOptions ? this.upload(this.lastUploadOptions) : false;

  reset(): void {
    this.session += 1;
    this.rendered = null;
    this.rendering = false;
    this.uploading = false;
    this.lastDownload = null;
    this.lastUpload = null;
    this.lastRender = null;
    this.lastUploadOptions = null;
  }

  private outputSize(edits: Edits, options: ExportOptions): PixelSize | null {
    const meta = this.ctx.meta;
    if (!meta) return null;
    const crop = croppedOutputSize(edits.geometry, meta.source_w, meta.source_h);
    return resizedSize(crop, options.resize);
  }
}
