import type { AssetSummary } from '$lib/types/album';
import type { SortDir, SortField } from '$lib/types/search';

export type SortKey = (asset: AssetSummary) => number | null;

interface Group {
  items: AssetSummary[];
  at: number | null;
}

export function timeOf(value: string | null | undefined): number | null {
  if (!value) return null;
  const ms = Date.parse(value);
  return Number.isNaN(ms) ? null : ms;
}

export function fieldKey(field: SortField): SortKey {
  if (field === 'rating') return (a) => a.exifInfo?.rating ?? null;
  if (field === 'size') return (a) => a.exifInfo?.fileSizeInByte ?? null;
  if (field === 'added') return (a) => timeOf(a.createdAt);
  return (a) => timeOf(a.fileCreatedAt);
}

export function sortAssets(assets: AssetSummary[], key: SortKey, dir: SortDir): AssetSummary[] {
  const groups: Group[] = [];
  for (const asset of assets) {
    const last = groups[groups.length - 1];
    if (asset.copyOf && last && last.items[0]?.id === asset.copyOf) {
      last.items.push(asset);
      continue;
    }
    groups.push({ items: [asset], at: key(asset) });
  }
  const sign = dir === 'asc' ? 1 : -1;
  return groups
    .slice()
    .sort((a, b) => {
      if (a.at === null || b.at === null) {
        if (a.at === b.at) return 0;
        return a.at === null ? 1 : -1;
      }
      return (a.at - b.at) * sign;
    })
    .flatMap((group) => group.items);
}
