import { GRID_SIZES, GRID_SIZE_LABELS, type GridSize } from '#lib/browse/gridSizes.ts';
import { TILE_INFO_ORDER, type TileInfo } from '#lib/browse/tileOverlay.ts';

export const GRID_SIZE_OPTIONS: readonly { value: GridSize; label: string; title: string }[] =
  GRID_SIZES.map((value) => {
    const label = GRID_SIZE_LABELS[value];
    return { value, label, title: `Thumbnail size ${label}` };
  });

const TILE_INFO_LABELS: Readonly<Record<TileInfo, string>> = {
  none: 'Hover only',
  icons: 'Icons',
  full: 'Full'
};

export const TILE_INFO_OPTIONS: readonly { value: TileInfo; label: string }[] = TILE_INFO_ORDER.map(
  (value) => ({ value, label: TILE_INFO_LABELS[value] })
);

export function tileInfoLabel(info: TileInfo): string {
  return `Thumbnail info: ${TILE_INFO_LABELS[info].toLowerCase()}`;
}
