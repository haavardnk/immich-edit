import { describe, expect, it } from 'vitest';
import {
  dayHeaderHeight,
  defaultGridSize,
  gridRowHeight,
  gridRowTolerance,
  GRID_SIZES,
  isGridSize,
  steppedGridSize
} from './gridSizes';

describe('grid sizes', () => {
  it('sizes rows for desktop, tablet and phone', () => {
    const heights = (mode: 'desktop' | 'touch', phone: boolean) =>
      GRID_SIZES.map((size) => gridRowHeight(size, { mode, phone }));
    expect(heights('desktop', false)).toEqual([110, 150, 220, 320]);
    expect(heights('touch', false)).toEqual([56, 80, 112, 160]);
    expect(heights('touch', true)).toEqual([72, 100, 140, 220]);
  });

  it('holds phone rows closer to their height than wider layouts', () => {
    expect(gridRowTolerance(true)).toBe(0.15);
    expect(gridRowTolerance(false)).toBe(0.5);
  });

  it('defaults to M on desktop, L on a phone and XL on a wide tablet', () => {
    expect(defaultGridSize({ mode: 'desktop', wide: false })).toBe('md');
    expect(defaultGridSize({ mode: 'touch', wide: false })).toBe('lg');
    expect(defaultGridSize({ mode: 'touch', wide: true })).toBe('xl');
  });

  it('steps within the four sizes', () => {
    expect(steppedGridSize('md', 1)).toBe('lg');
    expect(steppedGridSize('md', -1)).toBe('sm');
    expect(steppedGridSize('sm', -3)).toBe('sm');
    expect(steppedGridSize('sm', 2)).toBe('lg');
    expect(steppedGridSize('xl', 1)).toBe('xl');
  });

  it('accepts only known sizes', () => {
    expect(isGridSize('xl')).toBe(true);
    expect(isGridSize('huge')).toBe(false);
    expect(isGridSize(undefined)).toBe(false);
  });
});

describe('day header height', () => {
  it('is shorter on touch', () => {
    expect(dayHeaderHeight('desktop')).toBe(42);
    expect(dayHeaderHeight('touch')).toBe(34);
  });
});
