import type { LayoutMode } from '#lib/device/layoutRules.ts';

export type GridSize = 'sm' | 'md' | 'lg' | 'xl';

export type GridLayout = { mode: LayoutMode; wide: boolean };

export type RowLayout = { mode: LayoutMode; phone: boolean };

type GridDevice = 'desktop' | 'tablet' | 'phone';

export const GRID_SIZES: readonly GridSize[] = ['sm', 'md', 'lg', 'xl'];

export const GRID_SIZE_LABELS: Readonly<Record<GridSize, string>> = {
  sm: 'S',
  md: 'M',
  lg: 'L',
  xl: 'XL'
};

const ROW_HEIGHTS: Readonly<Record<GridDevice, Readonly<Record<GridSize, number>>>> = {
  desktop: { sm: 110, md: 150, lg: 220, xl: 320 },
  tablet: { sm: 56, md: 80, lg: 112, xl: 160 },
  phone: { sm: 72, md: 100, lg: 140, xl: 220 }
};

const ROW_TOLERANCE = 0.5;
const PHONE_ROW_TOLERANCE = 0.15;

const DAY_HEADER_HEIGHTS: Readonly<Record<LayoutMode, number>> = {
  desktop: 42,
  touch: 34
};

export function dayHeaderHeight(mode: LayoutMode): number {
  return DAY_HEADER_HEIGHTS[mode];
}

export function isGridSize(value: unknown): value is GridSize {
  return GRID_SIZES.some((size) => size === value);
}

function gridDevice({ mode, phone }: RowLayout): GridDevice {
  if (mode === 'desktop') return 'desktop';
  return phone ? 'phone' : 'tablet';
}

export function gridRowHeight(size: GridSize, layout: RowLayout): number {
  return ROW_HEIGHTS[gridDevice(layout)][size];
}

export function gridRowTolerance(phone: boolean): number {
  return phone ? PHONE_ROW_TOLERANCE : ROW_TOLERANCE;
}

export function defaultGridSize({ mode, wide }: GridLayout): GridSize {
  if (mode === 'desktop') return 'md';
  return wide ? 'xl' : 'lg';
}

export function steppedGridSize(size: GridSize, delta: number): GridSize {
  const index = GRID_SIZES.indexOf(size) + Math.trunc(delta);
  return GRID_SIZES[Math.min(GRID_SIZES.length - 1, Math.max(0, index))] ?? size;
}
