import { describe, expect, it } from 'vitest';
import { OVERLAY_DETAILS, OVERLAY_ICONS, type TileOverlay } from './tileOverlay';
import { DETAIL_SAMPLES, fitNotice, ICON_COLUMNS } from './tileOverlayText';

const FOUR: TileOverlay = { icons: [], details: ['filename', 'taken', 'place', 'exposure'] };

describe('tile overlay text', () => {
  it('lays out every icon once across the two columns', () => {
    expect(ICON_COLUMNS.flat().toSorted()).toEqual([...OVERLAY_ICONS].toSorted());
  });

  it('has a sample for every detail', () => {
    expect(Object.keys(DETAIL_SAMPLES).toSorted()).toEqual([...OVERLAY_DETAILS].toSorted());
  });

  it('explains when the size shows fewer details than chosen in full mode', () => {
    expect(fitNotice('full', FOUR, 150, 'md')).toBe(
      'Showing 2 of 4 at size M. Larger thumbnails show more.'
    );
    expect(fitNotice('full', FOUR, 320, 'xl')).toBeNull();
    expect(fitNotice('icons', FOUR, 150, 'md')).toBeNull();
    expect(fitNotice('full', { icons: [], details: ['filename'] }, 110, 'sm')).toBeNull();
  });
});
