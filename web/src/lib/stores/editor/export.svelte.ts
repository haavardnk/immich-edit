import { getAsset } from '$lib/api/assets';
import {
  downloadExport,
  EXTENSION_BY_FORMAT,
  uploadToImmich,
  type ExportOptions,
  type ImmichExportOptions
} from '$lib/api/export';
import { toasts } from '$lib/stores/toasts.svelte';
import type { AssetDetail } from '$lib/types/asset';
import type { Edits } from '$lib/types/edits';
import { downloadBlob } from '$lib/utils/download';
import { errorMessage } from '$lib/utils/errors';

export interface ExportResult {
  kind: 'success' | 'duplicate' | 'error';
  message: string;
  warnings: string[];
}

export interface ExportCtx {
  assetId: string | null;
  asset: AssetDetail | null;
  edits: Edits;
  exporting: boolean;
  exportingToImmich: boolean;
  lastDownloadOpts: ExportOptions | null;
  lastImmichOpts: ImmichExportOptions | null;
  lastDownload: ExportResult | null;
  lastUpload: ExportResult | null;
}

export async function onExport(ctx: ExportCtx, opts: ExportOptions): Promise<void> {
  if (!ctx.assetId) return;
  ctx.lastDownloadOpts = opts;
  ctx.exporting = true;
  ctx.lastDownload = null;
  try {
    const download = await downloadExport(ctx.assetId, $state.snapshot(ctx.edits), opts);
    const name = download.filename ?? `${ctx.assetId}.${EXTENSION_BY_FORMAT[opts.format]}`;
    downloadBlob(download.blob, name);
    ctx.lastDownload = {
      kind: 'success',
      message: `Saved ${name}`,
      warnings: download.warnings
    };
  } catch (e) {
    ctx.lastDownload = {
      kind: 'error',
      message: `Export failed: ${errorMessage(e)}`,
      warnings: []
    };
  } finally {
    ctx.exporting = false;
  }
}

export async function retryExport(ctx: ExportCtx): Promise<void> {
  if (ctx.lastDownloadOpts) await onExport(ctx, ctx.lastDownloadOpts);
}

export async function onUploadToImmich(ctx: ExportCtx, opts: ImmichExportOptions): Promise<void> {
  if (!ctx.assetId) return;
  ctx.lastImmichOpts = opts;
  ctx.exportingToImmich = true;
  ctx.lastUpload = null;
  try {
    const result = await uploadToImmich(ctx.assetId, $state.snapshot(ctx.edits), opts);
    const duplicate = result.status.toLowerCase() === 'duplicate';
    const message = duplicate
      ? `Not uploaded: identical asset already exists in Immich (matched by content hash)`
      : `Uploaded ${result.filename} to Immich`;
    toasts.push(duplicate ? 'warn' : 'success', message, 10000);
    ctx.lastUpload = {
      kind: duplicate ? 'duplicate' : 'success',
      message,
      warnings: result.warnings
    };
    if (opts.stackWithOriginal || opts.favorite) {
      try {
        ctx.asset = await getAsset(ctx.assetId);
      } catch {
        return;
      }
    }
  } catch (e) {
    const message = errorMessage(e);
    ctx.lastUpload = { kind: 'error', message: `Upload failed: ${message}`, warnings: [] };
    toasts.push('error', `Upload failed: ${message}`, 10000);
  } finally {
    ctx.exportingToImmich = false;
  }
}

export async function retryUpload(ctx: ExportCtx): Promise<void> {
  if (ctx.lastImmichOpts) await onUploadToImmich(ctx, ctx.lastImmichOpts);
}
