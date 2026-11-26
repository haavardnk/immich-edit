import { assetSummaries } from '#lib/api/map.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export async function resolveAssets(ids: readonly string[]): Promise<AssetSummary[]> {
  const found = ids.map((id) => browsing.find(id));
  const missing = ids.filter((_, index) => !found[index]);
  const fetched = missing.length > 0 ? await assetSummaries(missing) : [];
  const byId = new Map(fetched.map((asset) => [asset.id, asset]));
  return ids.flatMap((id, index) => {
    const asset = found[index] ?? byId.get(sourceId(id));
    return asset ? [asset] : [];
  });
}
