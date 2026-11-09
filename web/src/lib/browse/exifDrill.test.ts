import { describe, expect, it } from 'vitest';
import { drillCrumbs, drillLevel, pickExif, type ExifValues } from './exifDrill';
import { CAMERA_FILTERS } from './exifFilters';

const NONE: ExifValues = { make: [], model: [], lensModel: [], country: [], state: [], city: [] };

function camera(values: Partial<ExifValues>): ExifValues {
  return { ...NONE, ...values };
}

describe('exif drill-down', () => {
  it.each<[Partial<ExifValues>, number]>([
    [{}, 0],
    [{ make: ['FUJIFILM'] }, 1],
    [{ make: ['FUJIFILM', 'SONY'] }, 0],
    [{ make: [''] }, 0],
    [{ make: ['FUJIFILM'], model: ['X-T5'] }, 2],
    [{ make: ['FUJIFILM'], model: ['X-T5'], lensModel: ['XF23'] }, 2],
    [{ make: ['FUJIFILM'], model: ['X-T5', 'X-H2'] }, 1],
    [{ model: ['X-T5'] }, 1]
  ])('opens %j at level %i', (values, level) => {
    expect(drillLevel(CAMERA_FILTERS, camera(values))).toBe(level);
  });

  it('names the path down to the level', () => {
    const values = camera({ make: ['FUJIFILM'], model: ['X-T5'] });
    expect(drillCrumbs(CAMERA_FILTERS, values, 2)).toEqual([
      { level: 0, label: 'Makes' },
      { level: 1, label: 'FUJIFILM' },
      { level: 2, label: 'X-T5' }
    ]);
    expect(drillCrumbs(CAMERA_FILTERS, values, 0)).toEqual([]);
    expect(drillCrumbs(CAMERA_FILTERS, camera({ model: ['X-T5'] }), 1)).toEqual([
      { level: 0, label: 'Makes' },
      { level: 1, label: 'Any make' }
    ]);
  });

  it('replaces a value and clears deeper levels when single-select', () => {
    const values = camera({ make: ['FUJIFILM'], model: ['X-T5'], lensModel: ['XF23'] });
    expect(pickExif(CAMERA_FILTERS, values, 0, 'SONY', false)).toEqual({
      make: ['SONY'],
      model: [],
      lensModel: []
    });
  });

  it('adds and removes values when multi-select', () => {
    const values = camera({ make: ['FUJIFILM'], model: ['X-T5'] });
    expect(pickExif(CAMERA_FILTERS, values, 1, 'X-H2', true)).toEqual({
      make: ['FUJIFILM'],
      model: ['X-T5', 'X-H2'],
      lensModel: []
    });
    expect(pickExif(CAMERA_FILTERS, values, 1, 'X-T5', true)).toEqual({
      make: ['FUJIFILM'],
      model: [],
      lensModel: []
    });
  });

  it('keeps Unknown on its own', () => {
    const values = camera({ make: ['FUJIFILM'] });
    expect(pickExif(CAMERA_FILTERS, values, 0, '', true).make).toEqual(['']);
    expect(pickExif(CAMERA_FILTERS, camera({ make: [''] }), 0, 'SONY', true).make).toEqual([
      'SONY'
    ]);
  });
});
