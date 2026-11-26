import type { AssetSummary } from '#lib/types/album.ts';

export interface Camera {
  key: string;
  label: string;
  count: number;
}

export interface AllCameras {
  key: string;
  count: number;
}

const UNKNOWN = 'Unknown camera';

export function cameraKey(asset: AssetSummary): string {
  const make = asset.exifInfo?.make?.trim() ?? '';
  const model = asset.exifInfo?.model?.trim() ?? '';
  return `${make}\u0000${model}`;
}

export function cameraLabel(key: string): string {
  const [make = '', model = ''] = key.split('\u0000');
  return model || make || UNKNOWN;
}

export function camerasOf(assets: readonly AssetSummary[]): Camera[] {
  const counts = new Map<string, number>();
  for (const asset of assets) {
    const key = cameraKey(asset);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return [...counts]
    .map(([key, count]) => ({ key, label: cameraLabel(key), count }))
    .sort((a, b) => b.count - a.count || a.label.localeCompare(b.label));
}
