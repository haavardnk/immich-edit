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

export interface ExportCtx {
  assetId: string | null;
  asset: AssetDetail | null;
  edits: Edits;
  error: string | null;
  exporting: boolean;
  exportingToImmich: boolean;
  lastExportOpts: ExportOptions | null;
  lastImmichOpts: ImmichExportOptions | null;
  lastUpload: { kind: 'success' | 'duplicate' | 'error'; message: string } | null;
  lastWarnings: string[];
}

export async function onExport(ctx: ExportCtx, opts: ExportOptions): Promise<void> {
  if (!ctx.assetId) return;
  ctx.lastExportOpts = opts;
  ctx.exporting = true;
  try {
    const blob = await downloadExport(ctx.assetId, $state.snapshot(ctx.edits), opts);
    const base = (ctx.asset?.originalFileName ?? ctx.assetId).replace(/\.[^.]+$/, '');
    const name = `${base}_edit.${EXTENSION_BY_FORMAT[opts.format]}`;
    downloadBlob(blob, name);
  } catch (e) {
    ctx.error = errorMessage(e);
  } finally {
    ctx.exporting = false;
  }
}

export async function retryExport(ctx: ExportCtx): Promise<void> {
  if (ctx.lastExportOpts) await onExport(ctx, ctx.lastExportOpts);
}

export async function onUploadToImmich(ctx: ExportCtx, opts: ImmichExportOptions): Promise<void> {
  if (!ctx.assetId) return;
  ctx.lastImmichOpts = opts;
  ctx.exportingToImmich = true;
  ctx.lastUpload = null;
  ctx.lastWarnings = [];
  try {
    const result = await uploadToImmich(ctx.assetId, $state.snapshot(ctx.edits), opts);
    const duplicate = result.status.toLowerCase() === 'duplicate';
    const message = duplicate
      ? `Not uploaded: identical asset already exists in Immich (matched by content hash)`
      : `Uploaded ${result.filename} to Immich`;
    toasts.push(duplicate ? 'warn' : 'success', message, 10000);
    ctx.lastWarnings = result.warnings;
    ctx.lastUpload = { kind: duplicate ? 'duplicate' : 'success', message };
    if (opts.stackWithOriginal || opts.favorite) {
      try {
        ctx.asset = await getAsset(ctx.assetId);
      } catch {
        return;
      }
    }
  } catch (e) {
    const message = errorMessage(e);
    ctx.error = message;
    ctx.lastUpload = { kind: 'error', message: `Upload failed: ${message}` };
    toasts.push('error', `Upload failed: ${message}`, 10000);
  } finally {
    ctx.exportingToImmich = false;
  }
}

export async function retryUpload(ctx: ExportCtx): Promise<void> {
  if (ctx.lastImmichOpts) await onUploadToImmich(ctx, ctx.lastImmichOpts);
}
