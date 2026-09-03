import { getJson, url } from './client';
import type { AssetDetail } from '$lib/types/asset';

export function folderPaths(): Promise<string[]> {
  return getJson('/api/folders/paths');
}

export function folderAssets(path: string): Promise<AssetDetail[]> {
  return getJson(url`/api/folders/assets?path=${path}`);
}
