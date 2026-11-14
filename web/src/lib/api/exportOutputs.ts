import { sendJson } from './client';

export interface ExportedFile {
  filename: string;
  exported_at: string;
}

export interface EarlierExports {
  photos: number;
  files: ExportedFile[];
}

export function findEarlierExports(
  assetIds: readonly string[],
  presets: readonly string[]
): Promise<EarlierExports> {
  return sendJson<EarlierExports>(
    'POST',
    '/api/export-outputs/query',
    { asset_ids: assetIds, presets },
    undefined,
    { silent: true }
  );
}
