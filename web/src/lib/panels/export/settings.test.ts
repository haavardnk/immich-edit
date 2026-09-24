import { describe, expect, it } from 'vitest';
import {
  baseOptions,
  defaultExportForm,
  formatLabel,
  immichOptions,
  restoreExportForm,
  type ExportForm
} from './settings';

function form(patch: Partial<ExportForm> = {}): ExportForm {
  return { ...defaultExportForm(), ...patch };
}

describe('restoreExportForm', () => {
  it('round-trips a stored form', () => {
    const saved = form({ format: 'avif', quality: 70, albumIds: ['al'], filenameSuffix: '_warm' });
    expect(restoreExportForm(JSON.parse(JSON.stringify(saved)))).toEqual(saved);
  });

  it('falls back per field on values it does not know', () => {
    const restored = restoreExportForm({
      format: 'bmp',
      quality: 400,
      bitDepth: '12',
      includeExif: 'yes',
      albumIds: ['al', 3],
      stackPrimary: 'both'
    });
    expect(restored).toEqual(form({ quality: 100, albumIds: ['al'] }));
  });
});

describe('baseOptions', () => {
  it.each<[string, Partial<ExportForm>, boolean]>([
    ['webp keeping exif', { format: 'webp', includeExif: true, lossless: false }, true],
    ['webp dropping exif', { format: 'webp', includeExif: false, lossless: false }, false],
    ['webp asked for lossless', { format: 'webp', includeExif: false, lossless: true }, true],
    ['jpeg keeping exif', { format: 'jpeg', includeExif: true, lossless: false }, false]
  ])('resolves lossless for %s', (_name, patch, expected) => {
    expect(baseOptions(form(patch)).lossless).toBe(expected);
  });

  it('passes the encoder settings through unchanged', () => {
    const opts = baseOptions(form({ format: 'tiff', quality: 70, bitDepth: '16' }));
    expect(opts).toMatchObject({
      format: 'tiff',
      quality: 70,
      bitDepth: '16',
      pngCompression: 'default',
      tiffCompression: 'lzw',
      colorSpace: 'srgb'
    });
  });
});

describe('immichOptions', () => {
  it('adds the destination fields to the encoder settings', () => {
    const opts = immichOptions(
      form({ format: 'webp', includeExif: true, albumIds: ['al'], tagIds: ['t'], favorite: true })
    );
    expect(opts).toMatchObject({
      format: 'webp',
      lossless: true,
      albumIds: ['al'],
      tagIds: ['t'],
      favorite: true,
      stackWithOriginal: false,
      stackPrimary: 'edited',
      filenameSuffix: '_edit'
    });
  });
});

describe('formatLabel', () => {
  it.each([
    ['jpeg', 'JPEG'],
    ['jxl', 'JPEG XL']
  ] as const)('labels %s as %s', (format, label) => {
    expect(formatLabel(format)).toBe(label);
  });
});
