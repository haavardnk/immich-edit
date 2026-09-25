import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { captureDate, renderTemplate, templateError } from './filenameTemplate';
import type { ExifInfo } from '$lib/types/asset';

interface NamingCase {
  template: string | null;
  original?: string;
  date?: string | null;
  position?: number;
  total?: number;
  expect?: string;
  error?: string;
}

const CASES_PATH = fileURLToPath(
  new URL('../../../crates/backend/src/services/export/naming_cases.json', import.meta.url)
);
const CASES: NamingCase[] = JSON.parse(readFileSync(CASES_PATH, 'utf8'));

function exifTaken(dateTimeOriginal: string): ExifInfo {
  return {
    make: null,
    model: null,
    lensModel: null,
    fNumber: null,
    focalLength: null,
    iso: null,
    exposureTime: null,
    exifImageWidth: null,
    exifImageHeight: null,
    orientation: null,
    dateTimeOriginal,
    rating: null,
    fileSizeInByte: null
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
      total: c.total ?? 1
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
});
