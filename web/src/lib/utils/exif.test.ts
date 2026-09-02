import { describe, it, expect } from 'vitest';
import {
  fmtAperture,
  fmtFocal,
  fmtShutter,
  fmtDim,
  fmtSize,
  fmtDate,
  fmtCamera,
  exifDetailRows
} from './exif';
import type { ExifInfo } from '$lib/types/asset';

describe('fmtAperture', () => {
  it.each([
    [null, null],
    [1.8, 'f/1.8'],
    [2.0, 'f/2.0'],
    [11, 'f/11']
  ])('formats %s as %s', (input, expected) => {
    expect(fmtAperture(input)).toBe(expected);
  });
});

describe('fmtFocal', () => {
  it('rounds to whole millimetres', () => {
    expect(fmtFocal(35.4)).toBe('35mm');
    expect(fmtFocal(null)).toBeNull();
  });
});

describe('fmtShutter', () => {
  it.each([
    [null, null],
    ['', null],
    ['0.5', '1/2s'],
    ['0.005', '1/200s'],
    ['2', '2.0s'],
    ['not-a-number', 'not-a-number']
  ])('formats %s as %s', (input, expected) => {
    expect(fmtShutter(input)).toBe(expected);
  });
});

describe('fmtDim', () => {
  it('formats width and height', () => {
    expect(fmtDim(4000, 3000)).toBe('4000 × 3000');
    expect(fmtDim(0, 3000)).toBeNull();
    expect(fmtDim(null, null)).toBeNull();
  });
});

describe('fmtSize', () => {
  it('uses MB above one megabyte', () => {
    expect(fmtSize(5 * 1024 * 1024)).toBe('5.0 MB');
  });

  it('uses KB below one megabyte', () => {
    expect(fmtSize(512 * 1024)).toBe('512 KB');
  });

  it('returns null for missing size', () => {
    expect(fmtSize(null)).toBeNull();
    expect(fmtSize(0)).toBeNull();
  });
});

describe('fmtDate', () => {
  it('returns a non-empty string for a valid date', () => {
    const out = fmtDate('2024-01-02T03:04:05Z');
    expect(out).not.toBeNull();
    expect((out as string).length).toBeGreaterThan(0);
  });

  it('returns the input for an invalid date', () => {
    expect(fmtDate('garbage')).toBe('garbage');
    expect(fmtDate(null)).toBeNull();
  });
});

describe('fmtCamera', () => {
  it('joins make and model', () => {
    expect(fmtCamera('Sony', 'A7')).toBe('Sony A7');
    expect(fmtCamera(null, 'A7')).toBe('A7');
    expect(fmtCamera(null, null)).toBeNull();
  });
});

describe('exifDetailRows', () => {
  it('returns an empty array for null exif', () => {
    expect(exifDetailRows(null)).toEqual([]);
  });

  it('omits rows with no value', () => {
    const exif: ExifInfo = {
      make: 'Sony',
      model: 'A7',
      lensModel: null,
      fNumber: 2.8,
      focalLength: null,
      iso: 200,
      exposureTime: null,
      exifImageWidth: 4000,
      exifImageHeight: 3000,
      orientation: null,
      dateTimeOriginal: null,
      rating: null,
      fileSizeInByte: null
    };
    const rows = exifDetailRows(exif);
    const keys = rows.map((r) => r.key);
    expect(keys).toContain('Camera');
    expect(keys).toContain('Aperture');
    expect(keys).toContain('ISO');
    expect(keys).toContain('Dimensions');
    expect(keys).not.toContain('Lens');
    expect(keys).not.toContain('Focal');
    expect(keys).not.toContain('Taken');
  });
});
