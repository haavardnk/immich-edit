import type { GridSize, TileInfo } from '#lib/stores/browseView.svelte.ts';

export const GRID_SIZE_OPTIONS: readonly { value: GridSize; label: string }[] = [
  { value: 'sm', label: 'S' },
  { value: 'md', label: 'M' },
  { value: 'lg', label: 'L' },
  { value: 'xl', label: 'XL' }
];

const TILE_INFO_LABELS: Readonly<Record<TileInfo, string>> = {
  none: 'hover only',
  badges: 'badges',
  full: 'name and date'
};

export const TILE_INFO_OPTIONS: readonly { value: TileInfo; label: string }[] = (
  Object.keys(TILE_INFO_LABELS) as TileInfo[]
).map((value) => {
  const label = TILE_INFO_LABELS[value];
  return { value, label: `${label.charAt(0).toUpperCase()}${label.slice(1)}` };
});

export function tileInfoLabel(info: TileInfo): string {
  return `Thumbnail info: ${TILE_INFO_LABELS[info]}`;
}
