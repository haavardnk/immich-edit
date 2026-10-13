import { afterAll, beforeAll, describe, it, expect, vi } from 'vitest';
import {
  fmtAperture,
  fmtFocal,
  fmtShutter,
  fmtDim,
  fmtSize,
  fmtDate,
  fmtDay,
  fmtCamera,
  captureTime,
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

describe('captureTime', () => {
  beforeAll(() => {
    vi.stubEnv('TZ', 'Europe/Oslo');
  });
  afterAll(() => {
    vi.unstubAllEnvs();
  });

  const wall = (iso: string): string =>
    new Date(iso).toLocaleString(undefined, { timeZone: 'UTC' });
  const exifAt = (dateTimeOriginal: string, timeZone: string | null): ExifInfo =>
    ({ dateTimeOriginal, timeZone }) as ExifInfo;

  it.each([
    ['2017-02-21T12:56:14+00:00', null, '2017-02-21T12:56:14Z'],
    ['2026-10-10T15:21:43.83+00:00', 'UTC+1', '2026-10-10T16:21:43Z'],
    ['2026-10-10T12:59:34.29+00:00', 'UTC+2', '2026-10-10T14:59:34Z'],
    ['2026-10-10T12:59:34+00:00', 'Europe/Oslo', '2026-10-10T14:59:34Z'],
    ['2026-01-05T20:00:00+00:00', 'UTC-3:30', '2026-01-05T16:30:00Z'],
    ['2026-01-05T20:00:00+00:00', 'Mars/Olympus', '2026-01-05T20:00:00Z']
  ])('shows %s in zone %s as the camera wall clock', (original, zone, shown) => {
    const exifInfo = exifAt(original, zone);
    const localDateTime = shown.replace('Z', '.000Z');
    expect(fmtDate(captureTime({ exifInfo, localDateTime }))).toBe(wall(shown));
    expect(fmtDate(captureTime({ exifInfo, localDateTime: null }))).toBe(wall(shown));
    expect(fmtDay(captureTime({ exifInfo }))).toBe(
      new Date(shown).toLocaleDateString(undefined, { timeZone: 'UTC' })
    );
  });

  it('falls back to the file time in the viewer zone and drops unreadable dates', () => {
    const created = '2024-01-02T03:04:05Z';
    expect(fmtDate(captureTime({ exifInfo: null, fileCreatedAt: created }))).toBe(
      new Date(created).toLocaleString()
    );
    expect(captureTime({ exifInfo: exifAt('garbage', null), localDateTime: 'garbage' })).toBeNull();
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
      city: null,
      state: null,
      country: null,
      fNumber: 2.8,
      focalLength: null,
      iso: 200,
      exposureTime: null,
      exifImageWidth: 4000,
      exifImageHeight: 3000,
      orientation: null,
      dateTimeOriginal: null,
      timeZone: null,
      rating: null,
      fileSizeInByte: null
    };
    const rows = exifDetailRows({ exifInfo: exif });
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
