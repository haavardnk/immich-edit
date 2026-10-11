import { describe, expect, it } from 'vitest';
import { exclusiveUnknown, exifSummary, suggestionQuery, UNKNOWN_EXIF } from './exifFilters';

const SELECTED = {
  make: ['SONY'],
  model: ['ILCE-7M3', 'X-T5'],
  lensModel: ['FE 35mm F1.8'],
  country: ['Norway'],
  state: [],
  city: ['Bergen']
};

describe('exif filters', () => {
  it.each([
    ['make', { field: 'make' }],
    ['model', { field: 'model', make: ['SONY'] }],
    ['lensModel', { field: 'lensModel', make: ['SONY'], model: ['ILCE-7M3', 'X-T5'] }],
    ['country', { field: 'country' }],
    ['state', { field: 'state', country: ['Norway'] }],
    ['city', { field: 'city', country: ['Norway'] }]
  ] as const)('asks for %s narrowed by its parents', (key, expected) => {
    expect(suggestionQuery(key, SELECTED)).toEqual(expected);
  });

  it('leaves out parents with nothing selected', () => {
    expect(suggestionQuery('lensModel', { ...SELECTED, make: [] })).toEqual({
      field: 'lensModel',
      model: ['ILCE-7M3', 'X-T5']
    });
  });

  it('does not narrow by an unknown parent', () => {
    expect(suggestionQuery('model', { ...SELECTED, make: [UNKNOWN_EXIF] })).toEqual({
      field: 'model'
    });
  });

  it.each([
    [['SONY', UNKNOWN_EXIF], [UNKNOWN_EXIF]],
    [[UNKNOWN_EXIF, 'SONY'], ['SONY']],
    [
      ['SONY', 'Canon'],
      ['SONY', 'Canon']
    ],
    [[], []]
  ])('picking %j keeps %j', (picked, kept) => {
    expect(exclusiveUnknown(picked)).toEqual(kept);
  });

  it.each([
    [[], ''],
    [[UNKNOWN_EXIF], 'Unknown'],
    [['SONY', 'Canon'], 'SONY +1']
  ])('summarises %j as %s', (values, summary) => {
    expect(exifSummary(values)).toBe(summary);
  });
});
