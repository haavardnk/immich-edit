import { describe, expect, it } from 'vitest';
import {
  defaultGridSize,
  gridRowHeight,
  GRID_SIZES,
  isGridSize,
  steppedGridSize
} from './gridSizes';

describe('grid sizes', () => {
  it('sizes rows per layout mode', () => {
    expect(GRID_SIZES.map((size) => gridRowHeight(size, 'desktop'))).toEqual([110, 150, 220, 320]);
    expect(GRID_SIZES.map((size) => gridRowHeight(size, 'touch'))).toEqual([56, 80, 112, 160]);
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
