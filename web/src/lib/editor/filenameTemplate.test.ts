import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { captureDate, insertToken, renderTemplate, templateError } from './filenameTemplate';
import type { ExifInfo } from '#lib/types/asset.ts';

interface NamingCase {
  template: string | null;
  original?: string;
  date?: string | null;
  position?: number;
  total?: number;
  preset?: string | null;
  expect?: string;
  error?: string;
}

const CASES_PATH = fileURLToPath(
  new URL('../../../../crates/backend/src/services/export/naming_cases.json', import.meta.url)
);
const CASES: NamingCase[] = JSON.parse(readFileSync(CASES_PATH, 'utf8'));

function exifTaken(dateTimeOriginal: string): ExifInfo {
  return {
    make: null,
    model: null,
    lensModel: null,
    city: null,
    state: null,
    country: null,
    fNumber: null,
    focalLength: null,
    iso: null,
    exposureTime: null,
    exifImageWidth: null,
    exifImageHeight: null,
    orientation: null,
    dateTimeOriginal,
    timeZone: null,
    rating: null,
    fileSizeInByte: null,
    description: null,
    latitude: null,
    longitude: null
  };
}

describe('filename templates', () => {
  it.each(CASES)('$template matches the shared case', (c) => {
    const template = c.template ?? '';
    if (c.error !== undefined) {
      expect(templateError(template)).toBe(c.error);
      return;
    }
    expect(templateError(template)).toBeNull();
    const name = renderTemplate(template, {
      original: c.original ?? 'IMG_0001.ARW',
      date: c.date ?? null,
      position: c.position ?? 1,
      total: c.total ?? 1,
      preset: c.preset ?? null
    });
    expect(name).toBe(c.expect);
  });

  it('dates a photo by local capture time, then EXIF, then the file', () => {
    const asset = {
      localDateTime: '2025-12-31T23:30:00.000Z',
      exifInfo: exifTaken('2026-01-01T00:30:00+01:00'),
      fileCreatedAt: '2026-01-02T00:00:00.000Z'
    };
    expect(captureDate(asset)).toBe('2025-12-31');
    expect(captureDate({ ...asset, localDateTime: null })).toBe('2026-01-01');
    expect(captureDate({ ...asset, localDateTime: null, exifInfo: null })).toBe('2026-01-02');
    expect(captureDate({ exifInfo: null, fileCreatedAt: 'not a date' })).toBeNull();
  });

  it('inserts a token at the caret', () => {
    expect(insertToken('{date}_{name}_', '{seq}', 14, 14)).toEqual({
      value: '{date}_{name}_{seq}',
      caret: 19
    });
    expect(insertToken('{name}_edit', '{date}_', 0, 0)).toEqual({
      value: '{date}_{name}_edit',
      caret: 7
    });
  });

  it('replaces the selected text with the token', () => {
    expect(insertToken('{name}_edit', '{seq}', 7, 11)).toEqual({
      value: '{name}_{seq}',
      caret: 12
    });
    expect(insertToken('{name}_edit', '{seq}', 9, 2)).toEqual({
      value: '{name}_ed{seq}it',
      caret: 14
    });
  });

  it('appends the token when the caret is unknown or past the end', () => {
    expect(insertToken('{name}', '_{seq}', null, null)).toEqual({
      value: '{name}_{seq}',
      caret: 12
    });
    expect(insertToken('{name}', '{seq}', 40, 40)).toEqual({ value: '{name}{seq}', caret: 11 });
  });
});
