import { GRID_SIZE_LABELS, type GridSize } from '#lib/browse/gridSizes.ts';
import {
  detailFit,
  type OverlayDetail,
  type OverlayIcon,
  type TileInfo,
  type TileOverlay
} from '#lib/browse/tileOverlay.ts';

export const ICON_COLUMNS: readonly (readonly OverlayIcon[])[] = [
  ['edited', 'favorite', 'rejected', 'label', 'rating', 'copy'],
  ['location', 'tags', 'description', 'albums', 'format']
];

export const DETAIL_SAMPLES: Readonly<Record<OverlayDetail, string>> = {
  filename: 'DSCF4120.RAF',
  taken: new Date(Date.UTC(2026, 9, 31, 12)).toLocaleDateString(undefined, { timeZone: 'UTC' }),
  place: 'Bergen, Vestland, Norway',
  tags: 'Travel/Norway, Fjords',
  description: 'Bryggen at first light',
  albums: 'Norway 2026, Portfolio',
  exposure: '23mm · f/8.0 · 1/250s · ISO 160',
  camera: 'FUJIFILM X-T5',
  lens: 'XF16-55mmF2.8 R LM WR',
  size: '7728 × 5152 · 54.2 MB'
};

export const FORMAT_SAMPLE = 'RAF';

export function fitNotice(
  info: TileInfo,
  overlay: TileOverlay,
  rowHeight: number,
  size: GridSize
): string | null {
  const chosen = overlay.details.length;
  const shown = Math.min(detailFit(rowHeight), chosen);
  if (info !== 'full' || shown >= chosen) return null;
  return `Showing ${shown} of ${chosen} at size ${GRID_SIZE_LABELS[size]}. Larger thumbnails show more.`;
}
