import { describe, expect, it } from 'vitest';
import { TILE_INFO_OPTIONS, tileInfoLabel } from './viewOptions';

describe('thumbnail info options', () => {
  it('lists every mode in cycle order with a capitalised label', () => {
    expect(TILE_INFO_OPTIONS).toEqual([
      { value: 'none', label: 'Hover only' },
      { value: 'badges', label: 'Badges' },
      { value: 'full', label: 'Name and date' }
    ]);
  });

  it('names the current mode for the toggle', () => {
    expect(tileInfoLabel('full')).toBe('Thumbnail info: name and date');
  });
});
