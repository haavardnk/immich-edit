import { albumsWithAsset } from '#lib/api/albums.ts';
import { sourceId } from '#lib/browse/assetKey.ts';

export const MEMBERSHIP_CAP = 100;
const CONCURRENCY = 6;

export async function albumMembership(ids: readonly string[]): Promise<Map<string, number> | null> {
  const sources = [...new Set(ids.map(sourceId))];
  if (sources.length > MEMBERSHIP_CAP) return null;
  const counts = new Map<string, number>();
  let next = 0;
  async function worker(): Promise<void> {
    while (next < sources.length) {
      const id = sources[next++];
      if (!id) continue;
      for (const album of await albumsWithAsset(id)) {
        counts.set(album.id, (counts.get(album.id) ?? 0) + 1);
      }
    }
  }
  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, sources.length) }, worker));
  return counts;
}
