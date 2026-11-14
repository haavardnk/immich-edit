import type { ImmichExportResult } from '#lib/api/export.ts';

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
