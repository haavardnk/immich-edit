import type { AssetSummary } from '$lib/types/album';
import type { SortDir } from '$lib/types/search';

interface Group {
  items: AssetSummary[];
  at: number | null;
}

function timeOf(value: string | null): number | null {
  if (!value) return null;
  const ms = Date.parse(value);
  return Number.isNaN(ms) ? null : ms;
}

export function sortAssets(
  assets: AssetSummary[],
  at: (asset: AssetSummary) => string | null,
  dir: SortDir
): AssetSummary[] {
  const groups: Group[] = [];
  for (const asset of assets) {
    const last = groups[groups.length - 1];
    if (asset.copyOf && last && last.items[0].id === asset.copyOf) {
      last.items.push(asset);
      continue;
    }
    groups.push({ items: [asset], at: timeOf(at(asset)) });
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
