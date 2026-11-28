import { describe, expect, it } from 'vitest';
import { TILE_INFO_OPTIONS, tileInfoLabel } from './viewOptions';

describe('thumbnail info options', () => {
  it('lists every mode in cycle order', () => {
    expect(TILE_INFO_OPTIONS).toEqual([
      { value: 'none', label: 'Hover only' },
      { value: 'icons', label: 'Icons' },
      { value: 'full', label: 'Full' }
    ]);
  });

  it('names the current mode', () => {
    expect(tileInfoLabel('none')).toBe('Thumbnail info: hover only');
  });
});
