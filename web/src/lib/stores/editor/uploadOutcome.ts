import type { ImmichExportResult } from '#lib/api/export.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';

const NOT_UPLOADED: Record<string, string> = {
  skipped: 'Not uploaded: this photo is already in Immich from an earlier export',
  duplicate: 'Not uploaded: identical asset already exists in Immich (matched by content hash)'
};

export function uploadOutcome(result: ImmichExportResult): {
  kind: 'success' | 'warning';
  message: string;
} {
  const skipped = NOT_UPLOADED[result.status.toLowerCase()];
  const outcome =
    skipped ??
    (result.replaced_asset_id
      ? `Uploaded ${result.filename} to Immich; the earlier export moved to the Immich trash`
      : `Uploaded ${result.filename} to Immich`);
  return {
    kind: skipped ? 'warning' : 'success',
    message: result.original_trashed
      ? `${outcome}; the original moved to the Immich trash`
      : outcome
  };
}

export function reportUpload(
  assetId: string,
  result: ImmichExportResult
): { kind: 'success' | 'warning'; message: string } {
  const outcome = uploadOutcome(result);
  const trashed = [
    ...(result.original_trashed ? [sourceId(assetId)] : []),
    ...(result.replaced_asset_id ? [result.replaced_asset_id] : [])
  ];
  if (trashed.length > 0) {
    trash.removeTrashed(trashed, outcome.message);
    return outcome;
  }
  const warned = outcome.kind === 'warning' || result.warnings.length > 0;
  toasts.push(warned ? 'warn' : 'success', outcome.message, {
    description: result.warnings.length > 0 ? result.warnings.join('\n') : undefined
  });
  return outcome;
}
