import type { ImmichExportOptions } from '#lib/api/export.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { NONE_OWNED_STACK, NOT_OWNER_STACK, ownsAsset } from '#lib/browse/ownership.ts';
import { captureDate, type NameContext } from '#lib/editor/filenameTemplate.ts';
import { replaceBlocked } from '#lib/panels/export/replace.ts';
import { immichOptions, type ExportFormValues } from '#lib/panels/export/settings.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export type ExportAsset = Parameters<typeof captureDate>[0] &
  Parameters<typeof ownsAsset>[0] &
  Pick<AssetSummary, 'id' | 'originalFileName'>;

export interface ExportScope {
  stackable: boolean;
  stackReason: string;
  replaceReason: string | null;
  nameExample: NameContext | null;
}

export function exportScope({
  assets,
  count,
  trashOn
}: {
  assets: readonly ExportAsset[];
  count: number;
  trashOn: boolean;
}): ExportScope {
  const first = assets[0];
  const single = count === 1;
  const stackable = assets.length < count || assets.some(ownsAsset);
  return {
    stackable,
    stackReason: single ? NOT_OWNER_STACK : NONE_OWNED_STACK,
    replaceReason: replaceBlocked({
      owned: stackable,
      trashOn,
      versionsOfOnePhoto: new Set(assets.map((a) => sourceId(a.id))).size < assets.length,
      single
    }),
    nameExample: first
      ? {
          original: first.originalFileName,
          date: captureDate(first),
          position: 1,
          total: count
        }
      : null
  };
}

export function uploadOptions(
  form: ExportFormValues,
  scope: ExportScope,
  replaceOriginal: boolean
): ImmichExportOptions {
  return immichOptions(form, {
    stackable: scope.stackable,
    replaceOriginal: replaceOriginal && scope.replaceReason === null
  });
}
