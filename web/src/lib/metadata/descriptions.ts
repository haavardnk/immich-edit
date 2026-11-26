import type { AssetSummary } from '#lib/types/album.ts';

export interface DescriptionValue {
  text: string;
  count: number;
}

export function descriptionOf(asset: AssetSummary): string {
  return asset.exifInfo?.description?.trim() ?? '';
}

export function descriptionValues(assets: readonly AssetSummary[]): DescriptionValue[] {
  const counts = new Map<string, number>();
  for (const asset of assets) {
    const text = descriptionOf(asset);
    counts.set(text, (counts.get(text) ?? 0) + 1);
  }
  return [...counts]
    .map(([text, count]) => ({ text, count }))
    .sort((a, b) => b.count - a.count || a.text.localeCompare(b.text));
}

export interface DescriptionChange {
  id: string;
  from: string;
  to: string;
}

export function descriptionChanges(
  assets: readonly AssetSummary[],
  to: string
): DescriptionChange[] {
  return assets.flatMap((asset) =>
    descriptionOf(asset) === to
      ? []
      : [{ id: asset.id, from: asset.exifInfo?.description ?? '', to }]
  );
}
