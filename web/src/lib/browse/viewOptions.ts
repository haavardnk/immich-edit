import { GRID_SIZES, GRID_SIZE_LABELS, type GridSize } from '#lib/browse/gridSizes.ts';
import {
  EDIT_SORT_LABELS,
  SORT_DIRECTION_LABELS,
  SORT_FIELD_LABELS
} from '#lib/browse/sortOptions.ts';
import type { SortDir, SortField } from '#lib/stores/browseControls.svelte.ts';
import type { TileInfo } from '#lib/stores/browseView.svelte.ts';

export const GRID_SIZE_OPTIONS: readonly { value: GridSize; label: string; title: string }[] =
  GRID_SIZES.map((value) => {
    const label = GRID_SIZE_LABELS[value];
    return { value, label, title: `Thumbnail size ${label}` };
  });

const TILE_INFO_LABELS: Readonly<Record<TileInfo, string>> = {
  none: 'hover only',
  badges: 'badges',
  full: 'name and date'
};

function tileInfoName(info: TileInfo): string {
  const label = TILE_INFO_LABELS[info];
  return `${label.charAt(0).toUpperCase()}${label.slice(1)}`;
}

export const TILE_INFO_OPTIONS: readonly { value: TileInfo; label: string }[] = (
  Object.keys(TILE_INFO_LABELS) as TileInfo[]
).map((value) => ({ value, label: tileInfoName(value) }));

export function tileInfoLabel(info: TileInfo): string {
  return `Thumbnail info: ${TILE_INFO_LABELS[info]}`;
}

export type ViewState = {
  basis: 'capture' | 'edit' | null;
  field: SortField;
  dir: SortDir;
  size: GridSize;
  info: TileInfo;
};

function withoutFirst(label: string): string {
  return label.replace(/ first$/, '');
}

function sortSummary({ basis, field, dir }: ViewState): string | null {
  if (basis === 'edit') return withoutFirst(EDIT_SORT_LABELS[dir]);
  if (basis === 'capture')
    return `${SORT_FIELD_LABELS[field]}, ${withoutFirst(SORT_DIRECTION_LABELS[field][dir]).toLowerCase()}`;
  return null;
}

export function viewSummary(view: ViewState): string {
  return [sortSummary(view), GRID_SIZE_LABELS[view.size], tileInfoName(view.info)]
    .filter((part) => part !== null)
    .join(' · ');
}
