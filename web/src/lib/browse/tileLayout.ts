import { detailFit, type TileInfo, type TileOverlay } from '#lib/browse/tileOverlay.ts';

export const COUNTS_MIN_HEIGHT = 80;
export const COUNTS_MIN_WIDTH = 90;
export const STACK_BELOW_WIDTH = 120;
export const LOW_BELOW_HEIGHT = 90;
export const PINNED_STARS_MIN_HEIGHT = 140;

export interface TileSize {
  width: number;
  height: number;
}

export interface TileLayout {
  lines: number;
  pinnedStars: boolean;
  counts: boolean;
  stacked: boolean;
  low: boolean;
}

export function tileLayout(info: TileInfo, overlay: TileOverlay, size: TileSize): TileLayout {
  return {
    lines: detailFit(size.height),
    pinnedStars:
      info === 'full' && overlay.icons.includes('rating') && size.height >= PINNED_STARS_MIN_HEIGHT,
    counts: size.height >= COUNTS_MIN_HEIGHT && size.width >= COUNTS_MIN_WIDTH,
    stacked: size.width < STACK_BELOW_WIDTH,
    low: size.height < LOW_BELOW_HEIGHT
  };
}
