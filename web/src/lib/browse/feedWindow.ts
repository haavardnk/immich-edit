import type { AssetSummary } from '#lib/types/album.ts';

type Listed = Pick<AssetSummary, 'id' | 'copyOf'>;

export type WindowEnd = 'first' | 'last';

export function windowEdges(assets: readonly Listed[], end: WindowEnd, count: number): string[] {
  const masters: string[] = [];
  const ordered = end === 'first' ? assets : [...assets].reverse();
  for (const asset of ordered) {
    if (masters.length >= count) break;
    const id = asset.copyOf ?? asset.id;
    if (!masters.includes(id)) masters.push(id);
  }
  return masters;
}

export function windowEdge(assets: readonly Listed[], end: WindowEnd): string | null {
  return windowEdges(assets, end, 1)[0] ?? null;
}

export function singleAnchor(items: readonly Listed[]): boolean {
  return items.length > 0 && windowEdges(items, 'first', 2).length === 1;
}

export function mergeWindow<T extends Listed>(
  loaded: readonly T[],
  incoming: readonly T[],
  side: 'before' | 'after'
): T[] {
  const known = new Set(loaded.map((asset) => asset.id));
  const fresh = incoming.filter((asset) => {
    if (known.has(asset.id)) return false;
    known.add(asset.id);
    return true;
  });
  return side === 'before' ? [...fresh, ...loaded] : [...loaded, ...fresh];
}
