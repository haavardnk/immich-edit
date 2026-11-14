import { describe, expect, it } from 'vitest';
import type { ImmichExportResult } from '#lib/api/export.ts';
import { uploadOutcome } from './uploadOutcome';

function result(patch: Partial<ImmichExportResult> = {}): ImmichExportResult {
  return {
    asset_id: 'new',
    filename: 'IMG_0001_edit.jpg',
    status: 'created',
    warnings: [],
    original_trashed: false,
    ...patch
  };
}

describe('upload outcome', () => {
  it('reports a new upload', () => {
    expect(uploadOutcome(result())).toEqual({
      kind: 'success',
      message: 'Uploaded IMG_0001_edit.jpg to Immich'
    });
  });

  it('warns when nothing was uploaded', () => {
    expect(uploadOutcome(result({ status: 'skipped' }))).toEqual({
      kind: 'warning',
      message: 'Not uploaded: this photo is already in Immich from an earlier export'
    });
    expect(uploadOutcome(result({ status: 'duplicate' })).kind).toBe('warning');
  });

  it('says where the replaced export and the original went', () => {
    expect(uploadOutcome(result({ replaced_asset_id: 'old' })).message).toBe(
      'Uploaded IMG_0001_edit.jpg to Immich; the earlier export moved to the Immich trash'
    );
    expect(uploadOutcome(result({ original_trashed: true })).message).toBe(
      'Uploaded IMG_0001_edit.jpg to Immich; the original moved to the Immich trash'
    );
  });
});
